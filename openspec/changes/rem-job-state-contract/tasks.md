## 1. Freeze the Cross-Repository Contract

- [x] 1.1 Freeze `evidence/review-round-1-probes.md` as the caller, state-vocabulary, and trace-write baseline: `run_rem.sh` and `run_rem_noop.sh` consume exit codes, while `deploy-mem-claw.sh:649-656` consumes JSON fields and never reads the numeric contract. — evidence: `evidence/section-1-contract-freeze.md`, `run_rem_noop.sh`, `deploy-mem-claw.sh`
- [x] 1.2 Record the exact cross-repository ownership boundary: this change owns REM outcome classification and both runners' routing; it owns `run_rem.sh`'s own exits and trace fields, while `rem-operation-switches` owns which names `run_rem.sh` accepts and its validation message and this change owns the exit-number line in that same block. — evidence: `evidence/section-1-contract-freeze.md`, `REQ-18 Both traces record the routing tuple`, `REQ-20 Runner-owned exits are disjoint`
- [x] 1.3 Have a fresh independent test owner derive one complete plan for QCG-1 through QCG-17 and, before each product section, freeze that section's admitted smallest RED checks. Before section 2, freeze QCG-1, QCG-2, QCG-6, and QCG-7 under `evidence/80-rem-job-state-contract/`, keeping product implementation context out of that test owner. — evidence: `evidence/80-rem-job-state-contract/qcg-1.txt`, `test qcg_1_single_declaration_owns_codes ... ok`

## 2. Establish the Single Outcome Declaration

- [x] 2.1 Using `codex-coder`, add one outcome-class declaration that owns each class name, process exit code, and assigned error codes; give success and unclassified no assigned errors, invalid usage exactly `usage_error`, and map all thirteen existing REM error codes. Make both process-exit sites in `src/service.rs` and `src/cli.rs` derive classified REM machine codes from that declaration while generic runtime errors remain at their existing exits. Do not make the Rust source layout part of the contract. — evidence: `evidence/80-rem-job-state-contract/qcg-6.txt`, `test qcg_6_all_raisable_rem_codes_are_mapped ... ok`
- [x] 2.2 Make duplicate exit codes, duplicate error-code membership, moved or unmapped raisable REM codes, an error assigned to unclassified, and named assignment of exit `1` fail separate targeted tests. Prove the shipped declaration and real REM error-to-process paths satisfy those semantics without requiring a particular module, type, constructor, iterator, accessor, or formatting shape. — evidence: `evidence/80-rem-job-state-contract/qcg-2.txt`, `test qcg_2_duplicate_exit_and_error_are_rejected ... ok`
- [x] 2.3 Have the independent test owner rerun and freeze GREEN for QCG-1, QCG-2, QCG-6, and QCG-7 without changing the product implementation. — evidence: `evidence/80-rem-job-state-contract/qcg-7.txt`, `test qcg_7_exit_one_is_unclassified_only ... ok`

## 3. Separate State Vocabulary from Response Validity

- [ ] 3.1 Using `codex-coder`, implement the waiting and non-waiting state matrix: every non-empty unfamiliar state immediately emits `rem_state_unrecognised` with exit `5`, while invalid JSON and an absent or empty state emit `sidecar_response_invalid` with exit `6`.
- [ ] 3.2 In both modes, print the unfamiliar raw state to stdout before the error and include the job identifier, byte-identical state, and version-skew explanation in the message, including through command substitution.
- [ ] 3.3 Preserve the sidecar-provided error string in `rem_job_failed` messages.
- [ ] 3.4 Have the independent test owner rerun and freeze GREEN for QCG-5, QCG-8, QCG-9, QCG-10, and QCG-11.

## 4. Publish and Protect the CLI Contract

- [ ] 4.1 Using `codex-coder`, assign the exact ten outcome exit codes `0` through `9` and keep non-REM generic runtime failures at exit `1`.
- [ ] 4.2 Update `README.md` with the `station rem-*` exit-code table and compare its normalized semantic rows with the declaration without depending on either source format.
- [ ] 4.3 Have the independent test owner rerun and freeze GREEN for QCG-3 and QCG-4.

## 5. Route Both Memora Consumers

- [ ] 5.1 After section 4 is complete, use `codex-coder` to add the same enumerated routing table to `evals/memora/scripts/run_rem.sh` and `evals/memora/scripts/run_rem_noop.sh`, apply it to each runner's `rem-start` and `rem-status`, and fail closed while logging any unmatched code.
- [ ] 5.2 Keep every existing `--json` argument and route both runners only on exit codes. In `run_rem.sh`, move its own usage and rejected-operation failures to exits `20` and `21`; do not originate `0` through `9`, and return a code in that range only by propagating the immediately captured result of a real `sno` invocation.
- [ ] 5.3 In `run_rem.sh`, leave the sibling-owned accepted names and unknown-operation message byte-identical while changing the rejected-operation exit number, then prove that this change and `rem-operation-switches` work in either landing order against the same store.
- [ ] 5.4 Have the independent test owner rerun and freeze GREEN for QCG-12, QCG-13, QCG-15, QCG-16, and QCG-17, with QCG-12 covering both runners.

## 6. Extend Existing Traces

- [ ] 6.1 Using `codex-coder`, record the unrecognised state's byte-identical raw state, outcome class, and exit code in the existing CLI and `run_rem.sh` traces.
- [ ] 6.2 Have the independent test owner rerun and freeze GREEN for QCG-14 on the unrecognised-state path through `run_rem.sh`.

## 7. Run the Acceptance Gates

- [ ] 7.1 Run `cargo test outcome_class_is_sole_source` and save the actual output to `evidence/80-rem-job-state-contract/qcg-1.txt`.
- [ ] 7.2 Run `cargo test outcome_exit_codes_are_unique` and `cargo test outcome_error_codes_are_unique`; save separate duplicate-exit and duplicate-error-membership negative controls to `evidence/80-rem-job-state-contract/qcg-2.txt`.
- [ ] 7.3 Run `cargo test readme_exit_code_table_matches_declaration` with a documentation-drift negative control and save the output to `evidence/80-rem-job-state-contract/qcg-3.txt`.
- [ ] 7.4 Run `cargo test non_rem_commands_keep_exit_one` with these exact actors: `sno account machine register --json` against an unreachable loopback `SNO_OBSERVE_BASE_URL`, and `sno service fail-runtime` through a real executable external subcommand that exits `1`; then stop the live sidecar and prove both REM entrypoints exit `7`, restoring it afterward. Save the output to `evidence/80-rem-job-state-contract/qcg-4.txt`.
- [ ] 7.5 Run `bash tests/rem-status-exit-code-matrix.sh` for every required outcome across ten repetitions with no interchange and save the output to `evidence/80-rem-job-state-contract/qcg-5.txt`.
- [ ] 7.6 Run `cargo test every_rem_error_code_is_mapped` with an unmapped-raise-site negative control and save the output to `evidence/80-rem-job-state-contract/qcg-6.txt`.
- [ ] 7.7 Run `cargo test exit_one_is_unclassified_only` with explicit-mapping and deliberate-fallback controls and save the output to `evidence/80-rem-job-state-contract/qcg-7.txt`.
- [ ] 7.8 Run `bash tests/rem-status-non-waiting-poll.sh` for queued, running, done, failed, and a non-empty unfamiliar state; prove the unfamiliar state prints and exits `5`, and save the output to `evidence/80-rem-job-state-contract/qcg-8.txt`.
- [ ] 7.9 Run `bash tests/rem-status-unrecognised-state-message.sh` in waiting mode, including immediate exit, command-substitution capture, and byte-identical state checks, and save the output to `evidence/80-rem-job-state-contract/qcg-9.txt`.
- [ ] 7.10 Run `cargo test response_invalid_narrowed_to_malformed` for invalid JSON, empty state, and a well-formed unfamiliar state and save the output to `evidence/80-rem-job-state-contract/qcg-10.txt`.
- [ ] 7.11 Run `bash tests/rem-status-job-failed-carries-sidecar-error.sh` with present and absent sidecar error text and save the output to `evidence/80-rem-job-state-contract/qcg-11.txt`.
- [ ] 7.12 After the end-to-end preflight passes, run `bash tests/rem-routing-table-e2e.sh` through both ordinary callers, `run_rem.sh` and `run_rem_noop.sh`, exactly as the Memora harness invokes them, entering through the Chapter 0-selected installed `sno` binary and reaching a live REM sidecar over its real socket with a real persona store. Produce each of the ten tool exits one at a time from the sidecar and binary and verify each runner's routing-table fate. Use a real tool build that emits an eleventh code for the negative proof; require both runners to fail non-zero and log that unmatched code, with that log absent from all known-code runs. Save independently owned output to `evidence/80-rem-job-state-contract/qcg-12.txt`.
- [ ] 7.13 Run `bash tests/rem-unrecognised-state-fails-closed.sh` and save the persona failure plus useful version-skew log to `evidence/80-rem-job-state-contract/qcg-13.txt`.
- [ ] 7.14 Run `bash tests/rem-trace-records-state-class-and-exit-code.sh` for an unrecognised state through `run_rem.sh`; verify the CLI and runner traces contain the byte-identical raw state, outcome class, and exit code, and save output to `evidence/80-rem-job-state-contract/qcg-14.txt`.
- [ ] 7.15 Run `bash tests/rem-routing-ignores-message-text.sh` with a message-replacement negative control and save the output to `evidence/80-rem-job-state-contract/qcg-15.txt`.
- [ ] 7.16 Run `bash tests/rem-runner-own-exit-codes.sh` for `run_rem.sh` invoked without an argument and with a rejected operation, proving exits `20` and `21`; search the script for any literal `exit` in `0` through `9`, and walk every path returning a code in that range back to the tool's own immediately captured exit. Save the output to `evidence/80-rem-job-state-contract/qcg-16.txt`.
- [ ] 7.17 Run `bash tests/rem-runner-independent-landing-order.sh` for both single-change landing orders against the same store and save the output to `evidence/80-rem-job-state-contract/qcg-17.txt`.
## 8. Final Verification

- [ ] 8.1 Run the targeted Rust and shell test sets, typecheck or build checks required by each repository, and `openspec validate rem-job-state-contract --strict`; record exact commands and outputs without substituting a full test-tree run.
- [ ] 8.2 Confirm all 21 requirements are covered, all 17 acceptance rows name existing evidence containing their anchored output, no acceptance row is promoted without a real run, no compatibility path was introduced, and the only edit inside sibling-owned operation validation is this change's rejected-operation exit number.
