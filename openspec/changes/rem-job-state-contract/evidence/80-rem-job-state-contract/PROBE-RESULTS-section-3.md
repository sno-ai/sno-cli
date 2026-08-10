# Probe Results: Section 3 Test Admission

These are command-backed repository observations. They are not product-agent
rationale.

## Workspace

Command:

```text
git status --short
git branch --show-current
```

Observed before test-plan creation:

```text
main
```

The worktree was clean and the branch was `main`.

## Existing test boundary

Command:

```text
rg --files . | rg '(^|/)(tests?|support|fixtures?)(/|$)|openspec/changes/rem-job-state-contract/tasks\.md$'
```

Observed relevant files:

```text
./tests/rem_job_state_contract.rs
./tests/cli.rs
./tests/support/sno_service_server.rs
./openspec/changes/rem-job-state-contract/tasks.md
```

`tests/support/sno_service_server.rs` binds a real ephemeral `127.0.0.1` TCP
listener, reads real HTTP request bytes, and writes response bytes. The tests
invoke `env!("CARGO_BIN_EXE_sno")` as a child process with temporary profile
and discovery files. This is a real compiled CLI and real local socket, but the
fixture substitutes for the external sidecar.

## Existing exact or partial coverage

Command:

```text
rg -n 'qcg_[1267]|rem_one_shot_status_reads_running_then_stable_done|rem_exit_codes_are_stable|rem_common_local_errors_are_clean|rem_human_output_keeps_job_id_actionable' tests
```

Observed:

```text
tests/rem_job_state_contract.rs:289:fn qcg_1_single_declaration_owns_codes() {
tests/rem_job_state_contract.rs:311:fn qcg_2_duplicate_exit_and_error_are_rejected() {
tests/rem_job_state_contract.rs:341:fn qcg_6_all_raisable_rem_codes_are_mapped() {
tests/rem_job_state_contract.rs:374:fn qcg_7_exit_one_is_unclassified_only() {
tests/cli.rs:830:fn rem_human_output_keeps_job_id_actionable_on_success_and_failure() {
tests/cli.rs:921:fn rem_one_shot_status_reads_running_then_stable_done() {
tests/cli.rs:959:fn rem_exit_codes_are_stable() {
tests/cli.rs:1022:fn rem_common_local_errors_are_clean() {
```

The one-shot test already proves non-waiting `running` then `done` exit 0. The
exit-code test covers only done, failed, timeout, and usage. The local-error
test covers stopped discovery and unknown job. No test contains an unfamiliar
state or a missing/empty state response.

## Existing failed-job coverage

Commands:

```text
sed -n '830,920p' tests/cli.rs
rg -n -C 8 'state.*failed|rem_job_failed|error' tests/rem_job_state_contract.rs tests/cli.rs
```

Observed: `rem_human_output_keeps_job_id_actionable_on_success_and_failure`
supplies one non-waiting failed record with
`"error":"sidecar_restart"`, then asserts exit `3`, the job identifier, and
the exact `sidecar_restart` text on stderr. `rem_exit_codes_are_stable` has a
second non-waiting failed record with the same error but asserts only exit `3`.
No existing failed-record case uses `--wait`, and no existing failed record
omits or nulls the `error` field. Therefore the provided-error non-waiting
message oracle is reusable; waiting-with-error, waiting-without-error, and
non-waiting-without-error are the three unsupported cases.

## Current reachable RED location

Command:

```text
rg -n 'sidecar_response_invalid|rem_state_unrecognised|job.state.as_str' src/rem.rs src/rem_outcome.rs
```

Observed declaration members:

```text
src/rem_outcome.rs:78:            code: "rem_state_unrecognised",
src/rem_outcome.rs:87:                code: "sidecar_response_invalid",
```

Observed ordinary status branch in `poll_status_at`:

```text
"done" => return Ok(job),
"failed" => { ... RemError::JobFailed ... }
"queued" | "running" if wait => {}
"queued" | "running" => return Ok(job),
_ => { ... RemError::ResponseInvalid ... }
```

Therefore a well-formed non-empty unfamiliar state reaches the real status
branch but currently selects the malformed-response class. The expected RED
is a product-behavior mismatch, not a missing test symbol.

## Native local-profile trigger

Command:

```text
sed -n '253,263p' src/rem.rs
```

Observed: REM profile resolution uses `SNO_PROFILE_DIR`, then `SNO_HOME`, then
`HOME`, then `USERPROFILE`, and raises the typed profile error only when all
four are absent. A child process with those four variables removed reaches the
native local-environment outcome without a fake sidecar response.

## Boundary not available in this repository

No real external REM sidecar implementation or persona store appears under
this repository's test/support paths. The owner explicitly prohibited editing
the external repository. Consequently, the loopback CLI contract test is
feasible here; QCG-5's release-level cross-repository live-sidecar E2E is not.
