use crate::core::{ConfigAdapter, ConfigScope, McpServer, Skill};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

fn validate_name(name: &str) -> Result<(), String> {
    use std::path::Component;
    let path = std::path::Path::new(name);
    let components: Vec<_> = path.components().collect();
    if components.len() != 1 {
        return Err(format!("Invalid name: {}", name));
    }
    match components.first() {
        Some(Component::Normal(_)) => Ok(()),
        _ => Err(format!("Invalid name: {}", name)),
    }
}

pub struct OpenCodeAdapter;

#[derive(Debug, Deserialize, Serialize, Default)]
struct OpenCodeConfig {
    #[serde(default)]
    mcp: BTreeMap<String, OpenCodeMcpEntry>,
    #[serde(flatten)]
    other: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct OpenCodeMcpEntry {
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    server_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    command: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    environment: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(default, skip_serializing, skip_deserializing)]
    _enabled: bool,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SkillFrontmatter {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

fn parse_skill_frontmatter(content: &str) -> Option<(SkillFrontmatter, String)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];
    let body = rest[end + 4..].trim_start().to_string();
    let fm: SkillFrontmatter = serde_yaml::from_str(yaml).ok()?;
    Some((fm, body))
}

impl OpenCodeAdapter {
    fn config_path(&self, scope: &ConfigScope) -> PathBuf {
        match scope {
            ConfigScope::Global => self.global_config_path().join("opencode.json"),
            ConfigScope::Project(p) => self.project_config_path(p).join("opencode.json"),
        }
    }

    fn primary_skills_dir(&self, scope: &ConfigScope) -> PathBuf {
        match scope {
            ConfigScope::Global => self.global_config_path().join("skills"),
            ConfigScope::Project(p) => self.project_config_path(p).join("skills"),
        }
    }

    fn skills_fallback_dirs(&self) -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_default();
        vec![
            home.join(".claude").join("skills"),
            home.join(".agents").join("skills"),
        ]
    }

    fn rules_path(&self, scope: &ConfigScope) -> PathBuf {
        match scope {
            ConfigScope::Global => self.global_config_path().join("AGENTS.md"),
            ConfigScope::Project(p) => p.join("AGENTS.md"),
        }
    }

    fn rules_fallback_path(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".claude").join("CLAUDE.md")
    }
}

impl ConfigAdapter for OpenCodeAdapter {
    fn tool_name(&self) -> &'static str {
        "OpenCode"
    }

    fn global_config_path(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".config").join("opencode")
    }

    fn project_config_path(&self, project: &PathBuf) -> PathBuf {
        project.join(".opencode")
    }

    fn read_mcp_servers(&self, scope: &ConfigScope) -> Result<Vec<McpServer>, String> {
        let path = self.config_path(scope);
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
        };

        let config: OpenCodeConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.display(), e))?;

        let mut servers: Vec<McpServer> = config
            .mcp
            .into_iter()
            .map(|(name, entry)| {
                let is_remote = match entry.server_type.as_deref() {
                    Some("remote") => true,
                    Some("local") => false,
                    _ => entry.url.is_some() && entry.command.as_ref().map_or(true, |c| c.is_empty()),
                };

                let (command, args) = if is_remote {
                    (String::new(), Vec::new())
                } else {
                    match &entry.command {
                        Some(cmd) if !cmd.is_empty() => (cmd[0].clone(), cmd[1..].to_vec()),
                        _ => (String::new(), Vec::new()),
                    }
                };

                McpServer {
                    name,
                    command,
                    args,
                    env: entry.environment,
                    url: if is_remote { entry.url } else { None },
                    enabled: true,
                }
            })
            .collect();

        servers.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(servers)
    }

    fn write_mcp_server(&self, server: &McpServer, scope: &ConfigScope) -> Result<(), String> {
        let path = self.config_path(scope);

        let mut config = match fs::read_to_string(&path) {
            Ok(c) => serde_json::from_str::<OpenCodeConfig>(&c)
                .map_err(|e| format!("Invalid JSON in {}: {}", path.display(), e))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => OpenCodeConfig::default(),
            Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
        };

        let existing_extra = config.mcp.get(&server.name).map(|e| e.extra.clone()).unwrap_or_default();
        let is_remote = server.url.as_ref().is_some_and(|u| !u.is_empty());

        let command = if is_remote || (server.command.is_empty() && server.args.is_empty()) {
            None
        } else {
            let mut cmd = vec![server.command.clone()];
            cmd.extend(server.args.clone());
            Some(cmd)
        };

        config.mcp.insert(server.name.clone(), OpenCodeMcpEntry {
            server_type: Some(if is_remote { "remote" } else { "local" }.to_string()),
            command,
            environment: server.env.clone(),
            url: if is_remote { server.url.clone() } else { None },
            _enabled: false,
            extra: existing_extra,
        });

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
        }

        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    fn delete_mcp_server(&self, name: &str, scope: &ConfigScope) -> Result<(), String> {
        let path = self.config_path(scope);

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
        };

        let mut config: OpenCodeConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.display(), e))?;

        config.mcp.remove(name);

        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    fn read_skills(&self, scope: &ConfigScope) -> Result<Vec<Skill>, String> {
        let dirs: Vec<PathBuf> = match scope {
            ConfigScope::Global => {
                let mut all = vec![self.primary_skills_dir(scope)];
                all.extend(self.skills_fallback_dirs());
                all
            }
            ConfigScope::Project(_) => vec![self.primary_skills_dir(scope)],
        };

        let mut skills = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        for dir in dirs {
            if !dir.is_dir() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&dir) else { continue };

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let skill_file = path.join("SKILL.md");
                let Ok(content) = fs::read_to_string(&skill_file) else { continue };
                if let Some((fm, body)) = parse_skill_frontmatter(&content) {
                    if seen_names.insert(fm.name.clone()) {
                        skills.push(Skill {
                            name: fm.name,
                            description: fm.description,
                            content: body,
                            path: path.clone(),
                        });
                    }
                }
            }
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    fn write_skill(&self, skill: &Skill, scope: &ConfigScope) -> Result<(), String> {
        validate_name(&skill.name)?;
        let dir = self.primary_skills_dir(scope).join(&skill.name);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create dir {}: {}", dir.display(), e))?;

        let fm = SkillFrontmatter { name: skill.name.clone(), description: skill.description.clone() };
        let yaml = serde_yaml::to_string(&fm)
            .map_err(|e| format!("Failed to serialize frontmatter: {}", e))?;
        let yaml_clean = yaml.trim_start_matches("---\n");

        let content = format!("---\n{}---\n\n{}", yaml_clean, skill.content);
        let path = dir.join("SKILL.md");
        fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    fn delete_skill(&self, name: &str, scope: &ConfigScope) -> Result<(), String> {
        validate_name(name)?;
        let dir = self.primary_skills_dir(scope).join(name);
        if !dir.exists() {
            return Ok(());
        }
        fs::remove_dir_all(&dir).map_err(|e| format!("Failed to delete {}: {}", dir.display(), e))
    }

    fn read_rules(&self, scope: &ConfigScope) -> Result<String, String> {
        let path = self.rules_path(scope);
        match fs::read_to_string(&path) {
            Ok(c) => Ok(c),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if matches!(scope, ConfigScope::Global) {
                    let fallback = self.rules_fallback_path();
                    match fs::read_to_string(&fallback) {
                        Ok(c) => Ok(c),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
                        Err(e) => Err(format!("Failed to read {}: {}", fallback.display(), e)),
                    }
                } else {
                    Ok(String::new())
                }
            }
            Err(e) => Err(format!("Failed to read {}: {}", path.display(), e)),
        }
    }

    fn write_rules(&self, content: &str, scope: &ConfigScope) -> Result<(), String> {
        let path = self.rules_path(scope);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
        }
        fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }
}
