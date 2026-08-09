# w-ssh 项目指南

本文件适用于仓库全目录；同时遵循上级全局 `AGENTS.md`。若后续子目录出现更具体的 `AGENTS.md`，以更具体的约定为准。

## 项目概览

- w-ssh 是本地优先的跨平台 SSH 会话管理器。
- 前端位于 `src/`，使用 Vue 3、TypeScript、Pinia、Naive UI 和 xterm.js。
- 桌面后端位于 `src-tauri/`，使用 Tauri 2、Tokio、russh 和 SQLite/sqlx。
- 会话数据库在运行时创建于 `{app_data_dir}/sessions.db`；项目没有云端服务。

## 目录与职责

- `src/components/`：界面和终端组件。
- `src/stores/`：会话与终端状态，以及前端到 Tauri 的 IPC 调用。
- `src/types/`：前端共享类型。
- `src-tauri/src/commands.rs`：Tauri 命令薄层。
- `src-tauri/src/db.rs`：SQLite 初始化与会话 CRUD。
- `src-tauri/src/ssh.rs`：SSH 认证、PTY、收发与终端生命周期。
- `src-tauri/src/lib.rs`：应用状态、插件及命令注册。
- `docs/`：架构、开发和用户文档。
- `.github/workflows/build.yml`：`v*` tag 触发的多平台发布构建。
- `dist/`、`node_modules/`、`src-tauri/target/`、`src-tauri/gen/` 是生成内容，不手工修改。

## 常用命令

在仓库根目录运行：

```powershell
npm ci
npm run tauri dev
npm run build
npm run tauri build
```

Rust 最小验证：

```powershell
Push-Location src-tauri
cargo check
cargo test
cargo clippy
Pop-Location
```

项目目前没有独立的前端测试或 lint 脚本。前端行为变化至少运行 `npm run build`；Rust 行为变化至少运行 `cargo test` 或 `cargo check`，按改动风险补充更完整验证。

## 实现约定

- 沿用 Vue 单文件组件和 `<script setup lang="ts">`；共享状态放入 Pinia store，避免组件间复制 IPC 状态。
- 新增 Tauri 命令时，在 `commands.rs` 实现，并在 `lib.rs` 的 `generate_handler!` 中注册；前端 `invoke()` 的命令名和参数必须同步。
- 后端主动推送的数据使用 Tauri event；终端事件沿用 `ssh_data_{terminal_id}` 和 `ssh_closed_{terminal_id}` 命名。
- `terminal_id` 是前端 Tab、后端 `TerminalMap` 和事件名之间的关联键，不要为同一连接派生第二套标识。
- `commands.rs` 维持薄层，数据库逻辑放 `db.rs`，SSH 生命周期和协议逻辑放 `ssh.rs`。
- 数据库结构当前由 `init_db()` 中的 SQL 在启动时维护。修改字段时，同时检查 Rust model、CRUD 查询、前端类型、表单与已有数据库兼容性。
- 保持现有 UTF-8 文件和局部代码风格；只格式化本次改动涉及的范围，避免无关的大规模格式变化。
- 修改行为或发布方式后，同步检查 `README.md`、`CLAUDE.md`、`docs/` 和工作流说明，避免版本号、触发条件或命令描述相互矛盾。

## 安全与真实环境边界

- 不使用真实主机、账号、密码或私钥做测试，除非用户明确授权目标和操作范围。
- 不输出或提交 SSH 凭据、私钥内容、运行时数据库及本机敏感路径。
- 当前 SSH 服务端密钥检查会接受任意主机密钥。不要把现状描述为已验证；涉及 known-hosts、指纹确认或密钥轮换时，应作为显式安全设计处理。
- 会话认证信息当前保存在本地 SQLite。涉及加密、迁移或兼容性时，先说明数据影响并设计可恢复的迁移路径。
- 自动化验证不得默认建立外部 SSH 连接；优先使用单元测试、可控 fixture 或明确的本地测试服务。

## 提交前检查

- 确认 `git diff` 只包含任务范围内的修改，并保留用户已有改动。
- 检查新增 Tauri 命令是否已注册、前后端参数是否一致、事件监听是否在卸载时清理。
- 检查终端关闭、通道关闭和连接失败路径，避免残留 Tab、任务或 `TerminalMap` 条目。
- 运行覆盖改动的最小验证，并在无法运行时说明具体原因和残余风险。
