# Control Loop / Handoff Model

This model answers how Codex enters, executes, verifies, updates state, stops,
and hands work off.

## Loop

- input: 用户目标、优先级、接受或变更请求。
- context: `AGENTS.md`、adapter、Goal index、bounded status 和相关 Spec/Goal/Run。
- route explanation: 说明为何选择 direct、postflight 或 durable Harness。
- action: 只执行已接受范围，真实 SSH、凭据和发布操作需要额外授权。
- verification: 前端与 Rust 分层验证，再按风险进行 Tauri 集成验证。
- state update: 将阶段写入 Goal index，将当前投影写入 status，将证据写入 Goal/Run。
- stop condition: 完成门禁满足，或遇到产品方向、安全、凭据、破坏性数据迁移等暂停条件。

## Handoff

- what was requested: 记录用户原始目标及后续明确决定。
- what was decided: 标明格式、兼容、凭据和迁移策略。
- what changed: 列出代码与 Harness 状态的必要修改。
- what was verified: 记录命令、结果和未覆盖边界。
- candidate evidence not yet accepted: 标记尚未由控制任务确认的 worker、CI 或人工结果。
- what remains blocked or deferred: 区分真正阻塞、等待接受和计划后置项。
- where the next action starts: 指向唯一 Goal/Spec/Run 节点。
