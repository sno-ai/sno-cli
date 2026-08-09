## 1. Freeze the Cross-Repository Contract

- [ ] 1.1 Freeze `evidence/review-round-1-probes.md` as the caller, state-vocabulary, and trace-write baseline: `run_rem.sh` and `run_rem_noop.sh` consume exit codes, while `deploy-mem-claw.sh:649-656` consumes JSON fields and never reads the numeric contract.
- [ ] 1.2 Record the exact cross-repository ownership boundary: this change owns REM outcome classification, both runners' routing and own exits, and trace fields; `rem-operation-switches` owns which names `run_rem.sh` accepts and its validation message, while this change owns the exit-number line in that same block.
- [ ] 1.3 Before product edits, have a fresh independent test owner derive and freeze the smallest RED checks for QCG-1 through QCG-18, store each observed failure under `evidence/80-rem-job-state-contract/qcg-N.txt`, and keep product implementation context out of that test owner.

## 2. Establish the Single Outcome Declaration

- [ ] 2.1 Using `codex-coder`, add one outcome-class declaration whose entries contain `name`, `exit_code`, and `error_codes`; give success and unclassified empty lists, invalid usage exactly `usage_error`, map all thirteen existing REM error codes, and make both error-to-exit call sites in `src/service.rs` and `src/cli.rs` resolve through it.
- [ ] 2.2 Make duplicate class names, duplicate exit codes, duplicate error-code membership, unmapped raisable REM codes, a non-empty unclassified list, and named assignment of exit `1` fail separate targeted tests.
- [ ] 2.3 Have the independent test owner rerun and freeze GREEN for QCG-1, QCG-2, QCG-6, and QCG-7 without changing the product implementation.

## 3. Separate State Vocabulary from Response Validity

- [ ] 3.1 Using `codex-coder`, implement the waiting and non-waiting state matrix: every non-empty unfamiliar state immediately emits `rem_state_unrecognised` with exit `5`, while invalid JSON and an absent or empty state emit `sidecar_response_invalid` with exit `6`.
- [ ] 3.2 In both modes, print the unfamiliar raw state to stdout before the error and include the job identifier, byte-identical state, and version-skew explanation in the message, including through command substitution.
- [ ] 3.3 Preserve the sidecar-provided error string in `rem_job_failed` messages.
- [ ] 3.4 Have the independent test owner rerun and freeze GREEN for QCG-5, QCG-8, QCG-9, QCG-10, and QCG-11.

## 4. Publish and Protect the CLI Contract

- [ ] 4.1 Using `codex-coder`, assign the exact ten outcome exit codes `0` through `9` and keep non-REM generic runtime failures at exit `1`.
- [ ] 4.2 Update `README.md` with the `station rem-*` exit-code table and add a consistency check against the declaration.
- [ ] 4.3 Have the independent test owner rerun and freeze GREEN for QCG-3 and QCG-4.

## 5. Route Both Memora Consumers

- [ ] 5.1 After section 4 is complete, use `codex-coder` to add the same enumerated routing table to `evals/memora/scripts/run_rem.sh` and `evals/memora/scripts/run_rem_noop.sh`, apply it to each runner's `rem-start` and `rem-status`, and fail closed while logging any unmatched code.
- [ ] 5.2 Keep every existing `--json` argument, route only on exit codes, and move both runners' own usage and rejected-operation failures to exits `20` and `21`; each runner must only propagate, never originate, codes `0` through `9`.
- [ ] 5.3 In `run_rem.sh`, leave the sibling-owned accepted names and unknown-operation message byte-identical while changing the rejected-operation exit number, then prove that this change and `rem-operation-switches` work in either landing order against the same store.
- [ ] 5.4 Have the independent test owner rerun and freeze GREEN for QCG-13, QCG-15, QCG-16, QCG-17, and QCG-18.

## 6. Extend Existing Traces

- [ ] 6.1 Using `codex-coder`, add a final outcome row to the existing CLI and both runner traces with `raw_state`, `state_unavailable_reason`, `outcome_class`, and `exit_code`; enforce exactly one non-null state field and use the error code or `job-state-not-returned` when no decoded state exists.
- [ ] 6.2 Have the independent test owner rerun and freeze GREEN for QCG-14 across success, job failure, unfamiliar state, invalid response, and pre-connection failure, while treating an unavailable trace sink as the explicit `rem_trace_error` exception.

## 7. Run the Acceptance Gates

- [ ] 7.1 Run `cargo test outcome_class_is_sole_source` and save the actual output to `evidence/80-rem-job-state-contract/qcg-1.txt`.
- [ ] 7.2 Run `cargo test outcome_class_names_and_exits_are_unique` and `cargo test outcome_error_codes_are_unique`; save separate duplicate-name, duplicate-exit, and duplicate-error-membership negative controls to `evidence/80-rem-job-state-contract/qcg-2.txt`.
- [ ] 7.3 Run `cargo test readme_exit_code_table_matches_declaration` with a documentation-drift negative control and save the output to `evidence/80-rem-job-state-contract/qcg-3.txt`.
- [ ] 7.4 Run `cargo test non_rem_commands_keep_exit_one` against a live sidecar, covering unrelated command failures plus both REM entrypoints against a stopped sidecar, and save the output to `evidence/80-rem-job-state-contract/qcg-4.txt`.
- [ ] 7.5 Run `bash tests/rem-status-exit-code-matrix.sh` for every required outcome across ten repetitions with no interchange and save the output to `evidence/80-rem-job-state-contract/qcg-5.txt`.
- [ ] 7.6 Run `cargo test every_rem_error_code_is_mapped` with an unmapped-raise-site negative control and save the output to `evidence/80-rem-job-state-contract/qcg-6.txt`.
- [ ] 7.7 Run `cargo test exit_one_is_unclassified_only` with explicit-mapping and deliberate-fallback controls and save the output to `evidence/80-rem-job-state-contract/qcg-7.txt`.
- [ ] 7.8 Run `bash tests/rem-status-non-waiting-poll.sh` for queued, running, done, failed, and a non-empty unfamiliar state; prove the unfamiliar state prints and exits `5`, and save the output to `evidence/80-rem-job-state-contract/qcg-8.txt`.
- [ ] 7.9 Run `bash tests/rem-status-unrecognised-state-message.sh` in waiting mode, including immediate exit, command-substitution capture, and byte-identical state checks, and save the output to `evidence/80-rem-job-state-contract/qcg-9.txt`.
- [ ] 7.10 Run `cargo test response_invalid_narrowed_to_malformed` for invalid JSON, empty state, and a well-formed unfamiliar state and save the output to `evidence/80-rem-job-state-contract/qcg-10.txt`.
- [ ] 7.11 Run `bash tests/rem-status-job-failed-carries-sidecar-error.sh` with present and absent sidecar error text and save the output to `evidence/80-rem-job-state-contract/qcg-11.txt`.
- [ ] 7.12 After the end-to-end preflight passes, run `bash tests/rem-routing-table-e2e.sh` through the ordinary `run_rem.sh` caller, installed `sno` binary, live REM sidecar, and real persona store, including an unknown-code negative control, and save independently owned output to `evidence/80-rem-job-state-contract/qcg-12.txt`.
- [ ] 7.13 Run `bash tests/rem-unrecognised-state-fails-closed.sh` and save the persona failure plus useful version-skew log to `evidence/80-rem-job-state-contract/qcg-13.txt`.
- [ ] 7.14 Run `bash tests/rem-trace-records-state-class-and-exit-code.sh` for success, job failure, unfamiliar state, invalid response, and pre-connection failure; verify both components record the class and exit plus either raw state or its absence reason, and save output to `evidence/80-rem-job-state-contract/qcg-14.txt`.
- [ ] 7.15 Run `bash tests/rem-routing-ignores-message-text.sh` with a message-replacement negative control and save the output to `evidence/80-rem-job-state-contract/qcg-15.txt`.
- [ ] 7.16 Run `bash tests/rem-runner-own-exit-codes.sh` for `run_rem.sh` exits `20` and `21`, the no-originated-`0`-through-`9` sweep, and propagated tool codes; save the output to `evidence/80-rem-job-state-contract/qcg-16.txt`.
- [ ] 7.17 Run `bash tests/rem-runner-independent-landing-order.sh` for both single-change landing orders against the same store and save the output to `evidence/80-rem-job-state-contract/qcg-17.txt`.
- [ ] 7.18 Run `bash tests/rem-noop-runner-exit-contract.sh` against `run_rem_noop.sh`, covering its routing table, unknown-code fail-closed arm, own exits `20` and `21`, no originated code in `0` through `9`, and tool-code propagation; save the output to `evidence/80-rem-job-state-contract/qcg-18.txt`.

## 8. Final Verification

- [ ] 8.1 Run the targeted Rust and shell test sets, typecheck or build checks required by each repository, and `openspec validate rem-job-state-contract --strict`; record exact commands and outputs without substituting a full test-tree run.
- [ ] 8.2 Confirm all 21 requirements are covered, all 18 acceptance rows name existing evidence containing their anchored output, no acceptance row is promoted without a real run, no compatibility path was introduced, and the only edit inside sibling-owned operation validation is this change's rejected-operation exit number.
