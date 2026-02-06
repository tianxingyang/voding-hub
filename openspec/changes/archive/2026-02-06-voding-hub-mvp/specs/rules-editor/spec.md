## ADDED Requirements

### Requirement: Rules File Display

系统 SHALL 显示各工具的 Rules 文件内容。

#### Scenario: Display global rules
- **WHEN** 用户打开 Rules 编辑器
- **THEN** 显示四个工具的全局 Rules 文件
- **AND** 显示文件路径（如 `~/.claude/CLAUDE.md`）
- **AND** 显示文件最后修改时间

#### Scenario: Display project rules
- **WHEN** 用户选择某个项目
- **THEN** 显示该项目的 Rules 文件（如 `.claude/CLAUDE.md`）
- **AND** 若项目无 Rules 文件则显示"未配置"

---

### Requirement: Rules File Editor

系统 SHALL 提供 Markdown 编辑器编辑 Rules 文件。

#### Scenario: Edit rules content
- **WHEN** 用户在编辑器中修改 Rules 内容
- **THEN** 提供 Markdown 语法高亮
- **AND** 支持实时预览

#### Scenario: Save rules changes
- **WHEN** 用户保存 Rules 修改
- **THEN** 写入对应的 Rules 文件
- **AND** 保持文件编码为 UTF-8

#### Scenario: Handle import syntax
- **WHEN** Rules 文件包含 `@path/file.md` 导入语法
- **THEN** 在编辑器中显示导入语法（不展开）
- **AND** 提供跳转到被导入文件的链接

---

### Requirement: Rules Cross-Tool View

系统 SHALL 支持对比不同工具的 Rules 内容。

#### Scenario: Side-by-side comparison
- **WHEN** 用户选择对比两个工具的 Rules
- **THEN** 并排显示两个 Rules 文件内容
- **AND** 高亮差异部分

---

### Requirement: Rules File Creation

系统 SHALL 支持创建新的 Rules 文件。

#### Scenario: Create project rules
- **WHEN** 项目无 Rules 文件且用户点击"创建"
- **THEN** 在对应位置创建空的 Rules 文件
- **AND** 使用工具特定的文件名（CLAUDE.md/AGENTS.md/GEMINI.md）
