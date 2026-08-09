# Work Unit Model

This model answers what one harness-managed unit of work is.

```text
Goal with Tasks + status + spec + DAG + run + gate
```

## Components

- goal: 一个可验收的产品结果，例如 G1 双存储后端。
- tasks: Goal 内按依赖拆分的实现与验证节点。
- status: 当前唯一焦点、阶段、验证和阻塞的 bounded projection。
- spec: 决策、范围、非目标、验收条件和暂停条件。
- DAG: 存储契约 → 后端实现 → UI/迁移 → 集成验证。
- run: 接受 Goal 后的执行和验证证据；塑形阶段不创建 Run。
- gate: spec、execution、integration 三类持久化完成门禁。
- route explanation: 普通清晰改动 direct；已有状态的简单同步 postflight；跨阶段 Goal durable-harness。
- evidence: 源码、diff 和命令输出先是 candidate，控制任务校验并记录后才 accepted。

## Artifact Mapping

- Goal index: `harness/tasks.md`
- status file: `harness/status.md`
- specs: `harness/specs/`
- goals: `harness/goals/`
- milestones and deferred registers: `harness/milestones/`
- run logs: `.harness/runs/`
- mental models: `harness/mental-models/`
