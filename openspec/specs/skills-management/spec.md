## ADDED Requirements

### Requirement: Skills List View

系统 SHALL 在统一界面展示所有工具的 Skills 列表。

#### Scenario: Display skills grouped by tool
- **WHEN** 用户打开全局配置页
- **THEN** 按工具分组显示所有 Skills
- **AND** 每个 Skill 显示名称和描述

#### Scenario: Display skill content
- **WHEN** 用户点击某个 Skill
- **THEN** 展示 SKILL.md 完整内容
- **AND** 高亮显示 YAML frontmatter

---

### Requirement: Skills Cross-Tool Copy

系统 SHALL 支持将 Skill 从一个工具复制到另一个工具。

#### Scenario: Copy skill between compatible tools
- **WHEN** 用户选择复制 Claude Code 的 Skill 到 Gemini
- **THEN** 系统复制整个 skill 目录（SKILL.md + 脚本 + 资源）
- **AND** 保持目录结构不变
- **AND** 保持可执行脚本的权限

#### Scenario: Copy skill to OpenCode
- **WHEN** 用户选择复制 Skill 到 OpenCode
- **THEN** 系统复制到 `~/.config/opencode/skills/<name>/`
- **AND** OpenCode 通过兼容路径发现该 Skill

#### Scenario: Copy skill to Codex
- **WHEN** 用户选择复制 Skill 到 Codex
- **THEN** 系统复制到 `~/.agents/skills/<name>/`
- **AND** 遵循 agentskills.io 标准

---

### Requirement: Skills Scope Management

系统 SHALL 支持在全局和项目级别管理 Skills。

#### Scenario: View project-level skills
- **WHEN** 用户选择某个项目
- **THEN** 显示该项目的项目级 Skills（`.claude/skills/` 等）

#### Scenario: Copy skill from global to project
- **WHEN** 用户选择将全局 Skill 复制到项目
- **THEN** 复制到项目的 `.<tool>/skills/` 目录

---

### Requirement: Skill Format Validation

系统 SHALL 验证 SKILL.md 格式正确性。

#### Scenario: Validate YAML frontmatter
- **WHEN** 读取 SKILL.md 文件
- **THEN** 验证存在 `name` 和 `description` 字段
- **AND** `name` 符合 kebab-case 格式（`^[a-z0-9]+(-[a-z0-9]+)*$`）

#### Scenario: Handle invalid skill
- **WHEN** SKILL.md 格式无效
- **THEN** 在 UI 中标记该 Skill 为 "无效"
- **AND** 显示具体错误信息
