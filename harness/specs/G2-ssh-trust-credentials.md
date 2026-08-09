# Spec: G2 SSH 主机信任与凭据安全加固

Created: 2026-08-04
Status: accepted

## Authority Boundary

- 用户已于 2026-08-04 接受 `DR-1=A`、`DR-2=A`、`DR-3=A`、“用户方便第一、安全第二”的整体原则及完整 Spec，并明确授权按推荐方案推进。
- 本文件现为 G2 accepted contract；正式 G2 Goal/Run 必须引用本文件并保留其安全、迁移、真实环境和交付边界。
- 当前执行位置固定为已有 `D:\project\w-ssh` saved project/local checkout；不创建或切换 branch/worktree，不 commit、push、发布或连接真实 SSH 主机。

## Product Priority

- 第一优先级是减少用户日常连接、保存和升级迁移的操作负担；安全约束作为第二优先级，应尽量在后台完成而不是反复打断用户。
- “安全第二”不表示允许无条件接受变化的主机 key、向普通文件回退保存 secret、静默迁移/删除旧密码，或把 secret 返回前端列表。这些是防止凭据发给错误主机和不可恢复数据影响的硬边界。
- 候选 UI 应把首次信任、保存到系统凭据库和旧密码批量迁移设计为少步骤、可理解的显式动作；正常匹配的后续连接不重复确认。

## Observed Baseline

- `src-tauri/src/ssh.rs` 的 `check_server_key()` 当前无条件返回 `Ok(true)`，任何服务端主机密钥都会被接受。
- SQLite `sessions.password` 当前保存明文密码；list/get CRUD 会将该字段装入 `Session`，并可通过 Tauri IPC 返回前端。
- YAML schema v1 已禁止密码和私钥内容；密码会在连接时一次性传入。G2 必须保持该 G1 契约。
- 私钥目前按路径读取，`decode_secret_key(..., None)` 不支持用户提供 passphrase；错误中可能包含完整私钥路径或第三方错误文本。
- 当前固定依赖 `russh 0.46.0` / `russh-keys 0.46.0` 能提供 SHA-256 公钥指纹和 known-hosts 读取能力，但其直接追加写接口不满足本 Spec 的并发、原子替换和可恢复错误要求。
- 已检查本地 `russh 0.46.0` 源码：`client::connect()` 在密钥交换中调用 `Handler::check_server_key()`，只有该步骤接受并返回 handle 后，调用方才会执行 `authenticate_password()` / `authenticate_publickey()`。G2 可据此保证系统凭据读取和用户认证发生在主机验证之后。
- 候选 `keyring 4.1.x` 默认 `v1` feature 覆盖 Apple Keychain、Windows Credential Manager 和基于 zbus 的 Secret Service，许可证为 `MIT OR Apache-2.0`，MSRV 1.88；当前本机 Rust 1.93 满足本地编译前提。精确版本和跨平台依赖树仍需在 Spec 接受后的实现节点锁定并验证。
- G1 已达到 `done / validated-local`；真实 SSH、真实系统凭据库、跨平台安装包和实际 CI 运行均未验证。

## Decision Requests

### DR-1 首次连接信任模型

请选择：

- **A（推荐）：TOFU + 每个未知端点的显式用户确认。** 首次握手只取得主机公钥，不发送密码、passphrase 或用户私钥签名；应用显示规范化端点、算法和 `SHA256:<base64-no-padding>` 指纹。用户点击“信任并连接”后写入 app-owned known_hosts，再发起新的完整连接。主机密钥变化始终 fail closed，不能在连接错误弹窗中一键绕过；只能到“主机信任详情”查看旧/新指纹并走独立替换流程。
- **B：严格预配。** 未预先加入 known_hosts 的端点一律拒绝，不提供首次连接确认；用户必须先在信任管理界面粘贴或导入管理员提供的 OpenSSH 公钥记录。

两种方案都不允许“仅本次忽略”或自动接受变化。A 更适合本地会话管理器的首次使用，B 更适合集中管理环境。

Status: accepted by user on 2026-08-04 (`DR-1=A`).

### DR-2 known_hosts 作用域

请选择：

- **A（推荐）：应用私有、OpenSSH 兼容格式。** 固定使用 `{app_data_dir}/known_hosts`，由 w-ssh 独占写入；不自动读取或修改用户的 `~/.ssh/known_hosts`。后续可提供显式导入，但导入仍先预览并写入应用私有文件。
- **B：共享用户级 OpenSSH 文件。** 直接使用 `~/.ssh/known_hosts`，获得 CLI 互操作，但 w-ssh 的替换/删除会影响系统 SSH 客户端，且必须兼容更多 marker、通配符、hashed host、CA 和外部并发写入语义。

A 可将 G2 的写入所有权和回退边界保持在应用内，同时仍采用标准行格式供用户检查。

Status: accepted by user on 2026-08-04 (`DR-2=A`).

### DR-3 凭据持久化与既有明文迁移

请选择：

- **A（推荐）：系统凭据库按用户选择持久化 + 一次性输入回退。** 密码和私钥 passphrase 只有用户明确选择“保存到系统凭据库”时才持久化；系统凭据库不可用、锁定或拒绝访问时，只允许本次输入，不回退到 SQLite、YAML、普通文件或环境变量。既有 SQLite 明文密码升级后保留但立即隔离：不通过普通 CRUD/IPC 返回，也不用于自动连接；用户可显式“迁移到系统凭据库”或“停止保存并删除旧明文”。
- **B：完全不持久化。** 所有密码和 passphrase 每次连接输入；既有 SQLite 明文仍需用户显式确认后删除，应用不提供迁移到系统凭据库。
- **C：兼容过渡。** 新凭据使用系统凭据库，但既有 SQLite 明文可在只限 Rust 后端的过渡期继续用于连接并持续警告，直到用户迁移。该方案升级摩擦较小，但完成 G2 后仍暂时依赖普通明文持久化，因此不推荐。

无论选择何项，都不得在升级时静默删除、导出、显示或记录既有密码。

Status: accepted by user on 2026-08-04 (`DR-3=A`).

## Recommended Candidate Contract

以下内容以已接受的 `DR-1=A`、`DR-2=A`、`DR-3=A` 为 accepted contract。

### 1. Host Identity And Fingerprint

- 信任键由规范化的 `(host, port)` 唯一标识，与 session 名称、group 和 username 无关。
- 输入 host 先 trim 并拒绝空值、NUL、控制字符、路径分隔语义和内嵌端口。IP literal 使用标准 `IpAddr` 解析；DNS 名转为小写 IDNA ASCII 并移除一个末尾点。
- 内部端点保留裸 host 与 `u16` port；OpenSSH 文本中端口 22 使用 `host`，非 22 使用 `[host]:port`。IPv6 必须避免双重方括号。
- 不做 DNS 反查、CNAME 展开或“域名与解析后 IP 同时信任”。连接哪个规范化 host，就只匹配哪个 host。
- UI 指纹统一显示为 `SHA256:<base64-no-padding>`，同时显示主机密钥算法；不使用 MD5 指纹作为接受依据。

### 2. Host Trust State Machine

- `trusted-match`：端点和公钥精确匹配，才允许进入用户认证。
- `unknown`：端点无记录，握手在认证前中止并返回结构化 challenge；challenge 只包含规范化端点、算法、SHA-256 指纹和短生命周期 challenge id，不包含凭据。
- `changed`：端点已有记录但公钥不匹配，始终 fail closed；返回旧/新算法与指纹。普通连接界面不得提供绕过按钮。
- `unsupported`：服务端只提供当前安全策略不支持的主机密钥算法，明确失败，不降级到弱算法。
- `store-error`：known_hosts 缺失可视为空；不可读、不可解析、权限错误或原子写失败必须明确失败，不得当作空文件或自动接受。
- UI 连接流程先调用不含任何 secret 的 trust preflight。只有返回 `trusted-match` 或用户完成首次信任后，才显示一次性 secret 输入或请求后端读取系统凭据。
- 正式连接仍必须在同一 SSH transport 上再次执行 `check_server_key()`；后端只能在 `client::connect()` 返回已验证 handle 后读取系统凭据并进入用户认证，不能把 preflight 结果当作跳过二次验证的通行证。
- 首次确认必须消费未过期 challenge，并重新建立连接；确认动作本身不携带或缓存密码。若新连接呈现不同公钥，再次 fail closed。
- 替换流程位于“主机信任详情”：先做不认证的当前密钥探测，再显示端点、旧/新算法和两枚指纹，要求独立危险操作确认；替换后保留不含公钥私密材料的审计元数据。删除信任同样需要显式确认，删除后下次连接回到 `unknown`。

### 3. known_hosts File Contract

- 候选路径为 `{app_data_dir}/known_hosts`；文件不是秘密，但只能由当前用户写入，并尽力限制其他本机用户读取。
- 使用 OpenSSH known_hosts 行格式。G2 写出普通规范化 endpoint、key type、base64 public key 和可选 w-ssh comment；不写 hashed hostname、wildcard、`@cert-authority` 或 `@revoked`。
- 读取支持空行和 `#` comment。遇到语法损坏、重复端点下互相冲突的同算法 key、未知 marker 或当前实现无法安全解释的记录时整体 fail closed，并给出行号；不得跳过坏行后继续连接。
- 一个端点可并存多个受支持算法的公钥，但服务端呈现的具体公钥必须有精确记录。算法轮换或由 RSA 切换到 Ed25519 仍视为变化，需要显式替换/新增确认。
- G2 支持当前 `russh` 可验证的 Ed25519、ECDSA P-256/P-384/P-521 和 RSA key；RSA 认证只允许 SHA-2 签名协商，不允许 SHA-1 `ssh-rsa` 签名降级。host certificate、FIDO host key 和尚未被当前依赖支持的新算法明确报 `unsupported`，不静默降级。
- RSA known_hosts key type 按公钥材料规范化为 OpenSSH `ssh-rsa`，与协商得到的 `rsa-sha2-256` / `rsa-sha2-512` 签名算法分离；比较 RSA key 时忽略 `russh` 对同一公钥附带的 signature-hash variant。不得直接用 `PublicKey::name()` 同时代表存储 key type 和协商签名算法。
- 写入在进程内使用单一异步 mutex，并使用跨进程独占 lock/sidecar。每次执行 read-validate-modify，写入同目录唯一临时文件，flush/sync 后原子替换；失败时旧文件保持原样。
- 不直接使用 `russh_keys::learn_known_hosts_path()` 的 append 行为作为最终写路径。解析和序列化需要 fixture 证明与 OpenSSH 基础格式兼容。
- 应用不自动从备份恢复信任文件；自动恢复旧信任可能重新接受已经轮换或撤销的 key。错误修复必须由用户查看后显式完成。

### 4. Credential Boundary

- 引入可注入的 `CredentialProvider` 接口；生产 provider 映射到 Windows Credential Manager、macOS Keychain、Linux Secret Service，测试只用 in-memory/mock provider。
- 稳定 service namespace 使用 Tauri identifier `com.wssh.app`；item account/reference 由 session UUID 和 secret kind（`password` 或 `private_key_passphrase`）派生，不含 secret。
- 存储层 session metadata、wire DTO 和 transient secret input 必须拆分类型；`SessionStorage` 的 list/get/create/update/import contract 不接收或返回 secret-bearing 类型。
- YAML schema v1 保持不变，只保存现有 `auth.method` 和私钥路径；不新增 credential reference/state 字段。provider item reference 由 session UUID + secret kind 确定性派生，provider availability 和 credential state 在 Rust 边界按需计算。
- SQLite 可新增非敏感认证/迁移状态列，但不保存新 secret。凭据与 session ID 关联，因此 SQLite/YAML 复制不复制、导出、覆盖或删除系统凭据；相同 session ID 在两种 metadata 后端之间切换时引用同一 provider item。
- 普通 `Session` DTO 和 list/get IPC 永远不含 password/passphrase。前端只能看到 `credential_state`，例如 `none`、`stored`、`legacy_quarantined`、`unavailable` 或 `needs_rebind`。
- secret 只允许从专用输入命令进入 Rust，不能进入 Pinia session 列表、Tauri event、URL、配置文件或日志。连接完成或失败后尽快清空前端输入和 Rust 临时 buffer。
- 用户选择“仅本次”时，只在 trust preflight 已通过后把 secret 传给当前 `ssh_connect`；正式连接仍二次核验 host key，并在核验失败时不发送 secret。该行为与 G1 的 YAML 一次性密码一致，应用重启后必须重新输入。
- 保存的 password/passphrase 只能在正式连接的 `client::connect()` 返回已验证 handle 后从 provider 读取。读取失败不得继续认证，也不得回退到 legacy SQLite 明文。
- 用户选择“保存”时，先成功写入并回读验证系统凭据库，再提交非敏感 metadata。provider 失败时保持原 metadata，不降级保存到普通文件。
- 密码/passphrase 输入界面的“保存到系统凭据库”默认选中但始终可见、可取消；用户提交该表单视为本次明确保存授权。系统 provider 不可用时自动取消保存意图并继续提供仅本次输入，不阻塞 metadata 管理。
- 删除 session 不静默删除系统凭据。删除确认需要单独说明是否同时删除保存的凭据；provider 删除失败时不得谎报全部删除成功。
- 保存凭据绑定规范化 endpoint、username、auth method 和 secret kind 的非敏感 hash。修改其中任一字段后状态变为 `needs_rebind`，旧 secret 不会自动发送到新目标；用户必须显式确认重新绑定、替换或删除。

### 5. Existing SQLite Password Migration

- schema 升级只新增非敏感字段/迁移记录，不改写或清空现有 `sessions.password`。
- 启动时只统计含旧明文的 session 数量并标记 `legacy_quarantined`；普通 list/get、编辑表单和连接流程不得读出或自动使用其值。
- “迁移到系统凭据库”是显式用户动作。Rust 后端在单次操作中读取旧值、写入确定性 provider item、回读校验，再用 SQLite transaction 写入 credential metadata 并将该行旧 password 置空。
- 便利优先的入口包括启动后的批量“安全迁移”提示，以及 legacy session 首次连接时的主操作“安全迁移并连接”；两者均无需用户重新输入现有密码，但必须由用户点击触发，并逐条报告成功/失败。
- 在 provider 写入后、SQLite commit 前崩溃时，原明文仍保留；确定性 item id 允许安全重试。不得为了清理潜在重复项先删除原明文。
- provider 写入/验证失败时数据库完全不变。SQLite commit 失败时尽力清理本次新建 provider item；清理失败仅记录不含 secret 的 orphan 状态，后续可重试，不把迁移报告为成功。
- “停止保存并删除旧明文”必须逐会话或明确全选确认；只在 SQLite transaction 成功后报告完成。该动作不可由升级、后端切换或 YAML 复制隐式触发。
- G2 不提供查看、复制到剪贴板、导出或重新显示旧密码的入口。

### 6. Private Key Passphrase

- 私钥内容继续只从用户选择的路径按需读取，绝不进入 SQLite、YAML、系统凭据库或日志；系统凭据库只可保存用户明确授权的 passphrase。
- 未加密私钥直接解码；加密私钥返回结构化 `PrivateKeyPassphraseRequired`，前端提供“仅本次”或“保存到系统凭据库”输入。
- passphrase 与 password 使用相同 provider、rebind、一次性和脱敏规则，但使用独立 secret kind，不能互相回退。
- 私钥字节、passphrase 和 password 使用可清零容器并缩短存活期；不派生 `Debug`/`Serialize`，不放入长期共享 state。底层库无法保证的内存副本必须在代码注释和残余风险中准确说明。
- G2 不实现私钥内容托管、ssh-agent、PKCS#11 或硬件安全密钥。

### 7. Errors, Logs And UI Safety

- IPC 使用稳定错误 code + 可展示字段，至少包括 `HOST_KEY_UNKNOWN`、`HOST_KEY_CHANGED`、`HOST_KEY_UNSUPPORTED`、`HOST_TRUST_STORE_ERROR`、`CREDENTIAL_MISSING`、`CREDENTIAL_PROVIDER_UNAVAILABLE`、`CREDENTIAL_LOCKED`、`PRIVATE_KEY_PASSPHRASE_REQUIRED` 和 `AUTHENTICATION_FAILED`。
- password/passphrase、私钥内容、旧 SQLite 明文、provider 返回值永远不进入 error chain、`Display`、`Debug`、日志、事件或测试 snapshot。
- UI 可以按当前操作显示 endpoint、算法、SHA-256 指纹和用户选择的私钥路径；后台日志默认只记录 session id、错误 code 和阶段，不记录 host、username、完整路径或第三方原始错误文本。
- 第三方 provider/SSH 错误先映射为受控错误；用户认证失败不区分“密码错误”与“账号不存在”等服务端细节。
- 所有确认框都必须区分“信任主机身份”和“保存认证凭据”，不能用一次点击同时授权两件事。

### 8. Platform Capability And Fallback

| Platform | Production target | Expected failure modes | Required fallback |
| --- | --- | --- | --- |
| Windows | Windows Credential Manager | policy denial, item missing, API error | one-time input only; no DPAPI/file fallback in G2 |
| macOS | Keychain Services | keychain locked, access prompt denied, unsigned/dev app ACL differences | one-time input only; surface locked/denied state |
| Linux desktop | Secret Service over user-session D-Bus | no service, headless session, collection locked, prompt denied | one-time input only; metadata and host trust remain usable |

- provider 不可用不能阻止应用启动、会话 metadata CRUD 或 host trust 管理；只影响“保存/读取已保存凭据”。
- 不把 mock/sample keyring、内存 provider 或普通加密 SQLite 当作生产回退。
- 候选依赖使用 `keyring 4.1.x` 的 `v1` surface 或等价的 `keyring-core` + 三个明确 provider；不启用 CLI、sample、SQLite store 或 Linux keyutils store。Linux 必须显式选择持久化 Secret Service，不能调用会默认选择 keyutils 的 native-store 分支。
- provider API 是同步/平台阻塞边界时，所有调用进入受控 `spawn_blocking`/单一 provider lane；同一 credential 的 set/get/delete 必须串行，避免 Windows provider 对并发调用顺序不作保证。
- `keyring` 及 provider 的精确版本、feature、transitive native dependency、许可证和三目标编译结果是 `credential-provider-core` 的进入门禁；本 Spec 不以“跨平台 crate”标签替代实际构建证据。
- 当前 Windows 开发机只能对 adapter、mock provider 和临时文件行为给出 `validated-local` 证据。真实 Windows Credential Manager、macOS Keychain、Linux Secret Service、系统弹窗/锁定行为和安装包权限必须明确列为未验证，不能由单平台单元测试推断。

## Implementation Shape After Acceptance

正式 Goal/Run 只在 Spec 接受后创建，建议串行 DAG：

1. `trust-store-core`: endpoint 规范化、指纹、严格 parser、原子/并发 trust store 和 fail-closed 单元测试。
2. `credential-provider-core`: provider trait、mock provider、非敏感 credential metadata、DTO/IPC secret 隔离。
3. `legacy-migration`: SQLite quarantine 与显式幂等迁移/删除流程；保持 YAML schema 的 secret 禁止边界。
4. `ssh-ui-integration`: unknown/changed/replacement flow、一次性 password/passphrase、provider error 和 rebind UI。
5. `integration-docs`: 可控本地 SSH server、完整 Rust/前端验证、文档与 Harness 状态同步。

所有节点由当前唯一控制任务串行拥有；未另行授权时不委派 worker，不并行写文件。

## Acceptance Criteria

- 未知主机在用户显式确认前不会进入用户认证；已知主机 key 变化和 trust store 错误均 fail closed。
- 指纹为 SHA-256，endpoint 规范化、端口/IPv6、算法兼容、损坏文件和并发原子写有自动化覆盖。
- SQLite/YAML session CRUD 和前端 session state 不再返回或保存 password/passphrase；YAML 继续拒绝 password 和私钥内容。
- 新 secret 只存在于一次性连接内存或用户明确选择的系统凭据库；provider 失败不回退明文。
- 既有 SQLite 明文不被升级静默删除、显示、导出或自动使用；显式迁移具有 read-after-write 验证和可重试失败语义。
- 加密私钥支持一次性 passphrase，并可按用户选择保存到同一系统 provider；私钥内容始终只在 Rust 连接路径短暂读取。
- host/username/auth 变化不会把旧保存凭据自动发送到新目标。
- 日志、错误、Tauri event、测试输出和 fixture 不包含真实或 fixture secret 原文（专门的泄漏断言除外且只使用明显虚构 marker）。
- 本地验证不连接外部 SSH 主机，不访问真实系统凭据库，不读取真实私钥；跨平台/真实 provider 边界被准确标记为未验证。

## Verification Plan

- Rust unit/contract tests：endpoint 规范化、SHA-256 指纹、known_hosts parse/serialize、unknown/match/change/unsupported、原子失败、并发 writer、mock credential CRUD/rebind、legacy quarantine/migrate/retry/delete、secret leakage assertions。
- SSH ordering tests：在 mock provider 上记录调用序列，证明 unknown/changed/store-error 路径从不执行 provider get 或用户认证；match 路径严格为 host check -> provider get -> authenticate。
- Controlled integration：测试进程内生成 host key 并启动仅监听 loopback 的可控 SSH server，验证首次握手不认证、确认后匹配、key 变化拒绝；fixture 不含真实账号、密码或私钥。
- Frontend：`npm run build`，并用 mock command 响应验证 unknown/changed、迁移、provider unavailable、passphrase 和窄屏弹窗。
- Rust：`cargo test --manifest-path src-tauri/Cargo.toml`；按依赖能力补充 `cargo clippy --manifest-path src-tauri/Cargo.toml`。
- Repository：Harness config/Goal validation（正式 Goal 创建后）、`git diff --check`、secret-pattern scan；不得把测试 marker 误报为真实凭据。
- Deferred：真实 OS credential store、真实 SSH、macOS/Linux 行为、安装包和实际 CI 运行，除非用户后续单独授权并提供安全测试边界。

## Spec Acceptance Checklist

- Item: DR-1 首次连接信任模型
  - Evidence: 用户于 2026-08-04 明确选择 A（TOFU + 首次显式确认，主机 key 变化 fail closed）。
  - Status: accepted
  - Unblocker: N/A
- Item: DR-2 known_hosts 作用域
  - Evidence: 用户于 2026-08-04 明确选择 A（app-owned OpenSSH-compatible known_hosts）。
  - Status: accepted
  - Unblocker: N/A
- Item: DR-3 凭据持久化与迁移
  - Evidence: 用户于 2026-08-04 明确选择 A（系统凭据库按需持久化、一次性输入回退、旧明文显式验证后迁移）。
  - Status: accepted
  - Unblocker: N/A
- Item: 完整安全契约
  - Evidence: 用户于 2026-08-04 明确回复“完整 Spec 确认并按推荐方案推进”。
  - Status: accepted
  - Unblocker: N/A

## Non-Goals

- 不连接真实 SSH 主机，不访问真实系统凭据库，不读取真实 password/passphrase/私钥内容。
- 不自动迁移、删除、显示、导出或继续使用既有 SQLite 明文密码。
- 不向 YAML 添加 password、passphrase 或私钥内容。
- 不实现云同步、团队共享 vault、主机 CA/certificate、DNS SSHFP、ssh-agent、PKCS#11、硬件安全密钥或全局 `~/.ssh` 管理。
- 不创建 branch/worktree/commit/push，不发布、部署或执行真实跨平台安装验证。

## Reference Basis

- OpenSSH known_hosts format and revoked/CA semantics: <https://man.openbsd.org/OpenBSD-current/man8/sshd.8>
- OpenSSH SHA-256 fingerprint and `[hostname]:port` operations: <https://man.openbsd.org/ssh-keygen.1>
- Windows password handling guidance: <https://learn.microsoft.com/en-us/windows/win32/secbp/handling-passwords>
- Apple Keychain Services: <https://developer.apple.com/documentation/Security/using-the-keychain-to-manage-user-secrets>
- Freedesktop Secret Service: <https://specifications.freedesktop.org/secret-service/latest-single/>
- Rust keyring provider surface (candidate dependency, exact version deferred to implementation validation): <https://docs.rs/keyring/latest/keyring/cli/>

## Pause Conditions

- 依赖实现无法保证主机认证先于用户凭据发送、known_hosts fail-closed 或 secret 不经普通 DTO/IPC 返回。
- 跨平台 provider 需要明文文件、环境变量或未获授权的真实凭据库作为回退。
- 迁移路径可能静默删除旧密码、向前端暴露旧值或在失败后无法判断原值是否仍保留。
- 工作需要真实 SSH、真实凭据/私钥、破坏性数据操作、branch/worktree、commit/push、发布或其他未授权副作用。
