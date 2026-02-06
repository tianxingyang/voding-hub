## 1. Project Setup

- [x] 1.1 Initialize Tauri 2.0 + React + TypeScript + Vite project
- [x] 1.2 Configure Cargo.toml with dependencies: rusqlite, notify, serde, toml, serde_json
- [x] 1.3 Configure package.json with dependencies: zustand, @tauri-apps/api
- [x] 1.4 Set up project directory structure (commands/, adapters/, core/, db/)

## 2. Core Data Models

- [x] 2.1 Define `McpServer` struct with fields: name, command, args, env, url, enabled
- [x] 2.2 Define `Skill` struct with fields: name, description, content, path
- [x] 2.3 Define `Project` struct with fields: id, name, path, tools
- [x] 2.4 Define `ToolType` enum: ClaudeCode, Codex, Gemini, OpenCode
- [x] 2.5 Define `ConfigScope` enum: Global, Project(PathBuf)

## 3. Config Adapter Trait

- [x] 3.1 Define `ConfigAdapter` trait with MCP operations
- [x] 3.2 Add Skills operations to trait
- [x] 3.3 Add Rules operations to trait
- [x] 3.4 Add path helper methods: global_config_path(), project_config_path()

## 4. Claude Code Adapter

- [x] 4.1 Implement `read_mcp_servers` - parse `~/.claude/.mcp.json`
- [x] 4.2 Implement `write_mcp_server` - update .mcp.json
- [x] 4.3 Implement `read_skills` - scan `~/.claude/skills/*/SKILL.md`
- [x] 4.4 Implement `write_skill` - create skill directory and SKILL.md
- [x] 4.5 Implement `read_rules` / `write_rules` for CLAUDE.md

## 5. Codex Adapter

- [x] 5.1 Implement `read_mcp_servers` - parse `~/.codex/config.toml` [mcp_servers.*]
- [x] 5.2 Implement `write_mcp_server` - update config.toml
- [x] 5.3 Implement `read_skills` - scan `~/.agents/skills/*/SKILL.md`
- [x] 5.4 Implement `write_skill` - create skill in .agents/skills/
- [x] 5.5 Implement `read_rules` / `write_rules` for AGENTS.md

## 6. Gemini Adapter

- [x] 6.1 Implement `read_mcp_servers` - parse `~/.gemini/settings.json` mcpServers
- [x] 6.2 Implement `write_mcp_server` - update settings.json
- [x] 6.3 Implement `read_skills` - scan `~/.gemini/skills/*/SKILL.md`
- [x] 6.4 Implement `write_skill` - create skill directory
- [x] 6.5 Implement `read_rules` / `write_rules` for GEMINI.md

## 7. OpenCode Adapter

- [x] 7.1 Implement `read_mcp_servers` - parse `~/.config/opencode/opencode.json` mcp
- [x] 7.2 Implement `write_mcp_server` - update opencode.json
- [x] 7.3 Implement `read_skills` with fallback paths (.opencode/, .claude/, .agents/)
- [x] 7.4 Implement `write_skill` - create skill in .config/opencode/skills/
- [x] 7.5 Implement `read_rules` / `write_rules` with Claude compatibility

## 8. Format Converter

- [x] 8.1 Implement JSON to TOML converter for MCP configs
- [x] 8.2 Implement TOML to JSON converter for MCP configs
- [x] 8.3 Handle field mapping differences (env vs environment, command array vs string)
- [x] 8.4 Add warning generation for unsupported fields

## 9. Database Layer

- [x] 9.1 Create SQLite database initialization
- [x] 9.2 Define projects table schema (id, name, path, created_at, updated_at)
- [x] 9.3 Implement CRUD operations for projects
- [x] 9.4 Add path validation on project load

## 10. File Watcher

- [x] 10.1 Set up notify watcher for global config directories
- [x] 10.2 Implement debounce logic (500ms)
- [x] 10.3 Create Tauri event emission for file changes
- [x] 10.4 Add project-level watch management (start/stop on add/remove)

## 11. Tauri Commands

- [x] 11.1 Create config commands: get_mcp_servers, save_mcp_server, delete_mcp_server
- [x] 11.2 Create skills commands: get_skills, save_skill, delete_skill, copy_skill
- [x] 11.3 Create rules commands: get_rules, save_rules
- [x] 11.4 Create project commands: list_projects, add_project, remove_project
- [x] 11.5 Create copy commands: copy_mcp_to_tool, copy_skill_to_tool

## 12. Frontend - State Management

- [x] 12.1 Create Zustand store for config state
- [x] 12.2 Add actions for loading/saving configs
- [x] 12.3 Set up Tauri event listeners for file changes
- [x] 12.4 Implement optimistic updates with rollback

## 13. Frontend - Layout & Navigation

- [x] 13.1 Create app shell with sidebar navigation
- [x] 13.2 Create GlobalConfig page component
- [x] 13.3 Create Projects page component
- [x] 13.4 Create Settings page component

## 14. Frontend - MCP Management UI

- [x] 14.1 Create MCP server list component (grouped by tool)
- [x] 14.2 Create MCP server detail/edit form
- [x] 14.3 Create copy-to-tool dialog with target selection
- [x] 14.4 Add format conversion warnings display

## 15. Frontend - Skills Management UI

- [x] 15.1 Create Skills list component (grouped by tool)
- [x] 15.2 Create Skill detail view with SKILL.md content
- [x] 15.3 Create copy-to-tool dialog for skills
- [x] 15.4 Add skill validation status indicators

## 16. Frontend - Rules Editor UI

- [x] 16.1 Create Rules file selector (by tool)
- [x] 16.2 Integrate Markdown editor with syntax highlighting
- [x] 16.3 Add save/discard controls
- [x] 16.4 Implement external change conflict dialog

## 17. Frontend - Project Management UI

- [x] 17.1 Create project list with tool badges
- [x] 17.2 Create add project dialog (path input/picker)
- [x] 17.3 Create project config overview panel
- [x] 17.4 Add project removal confirmation

## 18. Integration & Testing

- [x] 18.1 Test MCP read/write for all four tools
- [x] 18.2 Test Skills read/write for all four tools
- [x] 18.3 Test cross-tool copy with format conversion
- [x] 18.4 Test file watcher event propagation
- [x] 18.5 Test project persistence across app restart
