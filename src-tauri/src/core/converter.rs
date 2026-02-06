use crate::core::{McpServer, ToolType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub server: McpServer,
    pub warnings: Vec<String>,
}

pub fn config_format(tool: ToolType) -> ConfigFormat {
    match tool {
        ToolType::Codex => ConfigFormat::Toml,
        _ => ConfigFormat::Json,
    }
}

pub fn requires_format_conversion(from: ToolType, to: ToolType) -> bool {
    config_format(from) != config_format(to)
}

pub fn convert_mcp_server(server: &McpServer, from: ToolType, to: ToolType) -> ConversionResult {
    let mut out = server.clone();
    let mut warnings = Vec::new();

    if from == to {
        return ConversionResult { server: out, warnings };
    }

    // Normalize empty url to None
    if out.url.as_ref().is_some_and(|u| u.trim().is_empty()) {
        out.url = None;
    }

    // Handle enabled field (only Codex supports it, only warn when from Codex)
    if from == ToolType::Codex && to != ToolType::Codex && !out.enabled {
        add_warning(&mut warnings, format!(
            "`enabled=false` not supported by {}, will be treated as enabled",
            tool_display_name(to)
        ));
        out.enabled = true;
    }

    // Handle url field
    let mut is_remote = out.url.as_ref().is_some_and(|u| !u.trim().is_empty());
    if is_remote && !supports_url(to) {
        add_warning(&mut warnings, format!(
            "`url` not supported by {}, dropped",
            tool_display_name(to)
        ));
        out.url = None;
        is_remote = false;
    }

    // Handle OpenCode command array semantics for remote servers
    if to == ToolType::OpenCode && is_remote {
        if !out.command.is_empty() || !out.args.is_empty() {
            add_warning(&mut warnings,
                "OpenCode remote servers ignore `command`/`args`, dropped".into()
            );
        }
        out.command.clear();
        out.args.clear();
    }

    // Validate command for local servers
    if !is_remote && out.command.trim().is_empty() {
        add_warning(&mut warnings, "Empty `command` for local server".into());
    }

    ConversionResult { server: out, warnings }
}

fn tool_display_name(tool: ToolType) -> &'static str {
    match tool {
        ToolType::ClaudeCode => "Claude Code",
        ToolType::Codex => "Codex",
        ToolType::Gemini => "Gemini",
        ToolType::OpenCode => "OpenCode",
    }
}

fn supports_url(tool: ToolType) -> bool {
    matches!(tool, ToolType::Codex | ToolType::OpenCode)
}

fn add_warning(warnings: &mut Vec<String>, msg: String) {
    if !warnings.contains(&msg) {
        warnings.push(msg);
    }
}

pub fn convert_mcp_servers(
    servers: &[McpServer],
    from: ToolType,
    to: ToolType,
) -> Vec<ConversionResult> {
    servers.iter().map(|s| convert_mcp_server(s, from, to)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_server(name: &str, cmd: &str, enabled: bool) -> McpServer {
        McpServer {
            name: name.into(),
            command: cmd.into(),
            args: vec![],
            env: HashMap::new(),
            url: None,
            enabled,
        }
    }

    #[test]
    fn same_tool_no_warnings() {
        let s = make_server("test", "cmd", true);
        let r = convert_mcp_server(&s, ToolType::ClaudeCode, ToolType::ClaudeCode);
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn enabled_false_warns_for_non_codex() {
        let s = make_server("test", "cmd", false);
        let r = convert_mcp_server(&s, ToolType::Codex, ToolType::ClaudeCode);
        assert!(r.warnings.iter().any(|w| w.contains("enabled")));
        assert!(r.server.enabled);
    }

    #[test]
    fn format_detection() {
        assert_eq!(config_format(ToolType::Codex), ConfigFormat::Toml);
        assert_eq!(config_format(ToolType::ClaudeCode), ConfigFormat::Json);
        assert!(requires_format_conversion(ToolType::ClaudeCode, ToolType::Codex));
        assert!(!requires_format_conversion(ToolType::ClaudeCode, ToolType::Gemini));
    }

    #[test]
    fn remote_to_unsupported_warns_empty_command() {
        let mut s = make_server("test", "", true);
        s.url = Some("http://example.com".into());
        let r = convert_mcp_server(&s, ToolType::Codex, ToolType::ClaudeCode);
        assert!(r.warnings.iter().any(|w| w.contains("url")));
        assert!(r.warnings.iter().any(|w| w.contains("Empty")));
        assert!(r.server.url.is_none());
    }
}
