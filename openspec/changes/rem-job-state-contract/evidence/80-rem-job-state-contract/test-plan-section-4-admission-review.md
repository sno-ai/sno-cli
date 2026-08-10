# Codex Adversarial Plan Review

Target: `openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-4.md`, `openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-4.sha256`, `openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/PROBE-RESULTS-section-4.md`
Verdict: needs-attention

不要按当前计划编辑测试。QCG-3 通过准入；QCG-4 的停止 sidecar 部分应保留，但两个非 REM 场景已被现有测试覆盖，不应复制进新的四角色测试。

Load-bearing claims:
- [VERIFIED] 收据将最终计划标识为 SHA-256 `4f47fe90c8ca3a3e1e01dabcfc31b78fc55f64543ef17d8dbe55b956d3aabc21` (`openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-4.sha256:1-1`)
- [VERIFIED] README 仍把失败和超时写成退出 `1`，而声明已有十行发布契约，QCG-3 会产生真实 RED (`openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/PROBE-RESULTS-section-4.md:25-27`)
- [VERIFIED] QCG-4 的四个调用路径均可真实到达，但现有测试已分别覆盖两个非 REM 机制 (`openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/PROBE-RESULTS-section-4.md:42-49`)
- [VERIFIED] 现有编译二进制、临时目录和 loopback 服务足够执行计划，无需新增依赖；核心断言使用真实进程、socket 和磁盘状态，空 mock inventory 成立 (`openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/PROBE-RESULTS-section-4.md:60-61`)

Test plan admission:
- Reviewed plan SHA-256: `4f47fe90c8ca3a3e1e01dabcfc31b78fc55f64543ef17d8dbe55b956d3aabc21`
- [ADMIT] QCG-3: 文档修复可直接令当前 RED 转 GREEN；文档与声明独立落地是现实漂移路径；源码契约测试是最低充分层；旧测试只锁定过时句子，未覆盖十行语义对应；无需新依赖或 mock，命令可行。
- [REJECT] QCG-4: 停止 sidecar 后两个 REM 命令退出 `7` 的新证明具有因果关系、现实可达，并且编译二进制层合适；但账户 HTTP 失败和外部命令退出保留已由同层现有测试证明。把四者复制到一个新测试并未增加独立 oracle，因而不满足“现有证明不重复或涵盖”的准入条件。执行条件本身可行。

Findings:
- [severity: high] QCG-4 把必要的新证明和重复证明绑成一行 (`openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-4.md:19-19`, confidence 0.98)
  “四个角色必须在同一测试中”不是额外行为保证；分别断言四个精确退出码已经足以证明边界。现有测试覆盖两个非 REM 角色，因此新建同层断言违反逐行准入规则。
  Recommendation: 将新测试缩到两个停止 sidecar 的 REM 命令；QCG-4 证据改为重跑并引用两个现有非 REM 测试，再加这个新测试。随后重新生成计划哈希并重新审查。

Coverage:
- Checked: 两行全部逐行审查；因果相关性、现实可达性、最低证明层、重复与涵盖、命令及依赖可行性、空 mock inventory、哈希收据。
- Not checkable from here: 未重新计算内联文本的 SHA-256，也未检查探针未展示的实现、完整需求正文或现有测试源码。

Next steps:
- 修订 QCG-4、重新生成 `.sha256` 收据，再提交独立准入；在此之前不要冻结或编辑测试。