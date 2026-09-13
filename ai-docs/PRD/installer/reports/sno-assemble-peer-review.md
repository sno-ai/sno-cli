# Codex Adversarial Plan Review

Target: ai-docs/PRD/installer/sno-assemble-PRD.md
Verdict: needs-attention

不要按当前版本执行。计划会漏装技能所需的命令，并可能在失败后留下无法清除的安装。已有技能的所有权也没有保护。部分验收要求与提供的环境证据直接冲突。

Load-bearing claims:
- [VERIFIED] Reach 的公开安装路径、入口和配置文件名称已有命名契约。 (ai-docs/PRD/installer/sno-assemble-PRD.md:78-78)
- [VERIFIED] 技能发布契约定义了 `requires.programs`、`requires.harness` 和三种 `need`。这证明契约存在，不证明发布物已经存在。 (ai-docs/PRD/installer/sno-assemble-PRD.md:79-79)
- [CONTRADICTED] Hermes 缺少 `2.pre-turn-context-injection`，因此应降级安装。 (ai-docs/PRD/installer/sno-assemble-PRD.md:211-211)
  提供的能力表明确标记 Hermes 支持此能力，并给出 `pre_llm_call` 的输入与输出。
- [CONTRADICTED] 安装所有 S 类技能只需复制技能文本。 (ai-docs/PRD/installer/sno-assemble-PRD.md:92-92)
  提供的技能发布契约明确要求安装器放置 `heartbeat` 和 `subscription-quota-check` 的 `public-bin` 命令。
- [CONTRADICTED] 最后写清单即可保证崩溃后清单与文件一致。 (ai-docs/PRD/installer/sno-assemble-PRD.md:120-120)
  本计划自己的失败规则要求保留已写入的新文件，同时保留旧清单或不写清单。
- [UNVERIFIED] 两个仓库已有可消费的发布物，并满足指定归档布局。 (ai-docs/PRD/installer/sno-assemble-PRD.md:44-44)
  需要两仓库的 `gh release view <tag> --repo sno-ai/<repo> --json tagName,assets` 输出，以及实际下载、校验和解包目录清单。现有探针只证明本地 Reach 目录不存在、技能目录只有 README；不能据此判定远端发布状态。
- [UNVERIFIED] 所列路径和钩子配置能在目标机器的实际运行配置下生效。 (ai-docs/PRD/installer/sno-assemble-PRD.md:80-82)
  需要记录各运行实例的配置根目录、技能发现结果，以及一次真实提示触发后收到的提醒内容；仅检查文件存在不能证明生效。

Findings:
- [severity: critical] 技能的可执行命令没有安装步骤 (ai-docs/PRD/installer/sno-assemble-PRD.md:117-117, confidence 1.00)
  上游契约把两个 `public-bin` 命令交给安装器，但本计划只要求复制技能文本。普通用户读到已安装的技能后执行其命令，会遇到命令不存在；当前座位验收不会发现此缺失。
  Recommendation: 将两个命令纳入安装、更新、清单、检查和删除，并在隔离 PATH 中执行安装后的命令。

- [severity: critical] Hermes 的验收要求与能力证据相反 (ai-docs/PRD/installer/sno-assemble-PRD.md:211-211, confidence 1.00)
  能力表明确支持该输入能力，验收却要求报告缺失。执行者必须写错能力表或写错测试，才能让此行通过。
  Recommendation: 按证据修正 Hermes 的结果。需要降级测试时，使用明确缺少该能力的测试输入。

- [severity: critical] 失败后清单失真，删除无法收尾 (ai-docs/PRD/installer/sno-assemble-PRD.md:120-120, confidence 1.00)
  首次安装在钩子写入失败后保留 Reach，却没有清单，因此 `remove` 拒绝执行；更新失败则保留旧清单和新文件。第 143 行和第 214 行不仅没有补救，反而把这种状态规定为正确结果。
  Recommendation: 在修改前记录可恢复的操作和所有权，失败时回滚或完成恢复。验收必须覆盖失败后的 `doctor`、重试和 `remove`，证明文件与记录重新一致。

- [severity: critical] 现有技能可能被覆盖后整目录删除 (ai-docs/PRD/installer/sno-assemble-PRD.md:117-123, confidence 0.99)
  普通安装目标已经有技能，但计划没有规定同名目录、文件或链接的冲突处理。执行者可以覆盖现有目录，再把它记成自己安装的目录；随后 `remove` 删除用户原有内容。第 217 行只保护旁边的无关文件，没有保护被安装目录内的原有文件。
  Recommendation: 写入前区分已有内容与本工具拥有的内容。对未拥有的同名目标拒绝覆盖；删除只处理可证明归本工具所有的内容，并验证安装前已有的目录及文件保持完整。

- [severity: high] 上游缺失时指定的替代步骤无法执行 (ai-docs/PRD/installer/sno-assemble-PRD.md:248-248, confidence 1.00)
  探针显示本地 `apps/reach` 不存在，两个上游项目均标记为未开始。计划却要求“没有发布物就从 `make install` 输出打包”，没有提供产生该输出的现成输入；本地自造夹具也不能证明真实发布物可安装。
  Recommendation: 将实际发布物及其校验结果设为发布验收前置条件。夹具可支持本地实现，但不得替代真实发布验收；上游未就绪时明确记录阻塞。

- [severity: high] 固定目录会把技能装到运行实例不读取的位置 (ai-docs/PRD/installer/sno-assemble-PRD.md:42-42, confidence 0.99)
  提供的证据明确指出，自定义 `OPENCLAW_STATE_DIR` 时默认 home 技能目录可能不参与发现。计划仍只认固定目录和 PATH，会报告安装成功，但实际会话没有技能；共享目录也不是独立运行环境，不能直接套用某一列能力。
  Recommendation: 从实际实例配置确定技能目标，明确共享目录的能力判定，并合并指向同一目录的链接目标。验收应由目标实例实际发现技能。

- [severity: high] 钩子存在不等于提醒能够执行 (ai-docs/PRD/installer/sno-assemble-PRD.md:119-119, confidence 0.98)
  计划只检查条目和提示信任操作。普通会话可能没有 `SNO_REACH_ADDR`，Codex 也可能尚未信任钩子；给出的包装命令会吞掉执行错误。座位收发消息成功并不能证明提示钩子运行过。
  Recommendation: 定义地址未设置时的行为和信任未完成时的检查结果。增加一次真实提示触发，确认指定座位的提醒进入上下文；缺少前置条件时不得报告钩子 `ok`。

- [severity: high] 安装完成后要求六个检查全绿无法由安装步骤保证 (ai-docs/PRD/installer/sno-assemble-PRD.md:216-216, confidence 0.99)
  自动更新默认没有开启，而计划明确把关闭状态报告为 `timer missing`。现有 station 检查还会把未初始化的身份和缓冲区报告为 `warn`；安装步骤没有创建它们，却要求“全部 assembled”后六节都是 `ok`。
  Recommendation: 分开定义未启用、未初始化和实际故障。保留原有 station 语义，并为全绿验收列出实际需要的初始化步骤；不要为了验收把缺失状态改报成功。

Coverage:
- Checked: 安装、重复安装、每日更新、写入失败、检查、删除和干净账户使用路径；发布依赖、技能载荷、能力判定、文件所有权、钩子执行、QCG-1 至 QCG-10；同时检查计划与探针的双向遗漏。
- Not checkable from here: 远端发布状态、真实归档内容、完整能力表、实际钩子执行与信任状态、调度器运行环境。未执行命令；提供的探针也没有证明这些外部行为。

Next steps:
- 先补齐命令安装和文件所有权规则。
- 修复失败恢复流程，再修正冲突的验收要求。
- 取得真实发布物和实际运行证据后，重新审查。