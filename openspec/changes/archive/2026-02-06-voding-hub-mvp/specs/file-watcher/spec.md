## ADDED Requirements

### Requirement: Config File Watching

系统 SHALL 监听配置文件变化。

#### Scenario: Watch global config directories
- **WHEN** 应用启动
- **THEN** 监听 `~/.claude/`, `~/.codex/`, `~/.gemini/`, `~/.config/opencode/`
- **AND** 监听关键配置文件（.mcp.json, config.toml, settings.json, opencode.json）

#### Scenario: Watch project config directories
- **WHEN** 项目被添加到列表
- **THEN** 开始监听该项目的配置目录
- **AND** 项目被移除时停止监听

---

### Requirement: Change Event Processing

系统 SHALL 处理文件变更事件。

#### Scenario: Debounce rapid changes
- **WHEN** 同一文件在 500ms 内多次变更
- **THEN** 仅处理最后一次变更
- **AND** 避免频繁 IO 和 UI 刷新

#### Scenario: Emit change events
- **WHEN** 检测到配置文件变更
- **THEN** 通过 Tauri Event 通知前端
- **AND** 事件包含：文件路径、变更类型（create/modify/delete）、所属工具

---

### Requirement: UI Real-time Update

系统 SHALL 在配置变更时实时更新 UI。

#### Scenario: Refresh config view
- **WHEN** 前端收到配置变更事件
- **THEN** 重新加载对应工具/项目的配置
- **AND** 更新显示内容
- **AND** 若用户正在编辑该配置，显示"外部修改"提示

#### Scenario: Conflict notification
- **WHEN** 用户正在编辑的文件被外部修改
- **THEN** 显示冲突提示
- **AND** 提供"保留我的修改"和"加载外部修改"选项

---

### Requirement: Watcher Performance

系统 SHALL 确保文件监听不影响性能。

#### Scenario: Limit watch scope
- **WHEN** 设置文件监听
- **THEN** 仅监听已知配置文件，不递归监听整个目录

#### Scenario: Graceful degradation
- **WHEN** 监听失败（权限问题等）
- **THEN** 记录错误日志
- **AND** 继续运行，不监听该路径
- **AND** 在 UI 显示警告
