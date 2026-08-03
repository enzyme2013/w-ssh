# w-ssh Roadmap

Updated: 2026-08-03

## Current Baseline

- Vue 3 + Pinia + xterm.js 前端和 Tauri 2 + Rust 后端已形成可构建 MVP。
- SQLite 仍是默认会话存储；用户可显式选择严格 YAML schema v1，YAML 不保存密码或私钥内容。
- SSH 支持密码或私钥路径认证、多终端 Tab、输入输出和窗口 resize。
- CI 在 `v*` tag 上构建 Windows、macOS 和 Linux 产物；本轮未验证实际 CI 运行或安装包。
- 前端生产构建通过；Rust 有 15 项本地自动化测试覆盖双后端 CRUD、重启、损坏、并发、原子失败、双向复制与回退。

## Ordered Roadmap

| Milestone | Goal | Status | Depends on | Exit condition |
| --- | --- | --- | --- | --- |
| M0 Harness 接入与基线盘点 | G0 | done | - | Adapter、Goal 索引、bounded status 和 roadmap 已建立，基线命令已验证 |
| M1 双存储规格与安全契约 | G1 | done | M0 | YAML schema、写回规则、错误模型、凭据边界和兼容策略被接受 |
| M2 存储抽象与 YAML 后端 | G1 | done | M1 | SQLite/YAML 实现同一接口，YAML 原子写入，默认仍兼容 SQLite |
| M3 设置 UI、切换与显式迁移 | G1 | done | M2 | 用户可选择后端/路径并显式复制数据，失败不破坏源数据 |
| M4 双后端集成验证 | G1 | done | M3 | CRUD、重启恢复、损坏文件、并发写和回退路径有自动化证据 |
| S1 SSH 主机信任与凭据加固 | G2 | todo | M1 | 主机指纹不再无条件接受，凭据不再依赖普通明文持久化 |
| Q1 测试、版本与文档一致性 | G3 | todo | M2 | 核心 Rust/前端测试建立，版本和文档漂移清理，大 chunk 有明确处理结论 |
| R1 跨平台发布就绪 | G4 | todo | M4, S1, Q1 | 三平台构建和安装验证完成，发布证据可追溯 |

## Current Priority

1. G1 已完成并达到 `validated-local`；不把未执行的发布、安装包或真实 SSH 验证计入完成证据。
2. 下一优先级是 G2 主机信任与凭据安全加固；需单独规格和授权后启动。
3. G3/G4 继续保留为测试治理与跨平台发布就绪，不在本次 G1 中展开。

## Explicit Boundaries

- SQLite 保持默认后端，避免升级后改变现有用户行为。
- 不自动迁移或删除现有数据库；迁移必须显式、可预览、可回退。
- YAML 默认不保存密码或私钥内容；密钥路径可作为普通连接资料保存。
- 本 roadmap 不授权真实 SSH 连接、真实凭据读取、发布、push 或 release。
