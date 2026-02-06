use crate::core::{ConfigAdapter, ConfigScope, McpServer, Skill};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "HOME directory not found".to_string())
}

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

pub struct CodexAdapter;

#[derive(Debug, Deserialize, Serialize, Default)]
struct CodexTomlConfig {
    #[serde(default)]
    mcp_servers: BTreeMap<String, CodexMcpServerEntry>,
    #[serde(flatten)]
    other: toml::Table,
}

#[derive(Debug, Deserialize, Serialize)]
struct CodexMcpServerEntry {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(flatten)]
    extra: toml::Table,
}

fn default_enabled() -> bool {
    true
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

impl CodexAdapter {
    fn config_path(&self, scope: &ConfigScope) -> Result<PathBuf, String> {
        match scope {
            ConfigScope::Global => Ok(home_dir()?.join(".codex/config.toml")),
            ConfigScope::Project(p) => Ok(p.join(".codex/config.toml")),
        }
    }

    fn skills_dir(&self, scope: &ConfigScope) -> Result<PathBuf, String> {
        match scope {
            ConfigScope::Global => Ok(home_dir()?.join(".agents/skills")),
            ConfigScope::Project(p) => Ok(p.join(".agents/skills")),
        }
    }

    fn rules_path(&self, scope: &ConfigScope) -> Result<PathBuf, String> {
        match scope {
            ConfigScope::Global => Ok(home_dir()?.join(".codex/AGENTS.md")),
            ConfigScope::Project(p) => Ok(p.join("AGENTS.md")),
        }
    }
}

impl ConfigAdapter for CodexAdapter {
    fn tool_name(&self) -> &'static str {
        "Codex"
    }

    fn global_config_path(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".codex")
    }

    fn project_config_path(&self, project: &PathBuf) -> PathBuf {
        project.join(".codex")
    }

    fn read_mcp_servers(&self, scope: &ConfigScope) -> Result<Vec<McpServer>, String> {
        let path = self.config_path(scope)?;
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
        };

        let config: CodexTomlConfig = toml::from_str(&content)
            .map_err(|e| format!("Invalid TOML in {}: {}", path.display(), e))?;

        let mut servers: Vec<McpServer> = config
            .mcp_servers
            .into_iter()
            .map(|(name, entry)| McpServer {
                name,
                command: entry.command,
                args: entry.args,
                env: entry.env.into_iter().collect(),
                url: entry.url,
                enabled: entry.enabled,
            })
            .collect();

        servers.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(servers)
    }

    fn write_mcp_server(&self, server: &McpServer, scope: &ConfigScope) -> Result<(), String> {
        let path = self.config_path(scope)?;

        let mut config = match fs::read_to_string(&path) {
            Ok(c) => toml::from_str::<CodexTomlConfig>(&c)
                .map_err(|e| format!("Invalid TOML in {}: {}", path.display(), e))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => CodexTomlConfig::default(),
            Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
        };

        let existing_extra = config
            .mcp_servers
            .get(&server.name)
            .map(|e| e.extra.clone())
            .unwrap_or_default();

        config.mcp_servers.insert(
            server.name.clone(),
            CodexMcpServerEntry {
                command: server.command.clone(),
                args: server.args.clone(),
                env: server.env.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                url: server.url.clone(),
                enabled: server.enabled,
                extra: existing_extra,
            },
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
        }

        let toml_str = toml::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&path, toml_str)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    fn delete_mcp_server(&self, name: &str, scope: &ConfigScope) -> Result<(), String> {
        let path = self.config_path(scope)?;

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
        };

        let mut config: CodexTomlConfig = toml::from_str(&content)
            .map_err(|e| format!("Invalid TOML in {}: {}", path.display(), e))?;

        config.mcp_servers.remove(name);

        let toml_str = toml::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&path, toml_str)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    fn read_skills(&self, scope: &ConfigScope) -> Result<Vec<Skill>, String> {
        let dir = self.skills_dir(scope)?;
        if !dir.exists() {
            return Ok(vec![]);
        }

        let entries = fs::read_dir(&dir)
            .map_err(|e| format!("Failed to read {}: {}", dir.display(), e))?;

        let mut skills = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_file = path.join("SKILL.md");
            if !skill_file.exists() {
                continue;
            }
            let content = match fs::read_to_string(&skill_file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Some((fm, body)) = parse_skill_frontmatter(&content) {
                skills.push(Skill {
                    name: fm.name,
                    description: fm.description,
                    content: body,
                    path: path.clone(),
                });
            }
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    fn write_skill(&self, skill: &Skill, scope: &ConfigScope) -> Result<(), String> {
        validate_name(&skill.name)?;
        let dir = self.skills_dir(scope)?.join(&skill.name);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create dir {}: {}", dir.display(), e))?;

        let fm = SkillFrontmatter {
            name: skill.name.clone(),
            description: skill.description.clone(),
        };
        let yaml = serde_yaml::to_string(&fm)
            .map_err(|e| format!("Failed to serialize frontmatter: {}", e))?;
        let yaml_clean = yaml.trim_start_matches("---\n");

        let content = format!("---\n{}---\n\n{}", yaml_clean, skill.content);
        let path = dir.join("SKILL.md");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    fn delete_skill(&self, name: &str, scope: &ConfigScope) -> Result<(), String> {
        validate_name(name)?;
        let dir = self.skills_dir(scope)?.join(name);
        if !dir.exists() {
            return Ok(());
        }
        fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to delete {}: {}", dir.display(), e))
    }

    fn read_rules(&self, scope: &ConfigScope) -> Result<String, String> {
        let path = self.rules_path(scope)?;
        match fs::read_to_string(&path) {
            Ok(c) => Ok(c),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(format!("Failed to read {}: {}", path.display(), e)),
        }
    }

    fn write_rules(&self, content: &str, scope: &ConfigScope) -> Result<(), String> {
        let path = self.rules_path(scope)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
        }
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }
}
