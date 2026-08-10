# Codex Adversarial Plan Review

Target: `tests/rem-trace-records-state-class-and-exit-code.plan.md` and its SHA-256 receipt, reviewed against released PRD REQ-18/QCG-14, tasks 6.1/6.2, and `PROBE-RESULTS-section-6-qcg-14.md`  
Verdict: approve

可以执行。QCG-14 与需求完全因果相关，走普通 runner 边界，覆盖两端真实 trace 写入；现有测试没有重复或涵盖它。回环服务仅提供协议输入，没有替换 runner、CLI 或 trace 行为。

Load-bearing claims:
- [VERIFIED] 普通 `run_rem.sh` 可使用当前编译 CLI、临时目录和回环协议服务完成 start/status 流程，无需修改受测系统 (`tests/rem-trace-records-state-class-and-exit-code.plan.md:10-16`)
- [VERIFIED] 当前 CLI 已能把陌生非空状态归类并退出 `5`，但两个 trace 均缺少完整结构化 tuple，因此计划具有明确的 RED→GREEN 因果路径 (`tests/rem-trace-records-state-class-and-exit-code.plan.md:22-22`)
- [VERIFIED] 现有覆盖只证明陌生状态分类或成功 trace 片段，没有通过普通 runner 同时断言两个 trace tuple (`openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/PROBE-RESULTS-section-6-qcg-14.md:83-89`)
- [VERIFIED] 该行直接对应 CLI 与 `run_rem.sh` 两端记录 tuple，并由独立测试所有者冻结和复跑 (`openspec/changes/rem-job-state-contract/tasks.md:33-36`)
- [VERIFIED] 最终计划存在精确字节对应的 SHA-256 receipt (`tests/rem-trace-records-state-class-and-exit-code.plan.sha256:1-1`)

Test plan admission:
- Reviewed plan SHA-256: `27536c728d7733338ac557531f1ec7b58f8f02db180cc835dd313655a2092a48`
- [ADMIT] QCG-14: 产品 trace 字段变更能直接让当前缺字段 RED 转为 GREEN；陌生状态能经真实 CLI 和普通 runner 到达；跨进程集成是同时证明两端持久化的最低充分边界；现有测试不重复；命令、依赖、运行成本均可行。Mock Inventory 可接受：回环服务是已披露的协议输入 fixture，受断言的 runner、CLI 和文件写入均为真实实现。

No material findings.

Coverage:
- Checked: 单行范围准入、因果关系、普通路径、可达性、最低证明层、重复覆盖、执行可行性、freeze 条件、Mock Inventory 和 receipt。
- Not checkable from here: 尚未创建的 helper/test 内容、实际 RED 输出、超时与清理实现，以及后续 GREEN 结果；本次按要求未执行命令。

Next steps:
- 按 receipt 冻结该行，创建测试与 fixture，然后运行 `bash tests/rem-trace-records-state-class-and-exit-code.sh`；只有在请求到达、退出码为 `5`、两个 JSONL 可解析且失败仅指向缺失 tuple 时，才接受 RED。