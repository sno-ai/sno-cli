# Section 5 test-plan review technical blocker

## Frozen plan

```text
2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69  openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.md
```

The separate receipt contains the same hash. The plan bytes did not change between reviews.

## First bounded review

Exact command:

```bash
REVIEW_KIND=plan FOCUS='Independent row-by-row Test Scope Admission for Section 5 only. Return ADMIT or REJECT for every QCG-12, QCG-13, QCG-15, QCG-16, and QCG-17 row; apply causal relevance, realistic reachability, lowest sufficient layer, duplication/subsumption, execution feasibility, real-boundary and no-mock rules. Copy the exact plan SHA-256 from the receipt. Pay special attention to whether the post-forward fault injector and temporary future-code CLI build preserve the published E2E boundary, whether QCG-16 provenance oracle is complete, and whether QCG-17 truly proves both landing orders against one store. Do not propose or execute tests.' REVIEW_OUTPUT_FILE='/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5-admission-review.md' PROGRESS_SECS=30 STALL_SECS=180 TIMEOUT_SECS=180 MAX_RETRIES=0 bash /home/lh/.dotfiles/codex/skills/codex-reviewer/scripts/run-adversarial-review.sh '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.md' '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.sha256' '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/PROBE-RESULTS-section-5.md' '/home/lh/code/sno-station-core-edge-rem-wave/ai-doc/ACTIVE/PRD/[IMP]-edge-rem/80-rem-job-state-contract-prd.md' '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/tasks.md'
```

Complete wrapper output:

```text
[run-adversarial-review] claimed concurrency slot 2/8
[run-adversarial-review] review: attempt 1/1…
[run-adversarial-review] running 30s, quiet 5s, output 76018 bytes
[run-adversarial-review] running 60s, quiet 0s, output 76301 bytes
[run-adversarial-review] running 90s, quiet 10s, output 76451 bytes
[run-adversarial-review] running 120s, quiet 5s, output 76692 bytes
[run-adversarial-review] running 150s, quiet 5s, output 76931 bytes
[run-adversarial-review] running 180s, quiet 0s, output 77173 bytes
[run-adversarial-review] hard timeout after 180s — killing codex (pid 4105245)
[run-adversarial-review] review: attempt 1/1 failed (watchdog, exit=124, final_bytes=0)
[run-adversarial-review] review failed after 0 retries — see stdout/stderr captures for the last attempt:
  stdout: /mnt/ramdisk/tmp/codex-review-stdout.l2QnaN
  stderr: /mnt/ramdisk/tmp/codex-review-stderr.ffDN8O
  final: /home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5-admission-review.md
  prompt: /mnt/ramdisk/tmp/codex-review-prompt.ESREsG
```

Wrapper exit: `6`. Final report bytes: `0`.

## One authorized reduced-payload review

Exact command:

```bash
REVIEW_KIND=plan FOCUS='Independent row-by-row Test Scope Admission for Section 5 only. Return ADMIT or REJECT for every QCG-12, QCG-13, QCG-15, QCG-16, and QCG-17 row; apply causal relevance, realistic reachability, lowest sufficient layer, duplication/subsumption, execution feasibility, real-boundary and no-mock rules. Copy the exact plan SHA-256 from the receipt. Pay special attention to whether the post-forward fault injector and temporary future-code CLI build preserve the published E2E boundary, whether QCG-16 provenance oracle is complete, and whether QCG-17 truly proves both landing orders against one store. Do not propose or execute tests.' REVIEW_OUTPUT_FILE='/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5-admission-review.md' PROGRESS_SECS=30 STALL_SECS=180 TIMEOUT_SECS=180 MAX_RETRIES=0 bash /home/lh/.dotfiles/codex/skills/codex-reviewer/scripts/run-adversarial-review.sh '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.md' '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.sha256' '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/PROBE-RESULTS-section-5.md' '/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/review-input-section-5-contract.md'
```

Complete wrapper output:

```text
[run-adversarial-review] claimed concurrency slot 1/8
[run-adversarial-review] review: attempt 1/1…
[run-adversarial-review] running 30s, quiet 5s, output 29180 bytes
[run-adversarial-review] running 60s, quiet 5s, output 29523 bytes
[run-adversarial-review] running 90s, quiet 20s, output 29664 bytes
[run-adversarial-review] running 120s, quiet 0s, output 30160 bytes
[run-adversarial-review] running 150s, quiet 0s, output 30495 bytes
[run-adversarial-review] running 180s, quiet 0s, output 30813 bytes
[run-adversarial-review] hard timeout after 180s — killing codex (pid 4124660)
[run-adversarial-review] review: attempt 1/1 failed (watchdog, exit=124, final_bytes=0)
[run-adversarial-review] review failed after 0 retries — see stdout/stderr captures for the last attempt:
  stdout: /mnt/ramdisk/tmp/codex-review-stdout.mXzo1K
  stderr: /mnt/ramdisk/tmp/codex-review-stderr.kfPOe3
  final: /home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5-admission-review.md
  prompt: /mnt/ramdisk/tmp/codex-review-prompt.Sed3EJ
```

Wrapper exit: `6`. Final report bytes: `0`.

## Gate consequence

The Test Writer gate requires Codex Reviewer to return `admit` or `reject` for every row before any
test, fixture, or helper is written. Neither bounded invocation produced a final report, so no row
has reviewer admission. No test-owned implementation file was created, no RED command was run, and
neither runner nor any product source was edited.

Frozen runner hashes remain:

```text
ae77cbeb852f23ae87f35cca8128d57fa3ece8456062fc6589f04493499cf084  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
e56ba30ccd0ef3488ad759febe0a069d365c1234c4dd3ad3b1ba416c2ab050d8  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
```
