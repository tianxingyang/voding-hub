## ADDED Requirements

### Requirement: Unified Config Adapter Trait

系统 SHALL 定义统一的 `ConfigAdapter` trait，为四个 AI Coding 工具提供一致的配置读写接口。

#### Scenario: Trait defines MCP operations
- **WHEN** 适配器实现 `ConfigAdapter` trait
- **THEN** 必须提供 `read_mcp_servers(scope)` 方法返回 `Vec<McpServer>`
- **AND** 必须提供 `write_mcp_server(server, scope)` 方法
- **AND** 必须提供 `delete_mcp_server(name, scope)` 方法

#### Scenario: Trait defines Skills operations
- **WHEN** 适配器实现 `ConfigAdapter` trait
- **THEN** 必须提供 `read_skills(scope)` 方法返回 `Vec<Skill>`
- **AND** 必须提供 `write_skill(skill, scope)` 方法
- **AND** 必须提供 `delete_skill(name, scope)` 方法

#### Scenario: Trait defines Rules operations
- **WHEN** 适配器实现 `ConfigAdapter` trait
- **THEN** 必须提供 `read_rules(scope)` 方法返回 Rules 文件内容
- **AND** 必须提供 `write_rules(content, scope)` 方法

---

### Requirement: Claude Code Adapter

系统 SHALL 实现 Claude Code 的配置适配器。

#### Scenario: Read MCP from global config
- **WHEN** 调用 `read_mcp_servers(Global)`
- **THEN** 从 `~/.claude/.mcp.json` 读取 `mcpServers` 对象
- **AND** 解析每个 server 的 `command`, `args`, `env` 字段

#### Scenario: Read MCP from project config
- **WHEN** 调用 `read_mcp_servers(Project(path))`
- **THEN** 从 `{path}/.mcp.json` 读取配置

#### Scenario: Read Skills
- **WHEN** 调用 `read_skills(Global)`
- **THEN** 扫描 `~/.claude/skills/*/SKILL.md`
- **AND** 解析 YAML frontmatter 获取 `name` 和 `description`

#### Scenario: Read Rules
- **WHEN** 调用 `read_rules(Global)`
- **THEN** 返回 `~/.claude/CLAUDE.md` 内容

---

### Requirement: Codex Adapter

系统 SHALL 实现 Codex 的配置适配器。

#### Scenario: Read MCP from TOML config
- **WHEN** 调用 `read_mcp_servers(Global)`
- **THEN** 从 `~/.codex/config.toml` 读取 `[mcp_servers.*]` 表
- **AND** 解析 `command`, `args`, `env`, `url`, `enabled` 字段

#### Scenario: Read Skills from .agents path
- **WHEN** 调用 `read_skills(Global)`
- **THEN** 扫描 `~/.agents/skills/*/SKILL.md`

#### Scenario: Read Rules
- **WHEN** 调用 `read_rules(Global)`
- **THEN** 返回 `~/.codex/AGENTS.md` 内容

---

### Requirement: Gemini Adapter

系统 SHALL 实现 Gemini 的配置适配器。

#### Scenario: Read MCP from settings.json
- **WHEN** 调用 `read_mcp_servers(Global)`
- **THEN** 从 `~/.gemini/settings.json` 读取 `mcpServers` 对象

#### Scenario: Read Skills
- **WHEN** 调用 `read_skills(Global)`
- **THEN** 扫描 `~/.gemini/skills/*/SKILL.md`

#### Scenario: Read Rules
- **WHEN** 调用 `read_rules(Global)`
- **THEN** 返回 `~/.gemini/GEMINI.md` 内容

---

### Requirement: OpenCode Adapter

系统 SHALL 实现 OpenCode 的配置适配器。

#### Scenario: Read MCP from opencode.json
- **WHEN** 调用 `read_mcp_servers(Global)`
- **THEN** 从 `~/.config/opencode/opencode.json` 读取 `mcp` 对象
- **AND** 解析 `type`, `command`, `environment`, `url`, `headers` 字段

#### Scenario: Read Skills with fallback paths
- **WHEN** 调用 `read_skills(Global)`
- **THEN** 按优先级扫描：`~/.config/opencode/skills/`, `~/.claude/skills/`, `~/.agents/skills/`

#### Scenario: Read Rules with Claude compatibility
- **WHEN** 调用 `read_rules(Global)`
- **THEN** 优先返回 `~/.config/opencode/AGENTS.md`
- **AND** 若不存在则尝试 `~/.claude/CLAUDE.md`
