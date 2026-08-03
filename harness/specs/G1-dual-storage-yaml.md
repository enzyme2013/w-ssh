# Spec: G1 SQLite + YAML 双存储后端

Created: 2026-08-03
Status: accepted

## Decision

- 选择 YAML 作为文本配置格式；该格式决策已由用户于 2026-08-03 确认。
- 原因：此配置面向用户直接阅读和维护，YAML 比 JSON 更紧凑，层级结构更自然，并允许用户添加说明性注释。
- SQLite 继续作为默认后端；YAML 是用户显式选择的替代后端，而不是自动导出副本。
- YAML 后端只持久化连接配置，不默认写入密码或私钥内容。密码认证会保存认证类型，但密码需在连接时临时提供；持久化敏感凭据留给独立安全存储 Goal。

## YAML Profile

- 文件扩展名使用 `.yml`，默认建议路径为 `{app_data_dir}/sessions.yml`。
- 文档使用 YAML 1.2 的安全基础子集：mapping、sequence、string、integer、boolean 和 null。
- v1 不允许自定义 tag、anchor、alias 或 merge key，避免隐式合并和跨解析器差异。
- `schema_version`、`port` 等结构字段严格类型校验；主机、用户名、ID 和路径始终按字符串处理。
- 解析器必须安全加载且不得实例化任意类型。
- 用户注释允许被读取。v1 不承诺 CRUD 后原位保留任意注释或手工排版；应用写回采用稳定格式规范化。
- 规范化已有文件前，应用必须在同目录创建包含原始字节的唯一恢复副本并向用户返回其路径；恢复副本创建失败时拒绝写回。该副本不参与后端自动选择，也不被后续写入覆盖或自动删除。

## Scope

- 定义统一的会话存储接口，覆盖 list/get/create/update/delete。
- 保留现有 SQLite 实现，并增加 YAML 文件实现。
- 增加带 `schema_version` 的 YAML 文档格式和严格校验。
- 支持应用级后端选择：`sqlite` 或 `yaml`；SQLite 为默认值。
- 支持用户选择 YAML 文件路径。
- YAML 写入采用同目录临时文件、flush 和原子替换；写入失败时保留最后一个有效文件。
- 对缺失文件创建空配置；对语法错误、未知 schema 版本、重复 ID 和无效字段返回可行动错误，不静默清空。
- 提供显式的 SQLite ↔ YAML 复制/迁移；源数据在验证成功前不得修改或删除。
- “迁移”在 G1 中实现为复制并切换：先读取并严格校验全部源记录，再写入目标、重新打开目标并逐字段核验，最后才允许用户显式切换当前后端；失败时保持当前后端和源数据不变。
- 目标包含记录时默认拒绝复制，避免隐式合并或覆盖。G1 不提供删除源数据或强制覆盖入口；回退只切换后端，不反向修改数据。
- 同一进程内的存储读写由应用级异步锁串行化；YAML 写入使用同目录临时文件、文件 flush/sync 与平台原子替换。并发测试必须证明不会出现部分文档或丢失已确认写入。

## Proposed YAML Contract

```yaml
schema_version: 1

sessions:
  - id: production-web
    name: Production Web
    host: example.com
    port: 22
    username: deploy
    group_name: production
    auth:
      method: private_key
      private_key: ~/.ssh/id_ed25519
    created_at: "1775400000"
    updated_at: "1775400000"
```

- `auth.method` 允许 `password` 或 `private_key`。
- `password` 字段不属于 v1 YAML schema。
- `auth.private_key` 只允许保存文件路径，不允许嵌入私钥内容。
- YAML 读取时出现 `password`、私钥 PEM/OpenSSH 内容、未知字段或禁止构造都必须失败，不得忽略或透传。
- 应用生成的 YAML 使用稳定字段顺序和两空格缩进，文件结尾保留换行，便于 diff 和版本控制。

## Non-Goals

- 不在 G1 中实现 OS keychain/credential manager。
- 不支持 JSON、TOML、多格式自动探测或 YAML 高级构造。
- 不做实时文件监听、多人协同编辑或远程同步。
- 不自动连接真实 SSH 主机验证配置。
- 不在没有显式确认的情况下迁移、覆盖或删除现有 SQLite 数据。

## Durable Control Invariants

- `harness-rule:path-containment`
- `harness-rule:run-dag-ownership`
- `harness-rule:candidate-accepted-evidence`
- `harness-rule:authoritative-completion-state`
- `harness-rule:state-sync-evidence`
- `harness-rule:bounded-status-snapshot`
- `harness-rule:project-neutral-core`
- `harness-rule:durable-tier-boundary`

## Codex-Native Execution

- 完整 G1 Spec 接受前只允许规格调整，不启动实现。
- 接受后使用 durable Goal/Run；Codex Plan 记录当前执行步骤，Harness 记录恢复状态、证据和门禁。
- 实现阶段按 M2 → M3 → M4 顺序推进；每个里程碑完成后同步 bounded status。

## Acceptance Criteria

- 现有 SQLite 用户升级后默认行为和数据位置不变。
- SQLite 与 YAML 后端对非敏感会话字段提供一致 CRUD 语义。
- YAML 后端重启后可恢复所有已写入的非敏感字段。
- YAML 文件写入失败不会破坏最后一个有效版本。
- 损坏 YAML、重复 ID、无效端口、禁止的高级构造和未知 schema 版本会显式失败且不回退为空列表。
- 密码和私钥内容不会写入 YAML；测试 fixture 不包含真实凭据。
- YAML 会话的密码只在发起连接时作为命令参数临时传入，不进入 Session model、事件、日志或文件；应用重启后必须再次输入。
- 后端切换与数据复制由用户显式触发，复制验证失败时源数据保持不变。
- 前端构建、Rust 测试和双后端集成测试通过。

## Spec Acceptance Checklist

- Item: YAML 格式与 schema v1
  - Acceptance: YAML 1.2 安全子集、字段、版本策略、严格校验和稳定序列化得到确认。
  - Evidence: 用户于 2026-08-03 确认 YAML 方向；本 Spec 的 Decision、YAML Profile 与 Proposed YAML Contract。
  - Status: accepted
  - Unblocker: N/A
- Item: 注释与写回策略
  - Acceptance: 明确应用 CRUD 写回时对用户注释和排版的保留承诺，并提供不会丢失配置数据的验证方案。
  - Evidence: YAML Profile 已明确“读取注释、规范化写回、写回前保存唯一原始字节恢复副本、备份失败即拒绝写回”；批次 2 验证原始字节可恢复。
  - Status: accepted
  - Unblocker: N/A
- Item: 兼容与迁移边界
  - Acceptance: SQLite 保持默认，迁移显式且不删除源数据。
  - Evidence: Scope 与 Acceptance Criteria 已固定“复制并校验后切换”、非空目标拒绝、失败不切换、不删除或覆盖源数据；用户已授权 Gate 0 后依次执行。
  - Status: accepted
  - Unblocker: N/A
- Item: 凭据安全边界
  - Acceptance: YAML 不保存密码/私钥内容，密码在连接时临时提供。
  - Evidence: Decision、YAML Contract、Acceptance Criteria 与 Non-Goals 已固定 YAML 排除 password/私钥内容且连接时一次性传入；用户已授权按该安全边界实施。
  - Status: accepted
  - Unblocker: N/A

## Required Gate Evidence

- Gate: spec
  - Required: yes
  - Evidence: `harness/specs/G1-dual-storage-yaml.md`
  - Status: accepted
  - Unblocker: N/A
- Gate: execution
  - Required: yes
  - Evidence: 单一 G1 Run 的批次 1-4 串行执行；SQLite/YAML 接口、严格 YAML、StorageManager、设置 UI 和一次性密码均已实现，未创建 branch/worktree/commit/push。
  - Status: satisfied
  - Unblocker: N/A
- Gate: integration
  - Required: yes
  - Evidence: `cargo test --manifest-path src-tauri/Cargo.toml` 15 项通过，覆盖双后端 CRUD、重启、损坏文件、并发、双向复制、非空目标与设置提交回退；`npm run build` 和桌面/窄屏 Browser 验证通过。
  - Status: satisfied
  - Unblocker: N/A

## Verification

- `npm run build`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- YAML codec 单元测试：round-trip、注释输入、安全子集、稳定输出、损坏文件、未知版本、重复 ID、非法端口。
- YAML 恢复副本测试：带注释/自定义排版的有效文件首次写回前保留原始字节；副本创建失败时目标文件不变。
- 后端契约测试：同一 CRUD suite 分别运行于临时 SQLite 和临时 YAML 文件。
- 原子写失败测试：失败后旧文件仍可解析并保持原数据。
- 应用级测试：后端选择、重启恢复和显式复制；不连接真实 SSH 主机。

## State Sync

- Records: `harness/tasks.md`、`harness/status.md`、`harness/milestones/roadmap.md` 和后续 G1 Goal/Run。
- Evidence: Spec 接受记录、实现 diff、测试命令输出和集成门禁记录。

## Pause Conditions

- 用户要求 YAML 保存明文密码或私钥内容，但没有明确接受安全影响。
- 需要自动覆盖/删除既有 SQLite 数据。
- 跨平台原子替换语义无法通过可重复测试验证。
- 注释/排版的写回承诺与选定解析库能力不匹配。
- 实现需要真实凭据、真实 SSH 主机、发布或其他外部副作用。
