# Probe results for independent REM job-state contract tests

Date: 2026-08-09

This file records read-only probes gathered after the test author independently derived the
behavior claims from the released PRD. It is ground truth for test-scope admission, not product
implementation rationale.

## Contract-ledger probe

Command:

```sh
rg -n '^[-] `\[QCG-' \
  '/home/lh/code/sno-station-core-edge-rem-wave/ai-doc/ACTIVE/PRD/[IMP]-edge-rem/80-rem-job-state-contract-prd.md'
```

Result: the released PRD contains QCG-1 through QCG-17 and no QCG-18. OpenSpec `tasks.md`
requires QCG-1 through QCG-18. `evidence/review-round-2-open-findings.md` classifies that ledger
split as open Critical.

## Existing sno-cli test-boundary probe

Commands:

```sh
rg --files tests
rg -n 'rem_exit_codes_are_stable|rem_one_shot_status_reads_running_then_stable_done|rem_human_output_keeps_job_id_actionable_on_success_and_failure|rem_wait_times_out_when_sidecar_never_appears' tests/cli.rs
```

Results:

- The repository has one CLI integration crate, `tests/cli.rs`, plus its loopback HTTP helper.
- Existing coverage proves the old collapsed exit behavior, a running-to-done non-waiting poll,
  preservation of a supplied sidecar error string, and bounded timeout behavior.
- No test named for the new outcome declaration, exact mapping, README consistency, unknown-code
  runner routing, runner-owned exits, or final trace tuple exists.
- The loopback helper is a protocol substitute, not the real REM sidecar required by the PRD's E2E
  claim. It cannot be used to promote QCG-5 or QCG-8 through QCG-15 as real-boundary proof.

## Runner and real-boundary probe

Commands:

```sh
rg -n 'SNO_CLI_BIN|rem-start|rem-status|exit 2' \
  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh \
  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
command -v sno
sno --version
```

Results:

- Both runners accept an explicit `SNO_CLI_BIN`, capture both REM command exits, and currently
  originate exit `2` for usage and rejected operation.
- The installed binary is `/home/lh/.cargo/bin/sno`, version `0.1.7`.
- A real-sidecar integration suite exists in the Station Core repository, but no checked-in
  fixture or command was found that makes that sidecar emit an unfamiliar non-empty state,
  malformed JSON, truncated JSON, every exit class, or a future exit code while preserving the
  ordinary installed-binary boundary.
- The PRD requires `e2e-chapter0` before E2E. That manual-only skill was not invoked by the caller,
  so no E2E command is admitted in this turn.

## Workspace probe

Command:

```sh
git status --short
git -C /home/lh/code/sno-station-core-edge-rem-wave status --short
git -C /home/lh/code/sno-station-core status --short
```

Result: all three worktrees contain pre-existing changes. The test owner will edit only test-owned
files and evidence in `sno-cli`; no runner, product source, manifest, README, or `tasks.md` edit is
authorized.

## Section 2 process-boundary reachability

Commands:

```sh
sed -n '360,410p' src/rem.rs
sed -n '285,365p' src/service.rs
sed -n '445,470p' src/service.rs
sed -n '790,830p' tests/cli.rs
sed -n '470,540p' tests/cli.rs
```

Results:

- A non-success `rem-start` response is decoded as `RemErrorResponse`; its unrestricted `error`
  string becomes the `CliError` machine code at `src/rem.rs:399-405`. The existing
  `rem_start_surfaces_failed_allocated_job` integration test proves the real compiled binary
  preserves a loopback response code (`unsupported_rem_type`) and exits through the ordinary CLI
  error boundary. Therefore the same protocol-valid response can supply each frozen REM code and
  a future code without importing a product-private Rust API.
- JSON `account machine claim` has a separate process-exit boundary at `src/service.rs:85-98`.
  Its device-token error path preserves the server's unrestricted `error` string through
  `claim_http_error` at `src/service.rs:445-464`. A response carrying the string `rem_timeout`
  therefore exercises the same machine-code text outside REM classification and must still exit
  `1`; this is the negative proof that mapping is command-contextual rather than a global lookup.
- Both boundaries use the checked-in loopback HTTP helper and complete in one request sequence;
  neither is a real-sidecar E2E claim and neither requires Chapter 0.

## Section 2 source-ownership exceptions

Commands:

```sh
rg -n 'profile_error|usage_error' src --glob '*.rs'
rg -n 'CliError::runtime\(' src/rem.rs
rg -n 'error\.exit_code' src/cli.rs src/service.rs
```

Results:

- `profile_error` has an intentional generic non-REM raise in `src/state.rs:110`; Section 2 must
  allow that occurrence while forbidding independently owned REM mappings or REM raise-site
  literals.
- `usage_error` is created by the generic parser path in `src/error.rs`; its non-REM occurrence is
  likewise not a second REM mapping owner.
- The current REM file still has generic runtime raise calls and the two process exits still read
  the stored exit field. Those are current-state facts, not required replacement names: the
  independent test may reject the legacy bypasses but may not prescribe what replaces them.
