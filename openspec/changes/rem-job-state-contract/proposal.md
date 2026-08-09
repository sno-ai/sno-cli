## Why

The `sno station rem-*` command family does not yet expose one stable, machine-readable outcome contract across job states, response failures, and client failures. The two Memora runners that consume the tool's exit code therefore cannot route every REM outcome deterministically without relying on message text or collapsing distinct failures into exit code 1.

## What Changes

- Establish one declaration as the source of truth for the ten REM outcome classes, stable error codes, and process exit codes 0 through 9.
- Map every existing REM error code and every terminal or non-terminal job state to that declaration, including a distinct version-skew outcome for unfamiliar states.
- Preserve queued and running states as successful non-wait observations, preserve sidecar failure details, and narrow malformed-response handling to malformed JSON or a missing or empty state.
- Document and verify the exit-code table while retaining exit code 1 as the fallback for non-REM commands and otherwise unclassified REM failures.
- Give both exit-code consumers, `run_rem.sh` and `run_rem_noop.sh`, an enumerated routing table, fail closed on unknown codes, reserve 20 and 21 for their own failures, and forbid routing by human-readable messages.
- Record the nullable raw state or its machine-readable absence reason, outcome class, and exit code on both sides of the command boundary.
- **BREAKING**: Replace the current implicit REM exit behavior with the new internal, unreleased contract without a compatibility path. A repository-wide caller search now evidences that the two Memora runners are the complete numeric-contract consumer set; the deployment verifier invokes the commands but never reads or branches on their exit codes.

## Capabilities

### New Capabilities

- `rem-job-state-contract`: Defines REM outcome classification, stable exit behavior, runner routing, diagnostics, and cross-boundary traceability. This is the repository's first main OpenSpec capability specification.

### Modified Capabilities

None. The repository has no existing main specifications, so this change establishes the initial specification rather than amending one.

## Impact

- Affects the Rust `sno station rem-*` command family, its README exit-code documentation, and its unit and integration coverage.
- Affects the station-core Memora runners at `evals/memora/scripts/run_rem.sh` and `evals/memora/scripts/run_rem_noop.sh` and their contract tests.
- Does not change which operation names either runner accepts. It does change the rejected-operation exit number inside each validation block; name ownership and exit ownership are separate lines.
- Does not apply the numeric routing contract to `evals/sno-memory-bench/deploy-mem-claw.sh`: the read at lines 649-656 shows that it consumes JSON fields and treats every command failure as fatal without inspecting an exit code. The complete caller probe is recorded in `evidence/review-round-1-probes.md`.
- Requires a later real-sidecar end-to-end run through the installed `sno` command; this artifact-only round changes no Rust or runner source.
