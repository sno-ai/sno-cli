# Probe results — Section 4 REM CLI contract

These are read-only facts gathered after deriving QCG-3 and QCG-4 from the released PRD.

## Worktree and task ledger

Commands:

```text
git status --short
sed -n '18,28p' openspec/changes/rem-job-state-contract/tasks.md
```

Result: the worktree was clean at Section 4 pickup. Tasks 4.1–4.3 require the ten exit codes,
normalized README/declaration comparison, and independent QCG-3/QCG-4 freeze.

## Documentation drift

Command:

```text
sed -n '40,78p' README.md
```

Result: the current README says a failed job or timeout exits `1`; it does not publish the released
ten-row REM contract. The shipped declaration already contains the ten released semantic rows.
Therefore QCG-3 has an expected genuine product-documentation RED.

## Exact QCG-4 actors

Commands:

```text
cargo run --quiet -- account machine register --help
cargo run --quiet -- service --help
cargo run --quiet -- station --help
rg -n 'external_subcommand_preserves_literal_arguments_and_exit_code|SNO_OBSERVE_BASE_URL|rem_common_local_errors_are_clean' tests/cli.rs
```

Results:

- `account machine register --json` is a released built-in command and honors
  `SNO_OBSERVE_BASE_URL`; an unlistened loopback endpoint reaches a generic runtime failure.
- `service` is not built in. A real `sno-service` executable on temporary `PATH` is the released
  external-subcommand boundary; `service fail-runtime` can deterministically exit `1`.
- Both `station rem-start` and `station rem-status` are released. Their discovery file is temporary
  local state, so a bound loopback listener can be stopped after its address is written, then a
  fresh loopback sidecar can be written and exercised to prove restoration before cleanup.
- Existing tests prove pieces of these mechanisms but not the exact four-actor QCG-4 discriminator.

## Harness and dependencies

Commands:

```text
sed -n '1,280p' tests/support/sno_service_server.rs
sed -n '1,120p' Cargo.toml
```

Result: `tests/rem_job_state_contract.rs` already uses the compiled binary, `TempDir`, and the
checked-in loopback HTTP server. No new runner or dependency is required. Mock inventory remains
empty.
