## ADDED Requirements

### Requirement: Project List Management

系统 SHALL 支持管理项目列表。

#### Scenario: Add project by path
- **WHEN** 用户输入或选择项目路径
- **THEN** 验证路径存在且为目录
- **AND** 将项目添加到列表
- **AND** 持久化到 SQLite 数据库

#### Scenario: Remove project
- **WHEN** 用户删除项目
- **THEN** 从列表中移除
- **AND** 从数据库中删除记录
- **AND** 不删除实际项目文件

#### Scenario: Project list persistence
- **WHEN** 应用重启
- **THEN** 从 SQLite 加载已保存的项目列表
- **AND** 验证每个项目路径仍然有效

---

### Requirement: Project Config Detection

系统 SHALL 自动检测项目中存在的工具配置。

#### Scenario: Detect tool configurations
- **WHEN** 添加项目或刷新项目状态
- **THEN** 扫描项目目录下的 `.claude/`, `.codex/`, `.gemini/`, `.opencode/`
- **AND** 记录每个工具的配置存在状态

#### Scenario: Display tool badges
- **WHEN** 显示项目列表
- **THEN** 每个项目显示已检测到的工具图标/徽章

---

### Requirement: Project Config Overview

系统 SHALL 提供项目配置概览。

#### Scenario: Show project config summary
- **WHEN** 用户选择某个项目
- **THEN** 显示该项目的 MCP 服务器数量、Skills 数量、Rules 文件状态
- **AND** 按工具分组展示

#### Scenario: Quick navigation
- **WHEN** 用户点击概览中的配置类型
- **THEN** 跳转到对应的详细视图

---

### Requirement: Project Database Schema

系统 SHALL 使用 SQLite 存储项目元数据。

#### Scenario: Database initialization
- **WHEN** 应用首次启动
- **THEN** 创建 `projects` 表
- **AND** 表结构包含：id, name, path, created_at, updated_at

#### Scenario: Query projects
- **WHEN** 加载项目列表
- **THEN** 按 `updated_at` 降序排列
