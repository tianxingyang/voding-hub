## Why

用户使用多个 AI Coding 工具（Claude Code、Codex、Gemini、OpenCode），面临配置分散、同步困难的痛点。发现好用的 MCP 服务器或 Skill 后，需要手动在每个工具中重复配置，效率低下。

## What Changes

- 新增 Tauri 2.0 桌面应用，提供统一的配置管理界面
- 实现四个工具的配置适配器（Claude Code、Codex、Gemini、OpenCode）
- 支持 MCP 服务器配置的跨工具复制（含 JSON↔TOML 格式转换）
- 支持 Skills 的跨工具复制
- 支持 Rules 文件的统一编辑
- 实现文件系统监听，实时同步外部配置修改
- 使用 SQLite 持久化项目列表

## Capabilities

### New Capabilities

- `config-adapters`: 四个 AI Coding 工具的配置读写适配器，支持 MCP、Skills、Rules 的统一抽象
- `mcp-management`: MCP 服务器配置的查看、编辑、跨工具复制，含格式转换（JSON↔TOML）
- `skills-management`: Skills 的查看、编辑、跨工具复制
- `rules-editor`: Rules 文件（CLAUDE.md/AGENTS.md/GEMINI.md）的统一编辑器
- `project-management`: 项目添加、删除、配置状态检测，SQLite 持久化
- `file-watcher`: 配置文件变更监听，实时更新 UI

### Modified Capabilities

（无现有 capabilities 需要修改）

## Impact

- **新增代码**: Tauri Rust 后端 + React TypeScript 前端
- **依赖**: Tauri 2.0, rusqlite, notify crate, serde, toml
- **文件系统**: 读写 `~/.claude/`, `~/.codex/`, `~/.gemini/`, `~/.config/opencode/` 及项目级配置目录
