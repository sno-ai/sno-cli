# Installer test close

## Verification

On `gpt1`, in `/home/lh/code/sno-cli`, the final `cargo test --test assemble -- --nocapture`
run passed **38 tests**, failed none, and explicitly ignored one external-data case.
The run took 48.18 seconds. One passing test is the `fixture_dispatch` subprocess entry,
not an acceptance case. [The retained log](installer-fixture-tests.log) is a byte-for-byte
copy of `/tmp/sno-installer-final-tests.log`.

The ignored mixed-utility case was run separately and passed. Its original output was
captured by the shell tool, not redirected to a file. The
[captured-output receipt](installer-mixed-utility-tests.log) identifies that distinction.
The exact command was:

```sh
SNO_TEST_HEARTBEAT_ARCHIVE=/home/lh/code/sno-station-core/apps/heartbeat/dist/heartbeat-1.0.tar.gz SNO_TEST_QUOTA_ARCHIVE=/home/lh/code/sno-station-core/apps/subscription-quota-check/dist/subscription-quota-check-1.0.tar.gz cargo test --test assemble accepted_core_utilities_install_with_synthetic_reach_and_skills -- --ignored --exact --nocapture
```

`rustfmt --check` and `git diff --check` passed for the changed test paths. The only change
to `tests/cli.rs` removes the obsolete `doctor` retirement assertion. Its exact
`legacy_root_commands_are_rejected` and `root_help_version_and_missing_command_are_stable`
cases passed. No test or review was rerun to prepare this close record.

## Authoritative test-review dispositions

The [one-pass authoritative report](installer-test-review.md) is preserved unchanged.
This table records repairs; it does not replace that report's findings. No second judgment
pass was run after repairs or subsequent acceptance-test additions.

| Finding | Disposition and verified repair |
|---|---|
| 1 — Escape test could fail for missing normal payload | Admitted and repaired. Both an escaping link and a raw traversal entry are inserted into an otherwise complete valid archive. Rejection must identify `../../outside`. |
| 2 — Scheduler substitute conflated enabled and running | Admitted and repaired. The substitute checks `--user` and the exact unit, stores separate enabled/running states, and starts/stops only with `--now`. Lifecycle and recovery assertions check both states. |
| 3 — Missing-program check lacked a healthy baseline | Admitted and repaired. The test requires the exact `doctor reach reach ok` row before deletion and `doctor reach reach fail` afterward, with matching JSON and preserved station results. |

## Source-review regression coverage

The [source review](installer-code-review.md) remains the authority for its original findings.

| Source finding or observed regression | Final evidence |
|---|---|
| Missing core programs reported as success | Returned to the parent as **[Possible FP]**: DEC-3/4 explicitly permits a named dependent-skill skip. No claim is made that an absent-program test already existed; the version-decision test proves the too-old heartbeat case. This is a contract disposition, not a claimed code repair. |
| Restrictive umask breaks later mutation/recovery | `restrictive_umask_preserves_modes_across_update_and_rollback` verifies exact modes under `umask 077`, unchanged repeat, rollback, update and remove. |
| Committed remove loses directory cleanup | `committed_remove_cleans_owned_directories_before_reassembly` kills after manifest removal, checks all owned payload was already removed, recovers directory cleanup, and reassembles. |
| Embedded bodies exceed journal capacity | `large_program_update_and_recovery_keep_payload_out_of_journal_metadata` installs and updates a deterministic 20 MiB resource, checks metadata stays below 1 MiB, verifies the referenced blob, retries after a kill, and checks old payload pruning. |
| Scheduler failure commits state; active-timer remove locks itself; repeat enable rewrites generation | Each was observed failing and then passing in its named scheduler/lifecycle regression case. |
| Reminder timeout leaves its external child running | Observed in the synthetic installed Reach process. `addressed_reminder_failure_is_visible_and_timeout_stops_the_program` now passes with visible errors, a bounded wrapper, and no surviving child. The test cleaned its own failing-run process. |

## Acceptance boundaries

Local fixture assertions now cover QCG-1/3/4/5/6/7/8/10, including full owned-file repeat
snapshots, whole-tree rollback and lock-conflict snapshots, a genuinely changed owned hook,
healthy station state created through the real CLI/SQLite path, read-only Codex trust,
human/JSON parity, exhaustive owned-file removal, and adjacent-state preservation.
The parent separately ran those QCG proof groups with nonzero exact case counts.

The compiled test driver invokes the real CLI parser. Its scheduled-command case executes
the generated `ExecStart` under only the generated HOME/PATH and observes an installed
version change. Native scheduler triggering is not proved.

The mixed consumer case uses the actual accepted local utility archives, but synthetic
Reach and skill payloads. It proves utility artifact parsing, installation, dependency
diagnostics and owned removal, not publication or a live utility/harness workflow.

The actual shared requirements contract exists and matches the supported digest. The real
four-unit `scripts/fixtures/s-category-requirements.json` was still absent at the final
read-only check. Gate-produced staged proof, published-release proof, live harness discovery,
adapter handshakes and prompt injection remain unproved. **QCG-2 and QCG-9 remain false/null;
neither this report nor the mixed utility case closes them.**
