# Section 1 contract freeze

```text
$ rg -n 'run_rem_noop\.sh|deploy-mem-claw\.sh' openspec/changes/rem-job-state-contract/proposal.md
status=0
11:- Give both exit-code consumers, `run_rem.sh` and `run_rem_noop.sh`, an enumerated routing table and fail closed on unknown codes; keep `run_rem.sh` routing independent of human-readable messages and reserve 20 and 21 for that runner's own failures.
28:- Affects the station-core Memora runners at `evals/memora/scripts/run_rem.sh` and `evals/memora/scripts/run_rem_noop.sh` and their contract tests.
30:- Does not apply the numeric routing contract to `evals/sno-memory-bench/deploy-mem-claw.sh`: the read at lines 649-656 shows that it consumes JSON fields and treats every command failure as fatal without inspecting an exit code. The complete caller probe is recorded in `evidence/review-round-1-probes.md`.
```

```text
$ rg -n '^### Requirement: REQ-(18|20)' openspec/changes/rem-job-state-contract/specs/rem-job-state-contract/spec.md
status=0
149:### Requirement: REQ-18 Both traces record the routing tuple
163:### Requirement: REQ-20 Runner-owned exits are disjoint
```
