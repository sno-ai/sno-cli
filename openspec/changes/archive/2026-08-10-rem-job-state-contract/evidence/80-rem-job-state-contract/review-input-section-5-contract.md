# Section 5 published contract excerpts

Source: `/home/lh/code/sno-station-core-edge-rem-wave/ai-doc/ACTIVE/PRD/[IMP]-edge-rem/80-rem-job-state-contract-prd.md`

- `[QCG-12]` `[E2E]` The ordinary callers are `run_rem.sh` and `run_rem_noop.sh`, each invoked as the Memora harness invokes it, entering through the installed `sno` binary and reaching a live REM sidecar over its real socket with a real persona store behind it; each of the ten exit codes is produced one at a time by the sidecar and the binary, and each drives exactly the fate its routing-table row states in **both** runners; the negative proof is a tool build emitting an eleventh code, which makes the script exit non-zero and log that code by name, while that log line is absent from every run where all codes are known; verified by the independent test agent — REQ-15, REQ-16.
- `[QCG-13]` A run in which the tool exits `5` fails the persona, and the same run's log names the state and the version skew rather than reporting an invalid response — REQ-17.
- `[QCG-15]` The script's own trace shows `--json` still passed where it was passed before, and its decision input is the exit code, proven by a run whose message text is replaced with an unrelated string producing the identical routing outcome — REQ-19.
- `[QCG-16]` `run_rem.sh` invoked with no argument exits `20`, and invoked with an operation type it rejects exits `21`; a search of the script finds no `exit` statement producing a literal in `0`–`9`, and every path that returns such a code is one that propagated the tool's own — REQ-20.
- `[QCG-17]` A checkout in which only this document's changes have landed still validates the operation names the sibling has not yet renamed, and a checkout in which only the sibling's changes have landed still routes every exit code it knew before; the proof is both orders run against the same store — REQ-21.

Source: `/home/lh/code/sno-cli/openspec/changes/rem-job-state-contract/tasks.md`

- [ ] 5.1 After section 4 is complete, use `codex-coder` to add the same enumerated routing table to `evals/memora/scripts/run_rem.sh` and `evals/memora/scripts/run_rem_noop.sh`, apply it to each runner's `rem-start` and `rem-status`, and fail closed while logging any unmatched code.
- [ ] 5.2 Keep every existing `--json` argument and route both runners only on exit codes. In `run_rem.sh`, move its own usage and rejected-operation failures to exits `20` and `21`; do not originate `0` through `9`, and return a code in that range only by propagating the immediately captured result of a real `sno` invocation.
- [ ] 5.3 In `run_rem.sh`, leave the sibling-owned accepted names and unknown-operation message byte-identical while changing the rejected-operation exit number, then prove that this change and `rem-operation-switches` work in either landing order against the same store.
- [ ] 5.4 Have the independent test owner rerun and freeze GREEN for QCG-12, QCG-13, QCG-15, QCG-16, and QCG-17, with QCG-12 covering both runners.
