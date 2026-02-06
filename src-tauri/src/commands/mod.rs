use crate::adapters::{ClaudeAdapter, CodexAdapter, GeminiAdapter, OpenCodeAdapter};
use crate::core::{convert_mcp_server, ConfigAdapter, ConfigScope, McpServer, Project, Skill, ToolType};
use crate::db::ProjectRepo;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

pub struct DbState(pub Mutex<Connection>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyResult {
    pub server: Option<McpServer>,
    pub warnings: Vec<String>,
    pub skipped: bool,
}

fn scope_from(project_path: Option<String>) -> ConfigScope {
    match project_path {
        Some(p) if !p.trim().is_empty() => ConfigScope::Project(PathBuf::from(p.trim())),
        _ => ConfigScope::Global,
    }
}

fn get_adapter(tool: ToolType) -> Box<dyn ConfigAdapter> {
    match tool {
        ToolType::ClaudeCode => Box::new(ClaudeAdapter),
        ToolType::Codex => Box::new(CodexAdapter),
        ToolType::Gemini => Box::new(GeminiAdapter),
        ToolType::OpenCode => Box::new(OpenCodeAdapter),
    }
}

#[tauri::command]
pub fn get_mcp_servers(tool: ToolType, project_path: Option<String>) -> Result<Vec<McpServer>, String> {
    get_adapter(tool).read_mcp_servers(&scope_from(project_path))
}

#[tauri::command]
pub fn save_mcp_server(tool: ToolType, server: McpServer, project_path: Option<String>) -> Result<(), String> {
    get_adapter(tool).write_mcp_server(&server, &scope_from(project_path))
}

#[tauri::command]
pub fn delete_mcp_server(tool: ToolType, name: String, project_path: Option<String>) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Server name cannot be empty".into());
    }
    get_adapter(tool).delete_mcp_server(name, &scope_from(project_path))
}

#[tauri::command]
pub fn get_skills(tool: ToolType, project_path: Option<String>) -> Result<Vec<Skill>, String> {
    get_adapter(tool).read_skills(&scope_from(project_path))
}

#[tauri::command]
pub fn save_skill(tool: ToolType, skill: Skill, project_path: Option<String>) -> Result<(), String> {
    get_adapter(tool).write_skill(&skill, &scope_from(project_path))
}

#[tauri::command]
pub fn delete_skill(tool: ToolType, name: String, project_path: Option<String>) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Skill name cannot be empty".into());
    }
    get_adapter(tool).delete_skill(name, &scope_from(project_path))
}

#[tauri::command]
pub fn get_rules(tool: ToolType, project_path: Option<String>) -> Result<String, String> {
    get_adapter(tool).read_rules(&scope_from(project_path))
}

#[tauri::command]
pub fn save_rules(tool: ToolType, content: String, project_path: Option<String>) -> Result<(), String> {
    get_adapter(tool).write_rules(&content, &scope_from(project_path))
}

#[tauri::command]
pub fn list_projects(db: State<'_, DbState>) -> Result<Vec<Project>, String> {
    let mut projects = {
        let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
        ProjectRepo::new(&conn).list()?
    };
    for p in &mut projects {
        p.tools = detect_tools(Path::new(&p.path));
    }
    Ok(projects)
}

#[tauri::command]
pub fn add_project(path: String, db: State<'_, DbState>) -> Result<Project, String> {
    let mut project = {
        let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
        ProjectRepo::new(&conn).add(&path)?
    };
    project.tools = detect_tools(Path::new(&project.path));
    Ok(project)
}

#[tauri::command]
pub fn remove_project(id: i64, db: State<'_, DbState>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    ProjectRepo::new(&conn).remove(id)
}

#[tauri::command]
pub fn copy_mcp_to_tool(
    from_tool: ToolType,
    to_tool: ToolType,
    server_name: String,
    project_path: Option<String>,
) -> Result<CopyResult, String> {
    let name = server_name.trim();
    if name.is_empty() {
        return Err("Server name cannot be empty".into());
    }
    let scope = scope_from(project_path);
    let to_adapter = get_adapter(to_tool);

    if to_adapter.read_mcp_servers(&scope)?.iter().any(|s| s.name == name) {
        return Ok(CopyResult { server: None, warnings: vec![], skipped: true });
    }

    let server = get_adapter(from_tool)
        .read_mcp_servers(&scope)?
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("MCP server not found: {}", name))?;

    let result = convert_mcp_server(&server, from_tool, to_tool);
    to_adapter.write_mcp_server(&result.server, &scope)?;
    Ok(CopyResult { server: Some(result.server), warnings: result.warnings, skipped: false })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCopyResult {
    pub skipped: bool,
}

#[tauri::command]
pub fn copy_skill_to_tool(
    from_tool: ToolType,
    to_tool: ToolType,
    skill_name: String,
    project_path: Option<String>,
) -> Result<SkillCopyResult, String> {
    let name = skill_name.trim();
    if name.is_empty() {
        return Err("Skill name cannot be empty".into());
    }
    let scope = scope_from(project_path);
    let to_adapter = get_adapter(to_tool);

    if to_adapter.read_skills(&scope)?.iter().any(|s| s.name == name) {
        return Ok(SkillCopyResult { skipped: true });
    }

    let skill = get_adapter(from_tool)
        .read_skills(&scope)?
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("Skill not found: {}", name))?;

    to_adapter.write_skill(&skill, &scope)?;
    Ok(SkillCopyResult { skipped: false })
}

fn detect_tools(project_path: &Path) -> Vec<ToolType> {
    let checks: &[(&str, ToolType)] = &[
        (".claude", ToolType::ClaudeCode),
        (".codex", ToolType::Codex),
        (".gemini", ToolType::Gemini),
        (".opencode", ToolType::OpenCode),
    ];
    checks
        .iter()
        .filter(|(dir, _)| project_path.join(dir).is_dir())
        .map(|(_, t)| *t)
        .collect()
}

#[tauri::command]
pub fn detect_project_tools(path: String) -> Result<Vec<ToolType>, String> {
    let p = Path::new(path.trim());
    if !p.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    Ok(detect_tools(p))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfigSummary {
    pub tool: ToolType,
    pub mcp_count: usize,
    pub skills_count: usize,
    pub has_rules: bool,
}

#[tauri::command]
pub fn get_project_config_summary(path: String) -> Result<Vec<ProjectConfigSummary>, String> {
    let p = Path::new(path.trim());
    if !p.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    let tools = detect_tools(p);
    let scope = ConfigScope::Project(PathBuf::from(path.trim()));
    let mut summaries = Vec::new();
    for tool in tools {
        let adapter = get_adapter(tool);
        let mcp_count = adapter.read_mcp_servers(&scope).unwrap_or_default().len();
        let skills_count = adapter.read_skills(&scope).unwrap_or_default().len();
        let has_rules = adapter.read_rules(&scope).map(|r| !r.trim().is_empty()).unwrap_or(false);
        summaries.push(ProjectConfigSummary { tool, mcp_count, skills_count, has_rules });
    }
    Ok(summaries)
}

pub fn register_commands() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        get_mcp_servers,
        save_mcp_server,
        delete_mcp_server,
        get_skills,
        save_skill,
        delete_skill,
        get_rules,
        save_rules,
        list_projects,
        add_project,
        remove_project,
        copy_mcp_to_tool,
        copy_skill_to_tool,
        detect_project_tools,
        get_project_config_summary
    ]
}
