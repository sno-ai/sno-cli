# Section 4 independent GREEN — QCG-3 and QCG-4

## Frozen artifact

- Test path: `tests/rem_job_state_contract.rs`
- Frozen SHA-256: `53e87fe7138aac55aac9b26cd2801de583cbeb2be3f01b11b090e7fb2b8a26f9`
- Reviewed plan path: `evidence/80-rem-job-state-contract/test-plan-section-4.md`
- Reviewed plan SHA-256: `4f47fe90c8ca3a3e1e01dabcfc31b78fc55f64543ef17d8dbe55b956d3aabc21`
- Owner exact-actor ruling: `evidence/80-rem-job-state-contract/test-plan-section-4-owner-ruling.md`

The frozen test hash matched before and after the GREEN rerun. No test or product bytes were edited
during this verification.

## QCG-3 — README and declaration stay semantically aligned

- Test anchor: `qcg_3_readme_matches_declaration_semantic_rows_and_detects_drift`
- Source anchor: `tests/rem_job_state_contract.rs:534`
- Evidence path: `evidence/80-rem-job-state-contract/qcg-3.txt`
- RED anchor: `REQ-3 README semantic rows are invalid: missing outcome name \`success\``
- GREEN anchor: `test qcg_3_readme_matches_declaration_semantic_rows_and_detects_drift ... ok`
- Result: GREEN. The same frozen test compares normalized semantic rows and retains the
  documentation-only mutation control.

## QCG-4 — exact non-REM and stopped-sidecar actors

- Test anchor: `qcg_4_unreachable_account_runtime_stays_exit_one`
- Source anchor: `tests/rem_job_state_contract.rs:564`
- GREEN anchor: `test qcg_4_unreachable_account_runtime_stays_exit_one ... ok`
- Test anchor: `qcg_4_external_service_runtime_stays_exit_one`
- Source anchor: `tests/rem_job_state_contract.rs:602`
- GREEN anchor: `test qcg_4_external_service_runtime_stays_exit_one ... ok`
- Test anchor: `qcg_4_stopped_sidecar_moves_both_rem_commands_to_exit_seven`
- Source anchor: `tests/rem_job_state_contract.rs:627`
- GREEN anchor: `test qcg_4_stopped_sidecar_moves_both_rem_commands_to_exit_seven ... ok`
- Evidence path: `evidence/80-rem-job-state-contract/qcg-4.txt`
- Result: GREEN. Unreachable account runtime and the real external service executable remain exit
  `1`; stopped-sidecar `rem-start` and `rem-status` both remain exit `7`; restored sidecar calls
  return to exit `0` before scoped temporary-state cleanup.

## Exact focused commands

```text
$ cargo test --test rem_job_state_contract qcg_3_readme_matches_declaration_semantic_rows_and_detects_drift -- --exact
status=0
test qcg_3_readme_matches_declaration_semantic_rows_and_detects_drift ... ok

$ cargo test --test rem_job_state_contract qcg_4_unreachable_account_runtime_stays_exit_one -- --exact
status=0
test qcg_4_unreachable_account_runtime_stays_exit_one ... ok

$ cargo test --test rem_job_state_contract qcg_4_external_service_runtime_stays_exit_one -- --exact
status=0
test qcg_4_external_service_runtime_stays_exit_one ... ok

$ cargo test --test rem_job_state_contract qcg_4_stopped_sidecar_moves_both_rem_commands_to_exit_seven -- --exact
status=0
test qcg_4_stopped_sidecar_moves_both_rem_commands_to_exit_seven ... ok
```
