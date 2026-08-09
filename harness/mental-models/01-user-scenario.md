# User / Scenario Model

本模型说明维护者和 Codex 如何在 w-ssh 中使用 Harness。

## Adoption State

- 本仓库是已存在项目，于 2026-08-03 以 `adapter` contract 接入 Harness。
- 既有产品事实来自源码、README、docs 和构建结果；Harness artifacts 从接入日起成为工作控制状态来源。
- `harness/tasks.md` 拥有 Goal 阶段，`harness/status.md` 只保存当前 bounded projection。

## Entry Flow

```text
读取 AGENTS 与 adapter
-> config inspect / orient next
-> 读取唯一焦点的 Spec/Goal/Run
-> 选择 direct、postflight 或 durable-harness
-> 执行已授权范围
-> 验证
-> 同步 Goal、status 与证据
```

## Human / Agent / Harness Division

### Human owns

- 产品目标、优先级和最终接受。
- 对真实 SSH、凭据、数据迁移、branch/commit/push 和发布的授权。
- 对 Spec 中兼容性或安全策略分歧的最终判断。

### Codex owns

- 读取代码和 Harness 状态，给出证据支持的当前判断。
- 在已授权边界内塑形、实现和运行最小相关验证。
- 区分 candidate evidence 与 accepted evidence，并同步持久化状态。
- 遇到产品方向、安全或不可逆数据影响时暂停并请求确认。

### Harness owns

- Goal 阶段、Spec、roadmap、Run、完成门禁和恢复上下文。
- 当前焦点、阻塞和验证结果的 bounded projection。
- durable work 的连续性，不替代源码和实际验证证据。

## Common Scenarios

### Orientation

1. 运行 `agent-harness config inspect --cwd . --json`。
2. 运行 `agent-harness orient next --cwd . --json`。
3. 汇总 ready、doing、review、blocked 和 done，不启动实现。

### Requirement Intake

1. 将新需求与现有 Goal、Spec 和 roadmap 对照。
2. 记录优先级、依赖、范围、非目标、风险和验证问题。
3. 产品方向清楚时创建或更新 Spec；不因记录需求而自动开始实现。

### Durable Development

1. Spec 被接受后创建 Goal，并准备 Run。
2. Goal/Run 持有恢复状态和门禁，Codex Plan 持有当前执行步骤。
3. 按依赖执行、验证并同步；完成需要 spec、execution 和 integration 证据。

### Ordinary Direct Work

- 小而清晰、无持久状态义务的修改可直接由 Codex 完成。
- 若工作已属于现有 Goal，只做 bounded postflight sync，不为记账额外创建 Run。
- 已准备的 enforced Run 不降级为 postflight。

## Confirmation Boundary

可直接继续：读取 artifacts、总结状态、草拟 Spec/Goal preview、运行本地只读检查和任务范围内的非外部验证。

需要确认：从未接受的塑形进入实现、改变产品方向、真实 SSH/凭据、破坏性迁移、branch/worktree、commit/push、发布或付费外部操作。

## Stability Promise

- orientation: 任一后续任务可从 Goal index、status 和当前 Spec 恢复上下文。
- intent: 用户最新明确决定优先于旧 artifact，并在确认后同步回状态。
- boundaries: 不把需求记录、构建成功或传输成功误报为产品完成。
- evidence: 只有被控制任务核验并记录的结果才是 accepted evidence。
- continuity: 交接必须指向唯一下一入口，并区分阻塞、等待接受和 deferred。
