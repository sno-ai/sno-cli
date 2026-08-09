# Codex Adversarial Plan Review

Target: `openspec/changes/rem-job-state-contract/{tasks.md,proposal.md,design.md,specs/rem-job-state-contract/spec.md,evidence/PROBE-RESULTS-owner-r6-alignment.md,evidence/review-round-2-open-findings.md}` and `/home/lh/code/sno-station-core-edge-rem-wave/ai-doc/ACTIVE/PRD/[IMP]-edge-rem/80-rem-job-state-contract-prd.md`

Verdict: approve

Section 2 may proceed. The OpenSpec plan matches owner ruling R6 on every requested boundary. It neither revives QCG-18 nor adds noop-owned exit or trace obligations.

Load-bearing claims:
- [VERIFIED] The acceptance ledger is exactly QCG-1 through QCG-17, with no QCG-18 (`openspec/changes/rem-job-state-contract/evidence/PROBE-RESULTS-owner-r6-alignment.md:5-26)
- [VERIFIED] QCG-12 exercises all ten tool exit codes and the unknown-code control through both `run_rem.sh` and `run_rem_noop.sh` (`openspec/changes/rem-job-state-contract/tasks.md:51-51`; `openspec/changes/rem-job-state-contract/evidence/PROBE-RESULTS-owner-r6-alignment.md:28-30)
- [VERIFIED] Runner-owned exits and trace expansion apply only to `run_rem.sh`; the noop runner receives routing coverage but no new exit or trace guarantee (`openspec/changes/rem-job-state-contract/tasks.md:28-35`; `openspec/changes/rem-job-state-contract/tasks.md:53-55`)
- [VERIFIED] QCG-1 and QCG-3 prove semantic ownership and documentation agreement without prescribing Rust modules, types, accessors, constructors, or formatting (`openspec/changes/rem-job-state-contract/specs/rem-job-state-contract/spec.md:3-8`; `openspec/changes/rem-job-state-contract/specs/rem-job-state-contract/spec.md:21-26`)
- [VERIFIED] QCG-4 uses the released `sno account`, `sno service`, `rem-start`, and `rem-status` actors (`openspec/changes/rem-job-state-contract/tasks.md:43-43`)
- [VERIFIED] QCG-16 proves both runner-owned exits, absence of runner-originated tool-range literals, and provenance of every returned code in that range, without requiring a particular function shape (`openspec/changes/rem-job-state-contract/tasks.md:55-55`; `openspec/changes/rem-job-state-contract/specs/rem-job-state-contract/spec.md:163-168`)

No material findings.

Coverage:
- Checked: exact gate identifiers and count, both-runner QCG-12 scope, noop ownership fences, semantic-versus-layout constraints for QCG-1 and QCG-3, QCG-4 actors, and the complete QCG-16 guarantee.
- Not checkable from here: whether the planned tests will faithfully implement these semantics or pass against the real repositories; no commands were run, as required.

Next steps:
- Proceed with section 2 and preserve the frozen independent-test ownership boundary.