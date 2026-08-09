# Ownership / Boundary Model

This model answers what belongs to plugin core, project adapters, artifacts,
and pause rules.

```text
Plugin defines protocol. Adapter defines overrides. Artifacts record facts.
```

## Ownership

- plugin core: Harness 协议、CLI、schema、生命周期和通用门禁语义。
- project adapter: w-ssh 的路径、验证命令、SSH/凭据边界和中文沟通偏好。
- artifacts: Goal 状态、Spec、roadmap、Run、门禁与验证证据。
- control lane acceptance: 当前主任务拥有范围接受和 authoritative state；worker 只返回 candidate evidence。
- candidate evidence sources: 源码检查、构建输出、测试、CI、人工 UI 检查和子任务报告。
- project-neutral docs boundary: Harness 通用规则不写进产品文档；w-ssh 行为不反向修改插件 core。

## Precedence

1. Current user instruction.
2. Repository `AGENTS.md` and nested instructions.
3. Project adapter.
4. `.harness/config.json`.
5. Plugin canonical defaults.

## Pause Conditions

- cost: 需要新增付费服务、证书或外部基础设施。
- risk: 可能损坏会话数据或降低 SSH 安全性。
- product direction: 后端选择、迁移语义或凭据策略与已接受 Spec 冲突。
- production safety: 需要连接真实主机或操作真实用户数据。
- compatibility: 需要改变默认后端、YAML schema 或旧 SQLite 数据语义。
- credentials: 需要读取、输出或持久化真实密码/私钥内容。
- paid calls: 需要收费 API 或外部账号操作。
- destructive operations: 需要覆盖、删除或不可逆迁移源数据库/配置文件。
- release behavior: 需要 tag、push、GitHub Release 或发布安装包。
