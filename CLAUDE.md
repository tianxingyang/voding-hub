# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

voding-hub 是一个 AI Coding 工具配置管理器，用于统一管理多个 AI coding 工具的配置：
- **支持的工具**: Claude Code, Codex, Gemini, OpenCode
- **管理范围**: Rules 文件、MCP 服务器、Skills、全局配置、项目级配置

## Tech Stack

- **App**: Tauri 2.0
- **Frontend**: React + TypeScript + Vite
- **Storage**: SQLite (via rusqlite)

## Development

```bash
pnpm install                # 安装依赖
pnpm tauri dev              # 开发模式
pnpm tauri build            # 构建应用
cargo test --manifest-path src-tauri/Cargo.toml  # 测试
```

## Architecture

```
voding-hub/
├── src/                    # React 前端
│   ├── components/
│   ├── pages/
│   └── main.tsx
├── src-tauri/              # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/       # Tauri Commands
│   │   ├── adapters/       # 各工具配置适配器
│   │   │   ├── claude.rs
│   │   │   ├── codex.rs
│   │   │   ├── gemini.rs
│   │   │   └── opencode.rs
│   │   ├── core/           # 核心业务逻辑
│   │   └── db/             # 数据库层
│   └── Cargo.toml
├── package.json
└── vite.config.ts
```

## Config Paths Reference

| Tool | Global Config | Project Config |
|------|--------------|----------------|
| Claude Code | `~/.claude/` | `.claude/` |
| Codex | `~/.codex/` | `.codex/` |
| Gemini | `~/.gemini/` | `.gemini/` |
| OpenCode | `~/.opencode/` | `.opencode/` |
