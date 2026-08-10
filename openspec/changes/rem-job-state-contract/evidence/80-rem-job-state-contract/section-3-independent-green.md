# Section 3 independent GREEN index

This index records existing independent GREEN evidence. It does not report a
new test run.

| Gate | Frozen test or harness SHA-256 | Existing GREEN evidence | Output anchor |
|---|---|---|---|
| QCG-5 | `a66676a91fd42c2d8707ccdb419ee71a42329509a002b298fd9c872e8d52dbec` (`tests/rem-status-exit-code-matrix.sh`); `0661e72b74ac8fcb85f6b705093bacff4df71da62b7d0c412a98196a3ee21b0a` (`tests/rem-status-exit-code-matrix.mts`) | `evidence/80-rem-job-state-contract/qcg-5.txt` | `QCG-5 PASS` |
| QCG-8 | `8e8296590d48d9af57184e1c08c4480e114b26a1a98f5fd2c0b377c6af158f01` (`tests/rem_job_state_contract.rs`) | `evidence/80-rem-job-state-contract/qcg-8.txt` | `test qcg_8_waiting_and_nonwaiting_known_states_succeed ... ok`; reused proof: `test rem_one_shot_status_reads_running_then_stable_done ... ok` |
| QCG-9 | `8e8296590d48d9af57184e1c08c4480e114b26a1a98f5fd2c0b377c6af158f01` (`tests/rem_job_state_contract.rs`) | `evidence/80-rem-job-state-contract/qcg-9.txt` | `test qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture ... ok` |
| QCG-10 | `8e8296590d48d9af57184e1c08c4480e114b26a1a98f5fd2c0b377c6af158f01` (`tests/rem_job_state_contract.rs`) | `evidence/80-rem-job-state-contract/qcg-10.txt` | `test qcg_10_invalid_responses_are_distinct_from_unfamiliar_states ... ok` |
| QCG-11 | `8e8296590d48d9af57184e1c08c4480e114b26a1a98f5fd2c0b377c6af158f01` (`tests/rem_job_state_contract.rs`) | `evidence/80-rem-job-state-contract/qcg-11.txt` | `test qcg_11_failed_job_preserves_only_the_supplied_sidecar_sentinel ... ok` |

Recorded evidence-index inspection:

```text
$ sha256sum tests/rem-status-exit-code-matrix.sh tests/rem-status-exit-code-matrix.mts tests/rem_job_state_contract.rs && rg -n 'QCG-5 PASS|test qcg_8_waiting_and_nonwaiting_known_states_succeed \.\.\. ok|test rem_one_shot_status_reads_running_then_stable_done \.\.\. ok|test qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture \.\.\. ok|test qcg_10_invalid_responses_are_distinct_from_unfamiliar_states \.\.\. ok|test qcg_11_failed_job_preserves_only_the_supplied_sidecar_sentinel \.\.\. ok' openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-{5,8,9,10,11}.txt
status=0
a66676a91fd42c2d8707ccdb419ee71a42329509a002b298fd9c872e8d52dbec  tests/rem-status-exit-code-matrix.sh
0661e72b74ac8fcb85f6b705093bacff4df71da62b7d0c412a98196a3ee21b0a  tests/rem-status-exit-code-matrix.mts
8e8296590d48d9af57184e1c08c4480e114b26a1a98f5fd2c0b377c6af158f01  tests/rem_job_state_contract.rs
openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-11.txt:42:test qcg_11_failed_job_preserves_only_the_supplied_sidecar_sentinel ... ok
openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-5.txt:67:QCG-5 PASS
openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-10.txt:55:test qcg_10_invalid_responses_are_distinct_from_unfamiliar_states ... ok
openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-9.txt:52:test qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture ... ok
openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-8.txt:42:test qcg_8_waiting_and_nonwaiting_known_states_succeed ... ok
openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/qcg-8.txt:55:test rem_one_shot_status_reads_running_then_stable_done ... ok
```
