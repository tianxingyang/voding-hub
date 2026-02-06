# voding-hub: AI Coding 工具配置管理器

## 1. 需求概述

### 1.1 核心问题

用户使用多个 AI Coding 工具（Claude Code、Codex、Gemini、OpenCode），面临以下痛点：

1. **配置分散** - 每个工具有独立的全局配置和项目级配置，分布在不同路径
2. **同步困难** - 发现好用的 MCP 服务器或 Skill 后，需要手动在每个工具中重复配置
3. **管理混乱** - 缺乏统一视图查看和管理所有工具的配置状态

### 1.2 解决方案

开发 **voding-hub**，一个桌面应用程序，提供：

- 统一界面查看和编辑所有 AI Coding 工具的配置
- 文件监听实时同步外部修改
- 配置复制功能，支持跨工具、跨项目复制 MCP/Skill
- Rules 文件在线编辑器

---

## 2. 约束集合

### 2.1 技术栈约束（已确定）

| 层级 | 技术选型 | 约束说明 |
|------|---------|---------|
| 应用框架 | Tauri 2.0 | 跨平台桌面应用 |
| 前端 | React + TypeScript + Vite | SPA 架构 |
| 存储 | SQLite (rusqlite) | 本地数据库 |
| 文件监听 | notify crate (Rust) | 跨平台文件系统事件 |

### 2.2 配置路径约束（已确定）

| 工具 | 全局配置路径 | 项目配置路径 | 配置格式 |
|------|-------------|-------------|---------|
| Claude Code | `~/.claude/` | `.claude/` | JSON |
| Codex | `~/.codex/` | `.codex/` | TOML |
| Gemini | `~/.gemini/` | `.gemini/` | JSON |
| OpenCode | `~/.opencode/` | `.opencode/` | JSON |

### 2.3 配置类型约束

#### MCP 服务器配置

| 工具 | 配置位置 | 格式 | 关键字段 |
|------|---------|------|---------|
| Claude Code | `~/.claude/.mcp.json` 或项目级 `.mcp.json` | JSON `mcpServers` 对象 | `command`, `args`, `env` |
| Codex | `~/.codex/config.toml` 内 `[mcp_servers.<name>]` | TOML table | `command`, `args`, `env`, `url`, `enabled`, `timeout` |
| Gemini | `~/.gemini/settings.json` 或 `.gemini/settings.json` 内 `mcpServers` | JSON `mcpServers` 对象 |
| OpenCode | `~/.config/opencode/opencode.json` 或项目 `opencode.json` 内 `mcp` | JSON `mcp` 对象 |

**Claude Code MCP 格式示例**：
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@package/name"],
      "env": { "API_KEY": "${TOKEN}" }
    }
  }
}
```

**Codex MCP 格式示例**：
```toml
[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]

[mcp_servers.context7.env]
MY_ENV_VAR = "value"
```

**Gemini MCP 格式示例**：
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@package/name"]
    }
  }
}
```

**OpenCode MCP 格式示例**：
```json
{
  "mcp": {
    "server-name": {
      "type": "local",
      "command": ["npx", "-y", "@package/name"],
      "environment": { "API_KEY": "value" }
    }
  }
}
```

#### Skills 配置

| 工具 | 配置位置 | 格式 | 支持状态 |
|------|---------|-----|---------|
| Claude Code | `~/.claude/skills/<name>/SKILL.md` | YAML frontmatter (`name`, `description`) + Markdown | ✅ 完整支持 |
| Codex | `~/.agents/skills/<name>/SKILL.md` 或 `.agents/skills/` | YAML frontmatter (`name`, `description`) + Markdown，遵循 agentskills.io 标准 | ✅ 完整支持 |
| Gemini | `~/.gemini/skills/<name>/SKILL.md` 或 `.gemini/skills/` | YAML frontmatter (`name`, `description`) + Markdown | ✅ 完整支持 |
| OpenCode | `~/.config/opencode/skills/<name>/SKILL.md` 或 `.opencode/skills/` | YAML frontmatter (`name`, `description`) + Markdown，兼容 `.claude/skills/` | ✅ 完整支持 |

**Codex Skills 搜索路径**（按优先级）：
1. `$CWD/.agents/skills/` - 当前工作目录
2. `$REPO_ROOT/.agents/skills/` - 仓库根目录
3. `$HOME/.agents/skills/` - 用户全局目录
4. `/etc/codex/skills/` - 系统管理员目录

**OpenCode Skills 搜索路径**（按优先级）：
1. `.opencode/skills/<name>/SKILL.md` - 项目级
2. `~/.config/opencode/skills/<name>/SKILL.md` - 全局
3. `.claude/skills/<name>/SKILL.md` - Claude 兼容路径
4. `.agents/skills/<name>/SKILL.md` - Agent 兼容路径

**Claude Code / Gemini SKILL.md 格式**：
```markdown
---
name: skill-identifier
description: Brief description of what this skill does
---

# Skill Title

## Instructions
Step-by-step guidance...
```

#### Rules 文件

| 工具 | 全局 Rules | 项目 Rules | 格式特性 |
|------|-----------|-----------|---------|
| Claude Code | `~/.claude/CLAUDE.md` | `.claude/CLAUDE.md` 或项目根 `CLAUDE.md` | 支持 `@path/file.md` 导入语法 |
| Codex | `~/.codex/AGENTS.md` | `.codex/AGENTS.md` 或项目根 `AGENTS.md` | 标准 Markdown |
| Gemini | `~/.gemini/GEMINI.md` | `.gemini/GEMINI.md` 或项目根 `GEMINI.md` | 支持 `@file.md` 导入语法，层级合并 |
| OpenCode | `~/.config/opencode/AGENTS.md` | 项目根 `AGENTS.md` | 兼容 Claude Code 的 `CLAUDE.md`，支持 `opencode.json` 中 `instructions` 字段引用多文件 |

### 2.4 行为约束

| 约束项 | 约束内容 |
|-------|---------|
| 核心定位 | 配置编辑器 + 实时同步，非自动同步工具 |
| 同步方式 | 用户手动选择配置项复制到目标位置 |
| 冲突策略 | 最后修改时间优先 |
| 项目发现 | 用户手动添加项目路径 |
| 实时同步 | 文件系统监听（notify crate） |
| 复制粒度 | MCP 服务器（整体）、Skill（整体）、Rules（可编辑） |

---

## 3. 功能规格

### 3.1 全局配置管理

**功能描述**：统一视图展示和编辑所有工具的全局配置

**子功能**：
- 3.1.1 MCP 服务器列表（按工具分组）
- 3.1.2 Skills 列表（按工具分组）
- 3.1.3 Rules 文件编辑器
- 3.1.4 其他全局设置查看

**验收标准**：
- [ ] 能正确读取 `~/.claude/`、`~/.codex/`、`~/.gemini/`、`~/.opencode/` 下的配置
- [ ] 配置修改后能正确写回对应格式的文件
- [ ] 文件监听能检测外部修改并更新 UI

### 3.2 项目管理

**功能描述**：管理用户添加的项目及其配置

**子功能**：
- 3.2.1 添加/删除项目
- 3.2.2 项目配置概览（显示各工具在该项目的配置状态）
- 3.2.3 项目级 MCP/Skills/Rules 管理

**验收标准**：
- [ ] 能手动添加项目路径
- [ ] 能检测项目中存在的 `.claude/`、`.codex/` 等配置目录
- [ ] 项目列表持久化存储在 SQLite

### 3.3 配置复制

**功能描述**：将配置从一个位置复制到另一个位置

**子功能**：
- 3.3.1 MCP 服务器复制（全局→项目、项目→项目、工具→工具）
- 3.3.2 Skill 复制（全局→项目、项目→项目、工具→工具）
- 3.3.3 格式自动转换（JSON↔TOML）

**验收标准**：
- [ ] 复制 MCP 时自动转换配置格式（如 Claude JSON → Codex TOML）
- [ ] 复制 Skill 时处理工具间的格式差异
- [ ] 复制到不支持该功能的工具时显示警告

### 3.4 实时同步

**功能描述**：监听配置文件变化，实时更新 UI

**子功能**：
- 3.4.1 全局配置目录监听
- 3.4.2 已添加项目的配置目录监听
- 3.4.3 变更通知和 UI 刷新

**验收标准**：
- [ ] 外部修改配置文件后，UI 在 1 秒内更新
- [ ] 监听不影响应用性能
- [ ] 支持 Linux/macOS/Windows

---

## 4. 架构设计

### 4.1 目录结构

```
voding-hub/
├── src/                          # React 前端
│   ├── components/
│   │   ├── ConfigList/           # 配置列表组件
│   │   ├── Editor/               # Rules 编辑器
│   │   └── ProjectSelector/      # 项目选择器
│   ├── pages/
│   │   ├── GlobalConfig/         # 全局配置页
│   │   ├── Projects/             # 项目管理页
│   │   └── Settings/             # 应用设置页
│   ├── hooks/
│   │   └── useConfigSync.ts      # 配置同步 hook
│   ├── stores/
│   │   └── configStore.ts        # 状态管理
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/             # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── config.rs         # 配置读写命令
│   │   │   ├── project.rs        # 项目管理命令
│   │   │   └── watcher.rs        # 文件监听命令
│   │   ├── adapters/             # 工具配置适配器
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs         # 适配器 trait 定义
│   │   │   ├── claude.rs
│   │   │   ├── codex.rs
│   │   │   ├── gemini.rs
│   │   │   └── opencode.rs
│   │   ├── core/                 # 核心业务逻辑
│   │   │   ├── mod.rs
│   │   │   ├── config.rs         # 配置模型
│   │   │   ├── converter.rs      # 格式转换
│   │   │   └── watcher.rs        # 文件监听逻辑
│   │   └── db/                   # 数据库层
│   │       ├── mod.rs
│   │       └── schema.rs
│   └── Cargo.toml
├── package.json
└── vite.config.ts
```

### 4.2 核心数据模型

```rust
// 统一的 MCP 服务器配置模型
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

// 统一的 Skill 配置模型
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,  // Markdown 内容
    pub scripts: Vec<PathBuf>,
}

// 项目模型
pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: PathBuf,
    pub tools: Vec<ToolType>,  // 检测到的工具配置
}

// 工具类型枚举
pub enum ToolType {
    ClaudeCode,
    Codex,
    Gemini,
    OpenCode,
}
```

### 4.3 适配器模式

```rust
pub trait ConfigAdapter {
    fn tool_type(&self) -> ToolType;
    fn global_config_path(&self) -> PathBuf;
    fn project_config_path(&self, project: &Path) -> PathBuf;

    // MCP 操作
    fn read_mcp_servers(&self, scope: ConfigScope) -> Result<Vec<McpServer>>;
    fn write_mcp_server(&self, server: &McpServer, scope: ConfigScope) -> Result<()>;
    fn delete_mcp_server(&self, name: &str, scope: ConfigScope) -> Result<()>;

    // Skill 操作
    fn read_skills(&self, scope: ConfigScope) -> Result<Vec<Skill>>;
    fn write_skill(&self, skill: &Skill, scope: ConfigScope) -> Result<()>;
    fn delete_skill(&self, name: &str, scope: ConfigScope) -> Result<()>;

    // Rules 操作
    fn read_rules(&self, scope: ConfigScope) -> Result<String>;
    fn write_rules(&self, content: &str, scope: ConfigScope) -> Result<()>;
}

pub enum ConfigScope {
    Global,
    Project(PathBuf),
}
```

---

## 5. 实现计划

### Phase 1: 基础框架

- [ ] 初始化 Tauri 2.0 + React + TypeScript 项目
- [ ] 实现 SQLite 数据库层（项目存储）
- [ ] 实现 4 个工具的 ConfigAdapter trait
- [ ] 基础 UI 框架（导航、布局）

### Phase 2: 全局配置管理

- [ ] 全局 MCP 服务器列表读取和展示
- [ ] 全局 Skills 列表读取和展示
- [ ] Rules 文件读取和编辑器
- [ ] 配置写入功能

### Phase 3: 项目管理

- [ ] 项目添加/删除功能
- [ ] 项目配置检测和展示
- [ ] 项目级配置管理

### Phase 4: 配置复制

- [ ] MCP 服务器复制（含格式转换）
- [ ] Skill 复制
- [ ] 复制目标选择 UI

### Phase 5: 实时同步

- [ ] 文件监听实现（notify crate）
- [ ] 变更事件处理和 UI 更新
- [ ] 性能优化

---

## 6. 成功判据

| 判据 | 验证方式 |
|-----|---------|
| 能正确读取 4 个工具的全局配置 | 手动验证配置内容与实际文件一致 |
| 能正确写入配置且格式正确 | 修改后工具能正常读取配置 |
| MCP 复制后格式转换正确 | Claude JSON → Codex TOML 转换后可用 |
| 文件监听响应及时 | 外部修改后 UI 在 1 秒内更新 |
| 项目管理持久化 | 重启应用后项目列表保留 |

---

## 7. 风险与待确认项

### 7.1 已确认项 ✅

- [x] **Gemini CLI MCP 配置** - 支持，位于 `settings.json` 内 `mcpServers` 对象，格式与 Claude Code 类似（来源：[官方文档](https://geminicli.com/docs)）
- [x] **Gemini CLI Rules 文件** - 支持 `GEMINI.md`，支持层级合并和 `@file.md` 导入语法（来源：[官方文档](https://geminicli.com/docs/cli/gemini-md)）
- [x] **Gemini CLI Skills** - 支持，格式与 Claude Code 一致（YAML frontmatter + Markdown）（来源：[官方文档](https://geminicli.com/docs/cli/skills)）
- [x] **OpenCode MCP 配置** - 支持，位于 `opencode.json` 内 `mcp` 对象，支持 local/remote 类型（来源：[官方文档](https://opencode.ai/docs/mcp-servers)）
- [x] **OpenCode Rules 文件** - 支持 `AGENTS.md`，兼容 Claude Code 的 `CLAUDE.md`（来源：[官方文档](https://opencode.ai/docs/rules)）
- [x] **OpenCode Skills** - 支持，格式与 Claude Code 一致，且兼容 `.claude/skills/` 路径（来源：[官方文档](https://opencode.ai/docs/skills)）
- [x] **Codex Skills** - 支持，使用 `.agents/skills/` 目录，遵循 agentskills.io 开放标准（来源：[官方文档](https://developers.openai.com/codex/skills)）

### 7.2 风险

| 风险 | 影响 | 缓解措施 |
|-----|-----|---------|
| 工具配置格式变更 | 适配器失效 | 版本检测 + 适配器版本化 |
| 文件监听性能问题 | 应用卡顿 | 防抖 + 批量处理 |
| 格式转换丢失信息 | 配置不完整 | 保留原始字段 + 警告提示 |
