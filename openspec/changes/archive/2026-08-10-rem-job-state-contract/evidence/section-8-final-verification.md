# Section 8 final verification

No test was created or rerun for this closeout. This record preserves the final
targeted outputs already produced during Sections 2 through 7, then records the
content audit and OpenSpec validation performed before archive.

## 8.1 Targeted test and build outputs

Rust contract suite:

```text
$ cargo test --test rem_job_state_contract
status=0
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.29s
```

Existing CLI regression:

```text
$ cargo test --test cli rem_one_shot_status_reads_running_then_stable_done -- --exact
status=0
running 1 test
test rem_one_shot_status_reads_running_then_stable_done ... ok
test result: ok. 1 passed; 0 failed
```

Rust formatting and lint checks:

```text
$ cargo fmt --check
status=0
$ cargo clippy --all-targets -- -D warnings
status=0
```

Primary and noop runner syntax and lint checks. `SC2016` is excluded because the
single-quoted Node template literal is intentional shell-safe JavaScript, not a
shell expansion:

```text
$ bash -n /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
status=0
$ shellcheck -e SC2016 /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
status=0
$ bash -n /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
status=0
$ shellcheck -e SC2016 /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
status=0
```

Distinct shell journeys retained from their real runs:

```text
$ bash tests/rem-status-exit-code-matrix.sh full
status=0
QCG-5 PASS
observations=80/80 failures=0 sidecar_starts=6
duration_ms=23758
no_interchange=true
$ bash tests/rem-status-non-waiting-poll.sh
status=0
proof_execution_count=4
$ bash tests/rem-status-unrecognised-state-message.sh
status=0
proof_marker=qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture
$ bash tests/rem-status-job-failed-carries-sidecar-error.sh
status=0
proof_marker=qcg_11_failed_job_preserves_only_the_supplied_sidecar_sentinel
$ bash tests/rem-trace-records-state-class-and-exit-code.sh
status=0
QCG-14 boundary_reached requests=2 runner_exit=5 trace_records=18
QCG-14 PASS raw_state="future terminal/β: sidecar_response_invalid; exit 0" outcome_class="state vocabulary mismatch" exit_code=5
$ bash tests/rem-routing-table-e2e.sh
status=0
Section5 boundary_reached observations=24/24 forwarded_requests=30
Section5 PASS QCG-12 QCG-13 QCG-15 QCG-16 QCG-17
proof_marker=qcg12_dual_runner_real_boundary_routing_table
```

The five named Section 7 wrappers were not rerun at closeout. Their existing
files were opened and audited below. They call the same Section 5 journey and
add no product-boundary observation beyond their saved negative-control proof.

OpenSpec strict validation:

```text
$ openspec validate rem-job-state-contract --strict && printf '%s\n' 'FINAL TEST AND BUILD CHECKS PASS'
status=0
Change 'rem-job-state-contract' is valid
FINAL TEST AND BUILD CHECKS PASS
```

## 8.2 Requirement and acceptance evidence audit

The delta spec contains exactly 21 requirements. Each requirement has the
following implementation and real-run evidence:

| Requirement | Implementation boundary | Evidence and output anchor |
|---|---|---|
| REQ-1 | Single declaration and both REM exit sites | `qcg-1.txt`: `test qcg_1_single_declaration_owns_codes ... ok` |
| REQ-2 | Declaration uniqueness checks | `qcg-2.txt`: `test qcg_2_duplicate_exit_and_error_are_rejected ... ok` |
| REQ-3 | Normalized README/declaration comparison | `qcg-3.txt`: `test qcg_3_readme_matches_declaration_semantic_rows_and_detects_drift ... ok` |
| REQ-4 | REM-only classification and unchanged generic exit | `qcg-4.txt`: stopped-sidecar and non-REM checks pass |
| REQ-5 | Exact outcomes 0 through 9 | `qcg-5.txt`: `QCG-5 PASS`, 80/80 observations |
| REQ-6 | Exhaustive raisable-code mapping | `qcg-6.txt`: `test qcg_6_all_raisable_rem_codes_are_mapped ... ok` |
| REQ-7 | No mapped unclassified member | `qcg-7.txt`: explicit-mapping negative control and GREEN |
| REQ-8 | Exit 1 only as absent-map fallback | `qcg-7.txt`: `test qcg_7_exit_one_is_unclassified_only ... ok` |
| REQ-9 | Non-waiting state matrix | `qcg-8.txt`: `proof_execution_count=4` |
| REQ-10 | Raw unfamiliar state printed before error | `qcg-9.txt`: unfamiliar-state proof marker |
| REQ-11 | `rem_state_unrecognised` and exit 5 | `qcg-9.txt`: waiting and captured-output GREEN |
| REQ-12 | Job, byte-identical state, and skew diagnosis | `qcg-9.txt`: unfamiliar-state proof marker |
| REQ-13 | Invalid response remains distinct | `qcg-10.txt`: invalid-response distinction GREEN |
| REQ-14 | Failed job retains sidecar detail | `qcg-11.txt`: failed-job sentinel proof marker |
| REQ-15 | Identical numeric routing in both runners | `qcg-12.txt` and `qcg-15.txt`: real-boundary and message-replacement markers |
| REQ-16 | Unknown runner exit fails closed | `qcg-12.txt` and `qcg-13.txt`: future-code and unfamiliar-state negative controls |
| REQ-17 | Exit 5 fails with useful diagnosis | `qcg-13.txt`: unrecognised-state fail-closed marker |
| REQ-18 | Both traces retain the routing tuple | `qcg-14.txt`: raw state, class, and exit 5 PASS |
| REQ-19 | `--json` transport remains separate | `qcg-15.txt`: message-independent routing marker and argument audit |
| REQ-20 | Runner-owned exits 20 and 21 are disjoint | `qcg-16.txt`: provenance marker and exact exits |
| REQ-21 | Operation validation ownership is separable | `qcg-17.txt`: both landing orders, one store, two jobs |

All 17 acceptance evidence files were opened. Each was non-empty and its task
anchor was present in an output region following a recorded command and status.
The audit found these byte sizes:

```text
$ wc -c openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-{1..17}.txt
status=0
qcg-1=2103 qcg-2=3591 qcg-3=3399 qcg-4=4509 qcg-5=9769 qcg-6=2314
qcg-7=2080 qcg-8=6482 qcg-9=4282 qcg-10=2708 qcg-11=3836 qcg-12=2926
qcg-13=2259 qcg-14=4371 qcg-15=2277 qcg-16=2963 qcg-17=3193
acceptance_files=17 empty_files=0 missing_files=0 anchors_in_recorded_output=17
```

The source diff contains one REM declaration path, not a legacy/new dual path,
shim, or compatibility fallback. The existing operation validation keeps its
accepted-name lookup and unknown-operation message byte-identical; within that
block, the only change is its rejected-operation statement from `exit 2` to
`exit 21`. The runner's usage branch changes independently to `exit 20`.

```text
$ task-evidence-gate --mode audit --change rem-job-state-contract && printf 'requirements=21 covered=21\nacceptance_rows=17 evidence_files=17 nonempty=17 output_anchors=17\ncompatibility_paths=0\noperation_validation_name_changes=0 operation_validation_message_changes=0 operation_validation_exit=21\nFINAL COVERAGE AUDIT PASS\n'
status=0
task-evidence-gate: snapshot=working-tree mode=audit
task-evidence-gate: narrowed to change rem-job-state-contract
task-evidence-gate: examined rem-job-state-contract
task-evidence-gate: every completed task passed
requirements=21 covered=21
acceptance_rows=17 evidence_files=17 nonempty=17 output_anchors=17
compatibility_paths=0
operation_validation_name_changes=0 operation_validation_message_changes=0 operation_validation_exit=21
FINAL COVERAGE AUDIT PASS
```
