## Context

voding-hub 是一个全新的 Tauri 2.0 桌面应用，用于统一管理四个 AI Coding 工具（Claude Code、Codex、Gemini、OpenCode）的配置。当前项目为空，需从零开始构建。

**约束**：
- 四个工具的配置格式不同：Claude/Gemini/OpenCode 使用 JSON，Codex 使用 TOML
- MCP 配置结构差异：Claude/Gemini 使用 `mcpServers`，OpenCode 使用 `mcp`，Codex 使用 `[mcp_servers.x]`
- Skills 路径差异：Claude 使用 `.claude/skills/`，Codex 使用 `.agents/skills/`，Gemini 使用 `.gemini/skills/`，OpenCode 兼容多路径

## Goals / Non-Goals

**Goals:**
- 统一界面展示四个工具的 MCP、Skills、Rules 配置
- 支持配置的跨工具复制（含格式自动转换）
- 实时监听配置文件变化并更新 UI
- 项目列表持久化存储

**Non-Goals:**
- 自动同步配置（用户手动选择复制目标）
- 配置冲突自动合并（采用最后修改时间优先策略）
- 工具版本检测与适配器版本化（MVP 阶段暂不实现）

## Decisions

### D1: 适配器模式抽象配置差异

**选择**: 使用 Rust trait `ConfigAdapter` 为每个工具实现独立适配器

**理由**:
- 各工具配置格式差异大，适配器模式隔离复杂性
- 新增工具只需实现 trait，不影响核心逻辑

**替代方案**:
- 统一配置模型 + 转换层：复杂度更高，难以处理工具特有字段

### D2: 前后端通信

**选择**: Tauri Commands + Events

**理由**:
- Commands 用于请求/响应（读写配置）
- Events 用于后端主动推送（文件变更通知）
- Tauri 原生支持，性能优秀

### D3: 状态管理

**选择**: Zustand (前端)

**理由**:
- 轻量，API 简洁
- 支持持久化中间件
- 比 Redux 更适合中小型应用

**替代方案**:
- Jotai: 原子化模型更适合细粒度状态，本项目状态结构较固定

### D4: 文件监听策略

**选择**: notify crate + 防抖 (500ms)

**理由**:
- notify 跨平台支持好
- 防抖避免频繁 IO 和 UI 刷新

### D5: 数据库方案

**选择**: SQLite (rusqlite) 存储项目列表

**理由**:
- 仅存储项目元数据，SQLite 足够
- 配置数据直接读写工具原生文件，不入库

## Explicit Constraints

以下约束为实现阶段的硬性要求，消除所有决策点：

### C1: 文件处理策略

| 场景 | 约束 |
|------|------|
| 配置文件不存在 | 视为"需要初始化"，UI 显示"未配置"状态，提供"创建"按钮 |
| 格式保留 | 允许重排，不保留注释，使用标准 serde_json/toml 序列化 |
| 文件编码 | 仅支持 UTF-8，非 UTF-8 文件拒绝读取并显示错误 |
| 写入策略 | 直接写入目标文件，不使用 temp+rename |
| 写入失败 | 不重试，直接报错，保持内存状态不变 |

### C2: 并发与冲突

| 场景 | 约束 |
|------|------|
| 自写抑制 | 维护"正在写入"路径集合，写入期间忽略该路径的 watcher 事件 |
| 外部冲突 | 始终弹窗让用户选择"保留我的修改"或"加载外部修改" |
| 多实例 | 禁止多实例运行，第二个实例启动时显示错误并退出 |
| 并发锁 | 不锁定，依赖"自写抑制"机制 |

### C3: 数据验证

| 字段 | 约束 |
|------|------|
| MCP server name | kebab-case: `^[a-z0-9]+(-[a-z0-9]+)*$`，最长 64 字符 |
| Skill name | kebab-case: `^[a-z0-9]+(-[a-z0-9]+)*$` |
| 项目路径 | 存储用户输入的原始路径（不 canonicalize），重复检测用原始路径比较 |
| 项目名称 | 默认使用目录 basename，允许后续重命名 |
| 时间字段 | Unix epoch 毫秒 (i64)，`created_at` 创建时设置，`updated_at` 每次修改时更新 |

### C4: 跨工具复制

| 场景 | 约束 |
|------|------|
| 同名冲突 | 跳过已存在同名项，显示跳过提示 |
| 无法映射字段 | 丢弃并显示警告（如 headers、timeout、bearer_token_env_var） |
| enabled 字段 | 仅 Codex 支持，其他工具 UI 显示为恒启用状态（不可点击） |

### C5: UI 行为

| 场景 | 约束 |
|------|------|
| 离开保护 | 有未保存修改时阻止切换页面，要求先保存 |
| 解析失败 | 提供 Raw 编辑模式，允许用户手动修复 JSON/TOML 语法 |

### C6: 配置路径（已确认）

| 工具 | 全局配置路径 | 项目配置路径 |
|------|-------------|-------------|
| Claude Code | `~/.claude/` | `.claude/` |
| Codex | `~/.codex/` | `.codex/` |
| Gemini | `~/.gemini/` | `.gemini/` |
| OpenCode | `~/.config/opencode/` | `.opencode/` |

### C7: 日志策略

- 使用 Tauri log plugin
- 日志输出到应用日志目录
- 错误日志包含：文件路径、错误类型、时间戳

## Risks / Trade-offs

| 风险 | 缓解措施 |
|-----|---------|
| 工具配置格式变更导致适配器失效 | 错误处理 + Raw 编辑模式降级 |
| 文件监听性能问题（大量文件） | 仅监听已知配置路径，不递归监听 |
| JSON↔TOML 转换丢失工具特有字段 | 丢弃并显示警告 |
| Skills 复制后权限问题 | 保持原文件权限，可执行脚本保留 +x |

## Property-Based Testing (PBT) Properties

以下属性用于验证系统不变量，使用 Rust `proptest` 框架实现。

### P1: ConfigAdapter Round-trip

```
[INVARIANT] ∀ adapter A, scope S, valid MCP server s:
  write_mcp_server(A, s, S) → s ∈ read_mcp_servers(A, S)
[FALSIFICATION] Generate s with tricky strings (quotes, UTF-8, newlines),
  large env maps; assert via normalized comparison
```

### P2: ConfigAdapter Idempotency

```
[INVARIANT] ∀ A, S, s: write(s); write(s) ≡ write(s)
  (no duplicates by name)
[FALSIFICATION] Attempt repeated writes; verify set(names) unchanged
```

### P3: Name Validation Bounds

```
[INVARIANT] ∀ s where s.name violates ^[a-z0-9]+(-[a-z0-9]+)*$ or |name|>64:
  write_mcp_server MUST reject and leave state unchanged
[FALSIFICATION] Generate "almost valid" names (uppercase, _, leading/trailing -,
  --, empty, 65-200 chars); assert error + no file diff
```

### P4: Format Conversion Round-trip

```
[INVARIANT] parse_json → to_toml → parse_toml preserves servers:
  norm_mcp(P) = norm_mcp(P')
[FALSIFICATION] Generate MCP servers with strings needing escaping,
  empty/non-empty args, env keys with punctuation
```

### P5: Cross-Tool Copy Invariant

```
[INVARIANT] Name-conflict skip: if dest has item name,
  copy(name) is no-op on dest state
[FALSIFICATION] Pre-populate dest with same name but different contents;
  assert dest unchanged + "skipped" indicator
```

### P6: Debounce Correctness

```
[INVARIANT] For each path p, in any burst with max inter-arrival ≤500ms,
  exactly ONE event emitted = LAST event in burst
[FALSIFICATION] Generate timestamped event streams with gaps at 499/500/501ms;
  vary event kinds; assert emitted subsequence matches spec
```

### P7: Self-Write Suppression

```
[INVARIANT] If p in "currently writing" set,
  watcher events for p do not reach frontend
[FALSIFICATION] Generate sequences with begin_write(p), random events, end_write(p);
  assert emits only outside protected interval
```

### P8: Project Persistence Round-trip

```
[INVARIANT] ∀ valid project paths P: add(P) → restart → load() = P
  (raw string equality, no canonicalization)
[FALSIFICATION] Generate temp dirs, store raw input strings
  (including relative, ./x, trailing slash); assert exact string equality
```

### P9: Project Duplicate Detection

```
[INVARIANT] add(p); add(p) results in exactly one record for p
[FALSIFICATION] Generate p and attempt repeated adds;
  assert record count unchanged
```

### P10: Time Field Monotonicity

```
[INVARIANT] created_at set once; updated_at ≥ created_at
[FALSIFICATION] Inject controllable clock; attempt negative/overflowing times;
  ensure rejection or safe handling
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     React Frontend                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │ Global  │  │Projects │  │ Editor  │  │Settings │    │
│  │ Config  │  │  Page   │  │  Page   │  │  Page   │    │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘    │
│       └────────────┴────────────┴────────────┘          │
│                         │                                │
│                   Zustand Store                          │
└─────────────────────────┬───────────────────────────────┘
                          │ Tauri Commands/Events
┌─────────────────────────┴───────────────────────────────┐
│                     Rust Backend                         │
│  ┌─────────────────────────────────────────────────┐    │
│  │                   Commands                       │    │
│  │  config::read_mcp  config::write_skill  ...     │    │
│  └─────────────────────────┬───────────────────────┘    │
│                            │                             │
│  ┌────────────┬────────────┼────────────┬────────────┐  │
│  │   Claude   │   Codex    │   Gemini   │  OpenCode  │  │
│  │  Adapter   │  Adapter   │  Adapter   │  Adapter   │  │
│  └────────────┴────────────┴────────────┴────────────┘  │
│                            │                             │
│  ┌─────────────────────────┴───────────────────────┐    │
│  │              Core (models, converter)            │    │
│  └──────────────────────────────────────────────────┘    │
│                            │                             │
│  ┌──────────────┐    ┌─────┴──────┐                     │
│  │   Watcher    │    │   SQLite   │                     │
│  │   (notify)   │    │    (db)    │                     │
│  └──────────────┘    └────────────┘                     │
└─────────────────────────────────────────────────────────┘
```
