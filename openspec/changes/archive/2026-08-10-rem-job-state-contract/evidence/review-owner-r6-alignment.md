# Codex Adversarial Plan Review

Target: `/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/{proposal.md,design.md,specs/rem-job-state-contract/spec.md,tasks.md,evidence/review-round-2-open-findings.md,evidence/PROBE-RESULTS-owner-r6-alignment.md}` and `/home/lh/code/sno-station-core-edge-rem-wave/ai-doc/ACTIVE/PRD/[IMP]-edge-rem/80-rem-job-state-contract-prd.md`
Verdict: needs-attention

不要执行。R6 的账本、双 runner 归属和错误映射已经对齐，但最终计划没有哈希回执；调用方、状态词汇和 trace 基线缺少内联探针；数个验收项不能按当前写法证明其保证，其中 QCG-12 还把 shell 路由验证塞进了无法稳定制造全部结果的真实 E2E 边界。

Load-bearing claims:
- [VERIFIED] 发布账本只有 QCG-1 至 QCG-17，变更工件中没有 QCG-18（`evidence/PROBE-RESULTS-owner-r6-alignment.md:5-26`）
- [VERIFIED] QCG-12 明确覆盖 `run_rem.sh` 和 `run_rem_noop.sh`，包括十个 tool exits 和未知码负控（`evidence/PROBE-RESULTS-owner-r6-alignment.md:28-30`）
- [VERIFIED] 十个 outcome class 的 exit/error-code membership 与发布 PRD 精确一致（`evidence/PROBE-RESULTS-owner-r6-alignment.md:32-49`）
- [VERIFIED] 没有给 noop runner 发明自有 exit 或 trace 保证；REQ-18 和 REQ-20 只绑定 CLI/`run_rem.sh`（`specs/rem-job-state-contract/spec.md:149-168`）
- [VERIFIED] Section 2 的文本顺序要求独立测试所有者先冻结 RED，再改产品，最后重跑 GREEN（`tasks.md:3-11`）
- [UNVERIFIED] sidecar 的权威状态集合恰好是 `queued/running/done/failed`（`design.md:57-67`）
  需提供权威状态类型的源码摘录，以及该仓库所有状态写入点的完整 `rg` 命令和输出，证明不存在第五个正常发出值。
- [UNVERIFIED] 两个 Memora runner 是完整的 numeric-contract 消费者集合，其他调用方不读取退出码（`proposal.md:11-12`）
  需保存并内联以下搜索的完整逐命中分类：`rg -n --hidden --glob '!.git/**' --glob '!target/**' 'sno station rem-(start|status)|run_rem(_noop)?\.sh|deploy-mem-claw\.sh|\$\?|PIPESTATUS' /home/lh/code/sno-cli /home/lh/code/sno-station-core-edge-rem-wave`。
- [UNVERIFIED] 现有 CLI 和 `run_rem.sh` trace 都有可扩展的最终写入点（`design.md:79-81`）
  需提供两个实际写入点、当前 JSONL schema 和一条真实 trace 样本。
- [UNVERIFIED] 按 error code 在两个通用 process-conversion sites 查表不会改变任何非 REM 命令（`design.md:32-34`, `specs/rem-job-state-contract/spec.md:28-33`）
  需运行并分类 `rg -n -A1 'CliError::runtime\(' /home/lh/code/sno-cli/src`，证明映射表中的错误码没有非 REM raise site，或明确加入 REM-family discriminator。

Test plan admission:
- Reviewed plan SHA-256: absent — inlined files contain no plan-hash receipt; plan rejected
- [REJECT] QCG-1: 单一声明可被改动打红，unit/compile 是正确层级且无既有证明；但源码字符串搜索不能证明所有运行时 raise/exit 都经过声明，也不具备 shape independence。
- [ADMIT] QCG-2: declaration collision 会直接由本变更转红；真实维护改动可触达；unit negative controls 是最低层级；没有重复证明；执行可行。
- [REJECT] QCG-3: README drift 可由本变更触发且 unit 层合适；但计划没有锁定“由声明生成”或“解析后语义比较”，文本扫描会把排版当合同。
- [REJECT] QCG-4: 非 REM 稳定性和 REM family-wide 分类可转红且可触达；CLI integration 层正确；但 `sno account`、`sno service` 的具体 failing subcommands 和 sidecar 状态切换未定义，oracle 可能只测到 usage exit。
- [ADMIT] QCG-5: 每个 CLI outcome 的 code/error 配对由变更直接控制；故障均属受支持路径；CLI integration 是最低充分边界；无重复；计划给出了可观测 oracle。
- [ADMIT] QCG-6: 新 REM raise site 可使测试转红；维护路径现实；unit mutation control 足以证明 mapping completeness；无重复；可执行。
- [ADMIT] QCG-7: unclassified fallback 由声明变化直接控制；运行时未知失败可达；unit 层充分；无重复；可执行。
- [ADMIT] QCG-8: 普通非等待轮询是主路径；状态与 exit 是正确 oracle；CLI integration 层充分；无重复；可执行。
- [ADMIT] QCG-9: 未识别状态的等待、输出顺序和消息由变更直接控制；版本偏差现实；command-substitution boundary 是最低充分层；无重复。
- [ADMIT] QCG-10: malformed 与 unfamiliar 分类由改动直接控制；坏响应可达；parser/unit 层充分；无重复；可执行。
- [ADMIT] QCG-11: sidecar error preservation 可因改动回归；失败 job 可达；CLI integration 层充分；无重复；可执行。
- [REJECT] QCG-12: runner 路由可转红且调用现实，但 shell router 的最低证明层是受控 fake `sno`；要求真实 binary/live sidecar/store 同时制造 usage、stopped-sidecar、profile、unclassified 和未知第十一码，没有定义可执行注入路径。
- [ADMIT] QCG-13: code `5` 到 persona failure 和有用日志是独立 caller guarantee；真实版本偏差可达；runner integration 层充分；无既有完整证明。
- [ADMIT] QCG-14: 双 trace tuple 是变更独有保证；未识别状态可达；跨 CLI/runner trace integration 是最低充分层；无重复。
- [ADMIT] QCG-15: 更换消息但保持 exit 可直接证明不按 prose 路由；真实文案变化可达；runner integration 层充分；无重复。
- [REJECT] QCG-16: exits `20/21` 可直接测试，但“所有 `0`–`9` 都来自 tool”是 provenance 保证；搜 literal `exit` 加有限 sweep 无法排除变量、函数返回或算术生成的 runner-owned code。
- [ADMIT] QCG-17: 两种落地顺序可由任一变更打红；跨仓独立发布现实；双 checkout integration 是最低充分证明；无重复；可执行。

Findings:
- [severity: high] 缺少最终计划哈希回执（`tasks.md:38-56`, confidence 1.00）
  无法证明本次逐行 admission 审查对应最终 QCG-1..17，而不是此前或随后修改的版本；按给定审查合同必须拒绝。
  Recommendation: 对最终计划工件生成 SHA-256 回执，内联命令、文件集合与结果，然后重新进行逐行 admission。

- [severity: high] “已冻结”的三项现实基线没有探针证据（`tasks.md:3-5`, confidence 0.99）
  完整 caller 集合、四状态词汇和既有 trace 写入点都决定修改范围及实现可行性，但内联的 PROBE 文件没有这些命令或结果。
  Recommendation: 将 caller 搜索、权威状态声明及 emit-site 搜索、CLI/runner trace 写入点与样本加入一个 `PROBE-RESULTS` 文件后再冻结 section 1。

- [severity: high] 单一查表没有 REM 范围判定（`design.md:32-34`, `specs/rem-job-state-contract/spec.md:28-33`, confidence 0.96）
  两个通用 error-to-process sites 只拿到 error，而声明只有 `name/exit_code/error_codes`。字面实现会按 error code 全局映射；计划既未证明映射码不被非 REM 使用，也未提供 REM discriminator，因此“非 REM 仍为 1”没有结构性保证。
  Recommendation: 明确让 REM command boundary 附加 typed outcome/class，再由通用出口读取；同时枚举所有 mapped code 的非 REM raise sites，并让 exhaustive test 覆盖它们。

- [severity: high] QCG-12 的真实 E2E 无法按声明输入稳定执行（`tasks.md:51-51`, confidence 0.99）
  普通 runner 通过安装的 binary 和 live sidecar，不能在同一已定义边界自然制造本地 usage、坏 profile、stopped sidecar、unclassified failure 和未知第十一码；即使另造测试 build，也不再是所声明的安装 binary。该门会在最后才因缺少 fault injection 卡死。
  Recommendation: 用受控 fake `sno` 在 runner integration 层穷举 `0`–`9` 和未知码；真实 E2E 只保留可由真实 sidecar/store 产生的普通成功、job failure 和版本偏差，并逐项写明故障注入。

- [severity: high] QCG-1、QCG-3、QCG-16 的 oracle 没有执行语义约束（`tasks.md:40-42`, `tasks.md:55-55`, confidence 0.97）
  搜索整数、字符串或 literal `exit` 可以被别名、变量、算术和格式变化绕过，也可能因无行为变化的重排误红；它们不能证明 sole-source、README 语义一致或 exit provenance。
  Recommendation: QCG-1 用 typed outcome API 和不可绕过的 REM error constructor；QCG-3 从声明生成表或比较规范化语义 rows；QCG-16 集中所有 runner exits 到一个带来源标签的函数，并对每个 runner-owned/tool-propagated path 做运行时注入。

- [severity: high] QCG-4 没有定义能够到达目标 runtime failure 的具体调用（`tasks.md:43-43`, confidence 0.98）
  “failing `sno account` and `sno service` paths”允许执行者选到参数错误、认证错误或其他非目标分支，从而得到错误的 exit，却仍声称验证了 generic runtime stability。
  Recommendation: 写出两个完整命令、fixture/profile 前置条件、期望 error code 和 exit `1`；另写清 live sidecar 到 stopped-sidecar 的切换及恢复步骤。

Coverage:
- Checked: R6 gate count；QCG-12 双 runner；noop own-exit/trace 边界；完整 error mapping；QCG-1/QCG-3 的 shape independence；QCG-4 actor 可执行性；QCG-16 provenance 完整性；section-2 RED/GREEN 顺序；普通 runner 调用；每个 QCG row 的 admission；plan→evidence 与 evidence→plan 两个方向。
- Not checkable from here: 实际 Rust/shell 源码、真实 sidecar fault injection、trace schema、仓库外调用方、部署/包发布记录、测试运行时与成本、最终计划 SHA-256。

Next steps:
- 先补最终计划哈希和三项缺失探针；再重写 QCG-1、QCG-3、QCG-4、QCG-12、QCG-16 的 proof boundary 与 oracle，重新提交独立审查。
