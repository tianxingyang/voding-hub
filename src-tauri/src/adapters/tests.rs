use crate::adapters::{ClaudeAdapter, CodexAdapter, GeminiAdapter, OpenCodeAdapter};
use crate::core::{ConfigAdapter, ConfigScope, McpServer, Skill};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn make_mcp_server(name: &str) -> McpServer {
    McpServer {
        name: name.into(),
        command: "test-cmd".into(),
        args: vec!["--arg1".into(), "value".into()],
        env: HashMap::from([("KEY".into(), "VAL".into())]),
        url: None,
        enabled: true,
    }
}

fn make_skill(name: &str) -> Skill {
    Skill {
        name: name.into(),
        description: Some("Test skill desc".into()),
        content: "# Test content".into(),
        path: PathBuf::new(),
    }
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(prefix: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "voding-test-{}-{}-{}-{:?}",
            prefix,
            std::process::id(),
            id,
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

mod claude_adapter_tests {
    use super::*;

    fn setup() -> (TempDir, ConfigScope) {
        let tmp = TempDir::new("claude");
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        (tmp, scope)
    }

    #[test]
    fn mcp_read_empty() {
        let (_tmp, scope) = setup();
        let servers = ClaudeAdapter.read_mcp_servers(&scope).unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn mcp_write_and_read() {
        let (tmp, scope) = setup();
        let server = make_mcp_server("test-server");
        ClaudeAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = ClaudeAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "test-server");
        assert_eq!(servers[0].command, "test-cmd");
        assert_eq!(servers[0].args, vec!["--arg1", "value"]);
        assert_eq!(servers[0].env.get("KEY"), Some(&"VAL".to_string()));

        let file = fs::read_to_string(tmp.path().join(".claude/.mcp.json")).unwrap();
        assert!(file.contains("mcpServers"));
        assert!(file.contains("test-server"));
    }

    #[test]
    fn mcp_delete() {
        let (_tmp, scope) = setup();
        let server = make_mcp_server("to-delete");
        ClaudeAdapter.write_mcp_server(&server, &scope).unwrap();

        ClaudeAdapter.delete_mcp_server("to-delete", &scope).unwrap();
        let servers = ClaudeAdapter.read_mcp_servers(&scope).unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn skills_write_and_read() {
        let (tmp, scope) = setup();
        let skill = make_skill("test-skill");
        ClaudeAdapter.write_skill(&skill, &scope).unwrap();

        let skills = ClaudeAdapter.read_skills(&scope).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, Some("Test skill desc".into()));
        assert!(skills[0].content.contains("# Test content"));

        let file = tmp.path().join(".claude/skills/test-skill/SKILL.md");
        assert!(file.exists());
    }

    #[test]
    fn skills_delete() {
        let (_tmp, scope) = setup();
        let skill = make_skill("del-skill");
        ClaudeAdapter.write_skill(&skill, &scope).unwrap();
        ClaudeAdapter.delete_skill("del-skill", &scope).unwrap();

        let skills = ClaudeAdapter.read_skills(&scope).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn rules_write_and_read() {
        let (tmp, scope) = setup();
        ClaudeAdapter.write_rules("# Rules content", &scope).unwrap();

        let rules = ClaudeAdapter.read_rules(&scope).unwrap();
        assert_eq!(rules, "# Rules content");

        let file = tmp.path().join("CLAUDE.md");
        assert!(file.exists());
    }
}

mod codex_adapter_tests {
    use super::*;

    fn setup() -> (TempDir, ConfigScope) {
        let tmp = TempDir::new("codex");
        fs::create_dir_all(tmp.path().join(".codex")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        (tmp, scope)
    }

    #[test]
    fn mcp_read_empty() {
        let (_tmp, scope) = setup();
        let servers = CodexAdapter.read_mcp_servers(&scope).unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn mcp_write_and_read() {
        let (tmp, scope) = setup();
        let mut server = make_mcp_server("codex-server");
        server.enabled = false;
        CodexAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = CodexAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "codex-server");
        assert!(!servers[0].enabled);

        let file = fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap();
        assert!(file.contains("[mcp_servers.codex-server]"));
        assert!(file.contains("enabled = false"));
    }

    #[test]
    fn mcp_with_url() {
        let (_tmp, scope) = setup();
        let mut server = make_mcp_server("remote-server");
        server.url = Some("http://localhost:8080".into());
        CodexAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = CodexAdapter.read_mcp_servers(&scope).unwrap();
        assert!(!servers.is_empty());
        assert_eq!(servers[0].url, Some("http://localhost:8080".into()));
    }

    #[test]
    fn mcp_delete() {
        let (_tmp, scope) = setup();
        CodexAdapter.write_mcp_server(&make_mcp_server("del"), &scope).unwrap();
        CodexAdapter.delete_mcp_server("del", &scope).unwrap();
        assert!(CodexAdapter.read_mcp_servers(&scope).unwrap().is_empty());
    }

    #[test]
    fn skills_write_and_read() {
        let (tmp, _scope) = setup();
        fs::create_dir_all(tmp.path().join(".agents/skills")).unwrap();
        let skill_scope = ConfigScope::Project(tmp.path().clone());
        let skill = make_skill("codex-skill");
        CodexAdapter.write_skill(&skill, &skill_scope).unwrap();

        let skills = CodexAdapter.read_skills(&skill_scope).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "codex-skill");
    }

    #[test]
    fn rules_write_and_read() {
        let (tmp, scope) = setup();
        CodexAdapter.write_rules("# Codex rules", &scope).unwrap();

        let rules = CodexAdapter.read_rules(&scope).unwrap();
        assert!(rules.contains("# Codex rules"));

        let file = tmp.path().join("AGENTS.md");
        assert!(file.exists());
    }
}

mod gemini_adapter_tests {
    use super::*;

    fn setup() -> (TempDir, ConfigScope) {
        let tmp = TempDir::new("gemini");
        fs::create_dir_all(tmp.path().join(".gemini")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        (tmp, scope)
    }

    #[test]
    fn mcp_read_empty() {
        let (_tmp, scope) = setup();
        let servers = GeminiAdapter.read_mcp_servers(&scope).unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn mcp_write_and_read() {
        let (tmp, scope) = setup();
        let server = make_mcp_server("gemini-server");
        GeminiAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = GeminiAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "gemini-server");
        assert_eq!(servers[0].command, "test-cmd");

        let file = fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap();
        assert!(file.contains("mcpServers"));
    }

    #[test]
    fn mcp_delete() {
        let (_tmp, scope) = setup();
        GeminiAdapter.write_mcp_server(&make_mcp_server("del"), &scope).unwrap();
        GeminiAdapter.delete_mcp_server("del", &scope).unwrap();
        assert!(GeminiAdapter.read_mcp_servers(&scope).unwrap().is_empty());
    }

    #[test]
    fn skills_write_and_read() {
        let (tmp, _scope) = setup();
        fs::create_dir_all(tmp.path().join(".gemini/skills")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        let skill = make_skill("gemini-skill");
        GeminiAdapter.write_skill(&skill, &scope).unwrap();

        let skills = GeminiAdapter.read_skills(&scope).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "gemini-skill");
    }

    #[test]
    fn rules_write_and_read() {
        let (tmp, scope) = setup();
        GeminiAdapter.write_rules("# Gemini rules", &scope).unwrap();

        let rules = GeminiAdapter.read_rules(&scope).unwrap();
        assert!(rules.contains("# Gemini rules"));

        let file = tmp.path().join("GEMINI.md");
        assert!(file.exists());
    }
}

mod opencode_adapter_tests {
    use super::*;

    fn setup() -> (TempDir, ConfigScope) {
        let tmp = TempDir::new("opencode");
        fs::create_dir_all(tmp.path().join(".opencode")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        (tmp, scope)
    }

    #[test]
    fn mcp_read_empty() {
        let (_tmp, scope) = setup();
        let servers = OpenCodeAdapter.read_mcp_servers(&scope).unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn mcp_write_and_read_local() {
        let (tmp, scope) = setup();
        let server = make_mcp_server("oc-server");
        OpenCodeAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = OpenCodeAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "oc-server");
        assert_eq!(servers[0].command, "test-cmd");
        assert!(servers[0].url.is_none());

        let file = fs::read_to_string(tmp.path().join(".opencode/opencode.json")).unwrap();
        assert!(file.contains("\"type\": \"local\""));
    }

    #[test]
    fn mcp_write_and_read_remote() {
        let (tmp, scope) = setup();
        let mut server = make_mcp_server("remote");
        server.url = Some("http://localhost:9000".into());
        server.command = String::new();
        server.args = vec![];
        OpenCodeAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = OpenCodeAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].url, Some("http://localhost:9000".into()));
        assert!(servers[0].command.is_empty());

        let file = fs::read_to_string(tmp.path().join(".opencode/opencode.json")).unwrap();
        assert!(file.contains("\"type\": \"remote\""));
    }

    #[test]
    fn mcp_delete() {
        let (_tmp, scope) = setup();
        OpenCodeAdapter.write_mcp_server(&make_mcp_server("del"), &scope).unwrap();
        OpenCodeAdapter.delete_mcp_server("del", &scope).unwrap();
        assert!(OpenCodeAdapter.read_mcp_servers(&scope).unwrap().is_empty());
    }

    #[test]
    fn skills_write_and_read() {
        let (tmp, _scope) = setup();
        fs::create_dir_all(tmp.path().join(".opencode/skills")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        let skill = make_skill("oc-skill");
        OpenCodeAdapter.write_skill(&skill, &scope).unwrap();

        let skills = OpenCodeAdapter.read_skills(&scope).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "oc-skill");
    }

    #[test]
    fn rules_write_and_read() {
        let (tmp, scope) = setup();
        OpenCodeAdapter.write_rules("# OC rules", &scope).unwrap();

        let rules = OpenCodeAdapter.read_rules(&scope).unwrap();
        assert!(rules.contains("# OC rules"));

        let file = tmp.path().join("AGENTS.md");
        assert!(file.exists());
    }
}

mod cross_tool_copy_tests {
    use super::*;
    use crate::core::{convert_mcp_server, ToolType};

    fn setup_multi() -> (TempDir, ConfigScope) {
        let tmp = TempDir::new("cross");
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::create_dir_all(tmp.path().join(".codex")).unwrap();
        fs::create_dir_all(tmp.path().join(".gemini")).unwrap();
        fs::create_dir_all(tmp.path().join(".opencode")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        (tmp, scope)
    }

    #[test]
    fn copy_mcp_claude_to_codex() {
        let (_tmp, scope) = setup_multi();
        let server = make_mcp_server("shared-server");
        ClaudeAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = ClaudeAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(servers.len(), 1);

        let result = convert_mcp_server(&servers[0], ToolType::ClaudeCode, ToolType::Codex);
        CodexAdapter.write_mcp_server(&result.server, &scope).unwrap();

        let codex_servers = CodexAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(codex_servers.len(), 1);
        assert_eq!(codex_servers[0].name, "shared-server");
        assert_eq!(codex_servers[0].command, "test-cmd");
    }

    #[test]
    fn copy_mcp_codex_to_gemini_with_url_warning() {
        let (_tmp, scope) = setup_multi();
        let mut server = make_mcp_server("remote-srv");
        server.url = Some("http://remote:8080".into());
        server.enabled = false;
        CodexAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = CodexAdapter.read_mcp_servers(&scope).unwrap();
        let result = convert_mcp_server(&servers[0], ToolType::Codex, ToolType::Gemini);

        assert!(result.warnings.iter().any(|w| w.contains("url")));
        assert!(result.warnings.iter().any(|w| w.contains("enabled")));
        assert!(result.server.url.is_none());
        assert!(result.server.enabled);

        GeminiAdapter.write_mcp_server(&result.server, &scope).unwrap();
        let gemini_servers = GeminiAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(gemini_servers.len(), 1);
    }

    #[test]
    fn copy_mcp_opencode_to_claude() {
        let (_tmp, scope) = setup_multi();
        let server = make_mcp_server("oc-srv");
        OpenCodeAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = OpenCodeAdapter.read_mcp_servers(&scope).unwrap();
        let result = convert_mcp_server(&servers[0], ToolType::OpenCode, ToolType::ClaudeCode);

        ClaudeAdapter.write_mcp_server(&result.server, &scope).unwrap();
        let claude_servers = ClaudeAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(claude_servers.len(), 1);
        assert_eq!(claude_servers[0].name, "oc-srv");
    }

    #[test]
    fn copy_skill_claude_to_gemini() {
        let (tmp, _scope) = setup_multi();
        fs::create_dir_all(tmp.path().join(".gemini/skills")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());

        let skill = make_skill("shared-skill");
        ClaudeAdapter.write_skill(&skill, &scope).unwrap();

        let skills = ClaudeAdapter.read_skills(&scope).unwrap();
        assert_eq!(skills.len(), 1);

        GeminiAdapter.write_skill(&skills[0], &scope).unwrap();

        let gemini_skills = GeminiAdapter.read_skills(&scope).unwrap();
        assert_eq!(gemini_skills.len(), 1);
        assert_eq!(gemini_skills[0].name, "shared-skill");
        assert_eq!(gemini_skills[0].description, Some("Test skill desc".into()));
    }

    #[test]
    fn copy_mcp_gemini_to_opencode() {
        let (_tmp, scope) = setup_multi();
        let server = make_mcp_server("gem-srv");
        GeminiAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = GeminiAdapter.read_mcp_servers(&scope).unwrap();
        let result = convert_mcp_server(&servers[0], ToolType::Gemini, ToolType::OpenCode);

        OpenCodeAdapter.write_mcp_server(&result.server, &scope).unwrap();
        let oc_servers = OpenCodeAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(oc_servers.len(), 1);
        assert_eq!(oc_servers[0].name, "gem-srv");
    }

    #[test]
    fn copy_remote_server_codex_to_opencode() {
        let (_tmp, scope) = setup_multi();
        let mut server = make_mcp_server("remote");
        server.url = Some("http://localhost:3000".into());
        server.command = String::new();
        server.args = vec![];
        CodexAdapter.write_mcp_server(&server, &scope).unwrap();

        let servers = CodexAdapter.read_mcp_servers(&scope).unwrap();
        let result = convert_mcp_server(&servers[0], ToolType::Codex, ToolType::OpenCode);

        assert!(result.warnings.is_empty() || result.warnings.iter().all(|w| !w.contains("url")));
        OpenCodeAdapter.write_mcp_server(&result.server, &scope).unwrap();

        let oc_servers = OpenCodeAdapter.read_mcp_servers(&scope).unwrap();
        assert_eq!(oc_servers.len(), 1);
        assert_eq!(oc_servers[0].url, Some("http://localhost:3000".into()));
    }
}

mod skill_name_validation_tests {
    use super::*;

    fn setup() -> (TempDir, ConfigScope) {
        let tmp = TempDir::new("validation");
        fs::create_dir_all(tmp.path().join(".claude/skills")).unwrap();
        fs::create_dir_all(tmp.path().join(".gemini/skills")).unwrap();
        fs::create_dir_all(tmp.path().join(".opencode/skills")).unwrap();
        fs::create_dir_all(tmp.path().join(".agents/skills")).unwrap();
        let scope = ConfigScope::Project(tmp.path().clone());
        (tmp, scope)
    }

    #[test]
    fn claude_rejects_path_traversal() {
        let (_tmp, scope) = setup();
        let mut skill = make_skill("valid");
        skill.name = "../evil".into();
        let result = ClaudeAdapter.write_skill(&skill, &scope);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid name"));
    }

    #[test]
    fn claude_rejects_nested_path() {
        let (_tmp, scope) = setup();
        let mut skill = make_skill("valid");
        skill.name = "a/b".into();
        let result = ClaudeAdapter.write_skill(&skill, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn claude_delete_rejects_invalid_name() {
        let (_tmp, scope) = setup();
        let result = ClaudeAdapter.delete_skill("../evil", &scope);
        assert!(result.is_err());
    }

    #[test]
    fn gemini_rejects_path_traversal() {
        let (_tmp, scope) = setup();
        let mut skill = make_skill("valid");
        skill.name = "../evil".into();
        let result = GeminiAdapter.write_skill(&skill, &scope);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid name"));
    }

    #[test]
    fn gemini_rejects_nested_path() {
        let (_tmp, scope) = setup();
        let mut skill = make_skill("valid");
        skill.name = "a/b".into();
        let result = GeminiAdapter.write_skill(&skill, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn opencode_rejects_dot_name() {
        let (_tmp, scope) = setup();
        let mut skill = make_skill("valid");
        skill.name = ".".into();
        let result = OpenCodeAdapter.write_skill(&skill, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn codex_rejects_path_traversal() {
        let (_tmp, scope) = setup();
        let mut skill = make_skill("valid");
        skill.name = "../hack".into();
        let result = CodexAdapter.write_skill(&skill, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn gemini_delete_rejects_invalid_name() {
        let (_tmp, scope) = setup();
        let result = GeminiAdapter.delete_skill("../evil", &scope);
        assert!(result.is_err());
    }

    #[test]
    fn opencode_delete_rejects_nested_path() {
        let (_tmp, scope) = setup();
        let result = OpenCodeAdapter.delete_skill("a/b/c", &scope);
        assert!(result.is_err());
    }
}
