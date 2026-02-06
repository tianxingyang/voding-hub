## ADDED Requirements

### Requirement: MCP Server List View

系统 SHALL 在统一界面展示所有工具的 MCP 服务器列表。

#### Scenario: Display MCP servers grouped by tool
- **WHEN** 用户打开全局配置页
- **THEN** 按工具分组显示所有 MCP 服务器
- **AND** 每个服务器显示名称、命令、状态（enabled/disabled）

#### Scenario: Display MCP server details
- **WHEN** 用户点击某个 MCP 服务器
- **THEN** 展示完整配置：command, args, env, url（如适用）

---

### Requirement: MCP Server Edit

系统 SHALL 支持编辑 MCP 服务器配置。

#### Scenario: Edit MCP server fields
- **WHEN** 用户修改 MCP 服务器的 command 或 args
- **THEN** 系统保存变更到对应工具的配置文件
- **AND** 保持原有文件格式（JSON/TOML）

#### Scenario: Toggle MCP server enabled state
- **WHEN** 用户切换 MCP 服务器的启用状态
- **THEN** 更新配置中的 `enabled` 字段

---

### Requirement: MCP Cross-Tool Copy

系统 SHALL 支持将 MCP 服务器配置从一个工具复制到另一个工具。

#### Scenario: Copy MCP from Claude to Codex
- **WHEN** 用户选择复制 Claude Code 的 MCP 到 Codex
- **THEN** 系统自动将 JSON 格式转换为 TOML 格式
- **AND** `mcpServers.x` 转换为 `[mcp_servers.x]`
- **AND** `env` 对象转换为 `[mcp_servers.x.env]` 表

#### Scenario: Copy MCP from Codex to OpenCode
- **WHEN** 用户选择复制 Codex 的 MCP 到 OpenCode
- **THEN** 系统将 TOML 转换为 JSON
- **AND** `command` + `args` 合并为 `command` 数组
- **AND** `env` 转换为 `environment`

#### Scenario: Copy to unsupported destination
- **WHEN** 目标工具不支持某些字段（如 `bearer_token_env_var`）
- **THEN** 系统显示警告提示
- **AND** 仍然完成复制，忽略不支持的字段

---

### Requirement: MCP Field Mapping Matrix

系统 SHALL 按以下映射规则进行跨工具转换。

#### Scenario: Field mapping rules
- **WHEN** 复制 MCP 配置到其他工具
- **THEN** 按以下矩阵映射字段：

| 源字段 | Claude | Codex | Gemini | OpenCode |
|--------|--------|-------|--------|----------|
| command | command | command | command | command[0] |
| args | args | args | args | command[1:] |
| env | env | env | env | environment |
| url | - | url | - | url |
| enabled | - | enabled | - | - |
| type | - | - | - | type |
| headers | - | - | - | headers |

- **AND** `-` 表示该工具不支持此字段
- **AND** 不支持的字段在复制时丢弃并显示警告

#### Scenario: OpenCode command array handling
- **WHEN** 从 OpenCode 复制到其他工具
- **THEN** `command[0]` 映射为 `command`
- **AND** `command[1:]` 映射为 `args`
- **AND** 空数组视为无效配置，拒绝复制

#### Scenario: OpenCode command array creation
- **WHEN** 复制到 OpenCode
- **THEN** 合并 `command` 和 `args` 为 `command` 数组
- **AND** 若 `command` 为空字符串，拒绝复制

---

### Requirement: MCP Format Converter

系统 SHALL 提供 MCP 配置格式转换器。

#### Scenario: JSON to TOML conversion
- **WHEN** 转换 Claude/Gemini/OpenCode 的 MCP 配置到 Codex
- **THEN** 生成有效的 TOML 格式
- **AND** 数组使用 `["item1", "item2"]` 语法
- **AND** 嵌套对象使用 `[section.subsection]` 语法

#### Scenario: TOML to JSON conversion
- **WHEN** 转换 Codex 的 MCP 配置到其他工具
- **THEN** 生成有效的 JSON 格式
- **AND** 保持字段名称映射正确
