# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

w-ssh 是一个跨平台 SSH 会话管理器，基于 **Tauri 2**（Rust 后端 + WebView 前端）构建。前端使用 Vue 3 + TypeScript + Naive UI，终端模拟使用 xterm.js，SSH 协议实现使用 russh。会话默认保存在本地 SQLite，也可显式选择严格 YAML 后端。

## 开发命令

```bash
# 安装前端依赖
npm install

# 启动开发模式（同时启动 Vite dev server 和 Tauri 窗口）
npm run tauri dev

# 生产构建（输出在 src-tauri/target/release/bundle/）
npm run tauri build

# 仅前端 TypeScript 类型检查 + Vite 构建
npm run build

# Rust 单独编译检查（不需要 Tauri 全量构建）
cd src-tauri && cargo check

# Rust 单元测试
cd src-tauri && cargo test
```

## 架构总览

### Tauri IPC 模式

前端与 Rust 后端通过两种机制通信：

- **`invoke(command, args)`** — 前端调用 Rust 命令（请求/响应模式）
- **`listen(event, handler)`** — 前端监听 Rust 主动推送的事件（流式数据）

所有 Tauri 命令在 `src-tauri/src/commands.rs` 注册，并在 `lib.rs` 的 `invoke_handler![]` 宏中声明。

### SSH 连接生命周期

```
前端 terminalsStore.openTerminal()
  → invoke('ssh_connect', { sessionId, cols, rows, password? })
    → Rust: 从当前存储读 session → 建立 russh 连接 → 生成 terminal_id (UUID)
    → 创建 mpsc 通道 (write_tx/resize_tx) → 存入 TerminalMap (DashMap)
    → 后台 Tokio select! 循环（SSH 输出 → emit('ssh_data_{id}') | 写入 | resize）
  ← 返回 terminal_id
前端: TerminalPanel.vue 监听 ssh_data_{terminal_id} 事件，写入 xterm.js
```

**`terminal_id`（UUID）** 是贯穿前后端的唯一标识：
- 前端 `TerminalTab.id` = `terminalsStore.activeTabId` = Tauri 事件名中的 ID
- 后端 `TerminalMap` 的键 = `TerminalHandle`（含 `write_tx` / `resize_tx`）

### 状态管理（Pinia）

| Store | 职责 |
|-------|------|
| `useSessionsStore` | SSH 会话 CRUD，`groupedSessions` 按 `group_name` 聚合 |
| `useTerminalsStore` | 打开的终端标签列表、活跃 Tab ID，桥接 invoke 调用 |
| `useStorageStore` | 当前后端/路径状态、显式切换及复制并切换 |

### Rust 后端模块

| 文件 | 职责 |
|------|------|
| `lib.rs` | AppState（StorageManager + TerminalMap）初始化，Tauri builder |
| `commands.rs` | 所有 Tauri 命令的薄层胶水代码，委托给存储/ssh 模块 |
| `ssh.rs` | SSH 连接逻辑、TerminalMap 类型定义、Tokio 事件循环 |
| `db.rs` | SQLite 表结构 + CRUD，数据库文件在 `{app_data_dir}/sessions.db` |
| `storage.rs` | SQLite/YAML 共用的异步会话存储接口 |
| `storage_manager.rs` | 后端选择、原子设置、显式复制校验与切换 |
| `yaml_storage.rs` | 严格 YAML schema、安全校验、恢复副本与原子写入 |
| `models.rs` | Serde 可序列化的数据结构（Session、CreateSession 等） |

### 前端组件职责

- **`App.vue`** — 顶部 Tab 栏布局；监听 `terminalsStore.activeTabId` 自动切换焦点 Tab
- **`VaultsView.vue`** — 主机管理视图，含左侧导航（All/Group/占位项）+ 卡片网格
- **`HostCard.vue`** — 单主机卡片，双击连接，右键菜单（连接/编辑/删除）
- **`TerminalPanel.vue`** — xterm.js 容器，处理全部 SSH 数据收发和窗口 resize
- **`SessionForm.vue`** — 新建/编辑 Session 的 NModal 表单，含私钥路径自动扫描
- **`StorageSettings.vue`** — 后端/路径设置和显式复制并切换

## 关键实现细节

**SSH 服务器密钥验证**：当前 `check_server_key()` 始终返回 `Ok(true)`（MVP 阶段，无主机指纹校验）。

**TerminalMap**：`Arc<DashMap<String, TerminalHandle>>`，支持多 SSH 连接并发安全访问。`ssh_disconnect` 命令仅从 map 中 remove，后台 Tokio 任务检测到 channel 关闭后自动发出 `ssh_closed_{id}` 事件。

**content-area 布局**：`.content-area > *` 使用 `position: absolute; inset: 0`，配合 `v-show` 实现 VaultsView 与多个 TerminalPanel 互不干扰的切换，保留各 xterm 实例状态。

**私钥扫描**：`get_ssh_key_paths` 命令扫描 `~/.ssh/` 下的标准私钥文件名（`id_rsa`、`id_ed25519` 等），兼容 Windows（USERPROFILE）和 Unix（HOME）。

**存储边界**：没有 `storage-settings.json` 时继续使用 `{app_data_dir}/sessions.db`。YAML 只保存非敏感连接资料和私钥文件路径；密码在连接时一次性传入。复制只允许空目标，校验成功后切换，且不删除源数据。

**sqlx 查询**：`db.rs` 使用运行时 `sqlx::query` / `query_scalar`，本项目当前不依赖 `DATABASE_URL` 或 `.sqlx/` 离线缓存完成编译。

## CI/CD

`.github/workflows/build.yml` 配置为推送 `v*` tag 时构建 Windows（.msi/.exe）、macOS（.dmg）、Linux（.deb/.rpm/.AppImage）安装包并创建 GitHub Release。本地没有验证该工作流的实际运行或发布结果。
