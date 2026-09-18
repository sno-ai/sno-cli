# Codex Adversarial Review

Target: src/assemble.rs, src/manifest.rs, src/harness_slots.rs, src/cli.rs, src/doctor.rs, src/lib.rs
Verdict: needs-attention

暂不建议交付。按首次安装、原样重跑、升级、卸载和崩溃恢复推演，发现以下交付阻断问题。均属于本次新增代码。结论仅来自内联源码，未验证真实发布物或实时通知。

Findings:
- [high] [BIN-A] 缺少核心程序仍可报告安装成功 (src/assemble.rs:216-223, confidence 0.97)
  Trigger: 发布列表包含 Reach，但缺少另外两个核心程序中的任一个。解析器仅在 Reach 缺失时报错，安装阶段也只强制检查 Reach。
  Impact: 安装可以返回成功，却没有交付约定的三个核心程序。依赖缺失程序的技能被跳过，机器长期处于不完整状态。
  Recommendation: 在规划写入前，要求程序集合恰好包含三个 PROGRAM_IDS；任何缺失都应报源错误。

- [high] [BIN-A] 常见权限设置使后续更新和卸载全部被拒绝 (src/manifest.rs:145-159, confidence 1.00)
  Trigger: 用户以 `umask 077` 安装包含 `0755` 程序或 `0644` 文件的正常发布包。`options.mode(mode)` 仍受 umask 限制，实际文件成为 `0700` 或 `0600`，清单却记录原权限。
  Impact: 首次安装报告成功。下一次安装、更新或卸载会把这些文件判断为用户改动并拒绝操作。若首次安装中途失败，权限差异也会阻止回滚。
  Recommendation: 写入临时文件后，显式设置清单要求的权限，再同步并重命名。

- [high] [BIN-A] 卸载提交后崩溃会永久丢失目录清理记录 (src/manifest.rs:453-458, confidence 0.99)
  Trigger: 卸载已删除清单，但在清理旧目录前进程退出。下一次调用进入恢复流程；恢复认定事务已提交，删除日志，却没有清理 previous.directories。
  Impact: 空技能目录留下，但所有权记录消失。再次安装会将这些目录判为未拥有的目标并拒绝；再次卸载也无法清理，需要人工处理。
  Recommendation: 将已提交卸载的目录清理放进 recover，并在清理完成后才删除事务日志。

- [high] [BIN-A] 日志容量小于允许安装的文件容量 (src/manifest.rs:390-395, confidence 0.98)
  Trigger: 首次安装的普通文件合计超过 16 MiB，仍低于允许的解包上限。每份内容转为十六进制后，同时写入 next.files 和 operations.after，日志超过 64 MiB。升级保留全部旧版本，也会逐步触及限制。
  Impact: 下载和校验通过后，安装或更新必然失败。已有安装也可能随着版本累计而无法继续更新。实际发布物大小未提供；上述阈值直接来自序列化结构。
  Recommendation: 将文件正文存为独立快照，让清单和日志引用快照；更新时清理不再保留的旧版本。

Next steps:
- 修复上述四处，并仅验证缺失程序、严格 umask、卸载提交后中断，以及跨越日志容量阈值的安装和更新。