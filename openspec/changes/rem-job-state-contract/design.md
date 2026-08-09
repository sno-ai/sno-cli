## Context

The Rust CLI currently collapses materially different REM outcomes into generic runtime failure behavior. The Memora caller crosses a repository boundary and needs a stable numeric contract: the tool owns classification and the shell runner owns the fate of each classified result. The repository has no main OpenSpec specifications today, so this change establishes the initial capability contract.

The implementation spans `/home/lh/code/sno-cli` and two station-core consumers: `evals/memora/scripts/run_rem.sh` and `evals/memora/scripts/run_rem_noop.sh`. A repository-wide search found one other command invoker, `evals/sno-memory-bench/deploy-mem-claw.sh`, but its lines 649-656 consume `.job_id` and `.state` from JSON and die on any command failure; it never reads or branches on the numeric exit and therefore does not consume this contract. The probe and source revisions are recorded in `evidence/review-round-1-probes.md`.

The REM sidecar protocol and the Memora harness above the runners remain unchanged. The sibling owns which operation names are accepted; this change owns the rejected-operation exit number in the same validation block.

## Goals / Non-Goals

**Goals:**

- Make one declaration the sole owner of every REM outcome class, machine-readable error code, and process exit code.
- Give every known REM failure a deterministic class and preserve exit code `1` for unknown failures.
- Distinguish a well-formed unfamiliar state from malformed or truncated sidecar data.
- Let both Memora runners route by an exhaustive numeric table and fail closed on an unknown code.
- Leave enough trace data on both sides to reconstruct the routing decision.
- Verify the installed CLI, ordinary runner, live sidecar, and real persona-store boundary with an independent test owner.

**Non-Goals:**

- Changing the REM sidecar's state vocabulary or response schema.
- Changing non-REM CLI exit behavior.
- Changing the Memora harness's handling of either runner's final success or failure.
- Changing which operation identifiers are accepted or the associated validation message owned by `rem-operation-switches`; changing the rejected-operation exit number remains in scope.
- Adding a compatibility path for the current internal, unreleased exit behavior.

## Decisions

### One declaration owns both machine contracts

The Rust tool will define one declaration shaped as an ordered collection of records with exactly three fields: `name`, `exit_code`, and `error_codes`. `error_codes` is a list because several error codes fold into one outcome class. The success and unclassified records have empty lists; invalid usage contains exactly `usage_error`; every other named failure class contains one or more error codes. Both sites that convert a runtime error into a process result will resolve through this declaration.

Class-name uniqueness, exit-code uniqueness, and global error-code uniqueness are separate invariants. Separate negative tests will duplicate a class name, duplicate an exit code, and place one error code in two lists. Other tests reject missing mappings, a non-empty unclassified list, and README drift.

Maintaining separate error and exit tables was rejected because they could disagree while each remained locally valid. Writing codes at raise sites was rejected because it would recreate the same distributed ownership.

### The ten classes use a closed exit-code range

The tool-owned contract is:

| Exit | Outcome class |
|---:|---|
| 0 | Success |
| 1 | Unclassified failure |
| 2 | Invalid usage |
| 3 | Job failed |
| 4 | Wait deadline passed |
| 5 | State vocabulary mismatch |
| 6 | Malformed or truncated response |
| 7 | Sidecar reachability, discovery, authentication, or client failure |
| 8 | Local environment, profile, or trace failure |
| 9 | Unknown job identifier |

Exit code `1` has an empty `error_codes` list. It is reached only when classification has no mapping, preserving a visible fail-closed fallback. Success also has an empty list because it is not an error. Non-REM callers of the generic runtime error keep exit code `1`.

### Response validity and state vocabulary are separate boundaries

The authoritative sidecar state declarations contain exactly `queued`, `running`, `done`, and `failed`; they expose no independent terminal-state field. The CLI intentionally deserializes `state` as a string, so it can diagnose a state introduced by another build. The contract therefore treats every non-empty unfamiliar state as version skew regardless of wait mode rather than trying to infer whether it is terminal.

| State input | Non-waiting `rem-status` | Waiting `rem-status --wait` |
|---|---|---|
| `queued` or `running` | Print state and exit `0` | Continue polling until deadline or another state |
| `done` | Print state and exit `0` | Print state and exit `0` |
| `failed` | Preserve sidecar error and exit `3` | Preserve sidecar error and exit `3` |
| Any other non-empty string | Print raw state, emit `rem_state_unrecognised`, and exit `5` | Print raw state, emit `rem_state_unrecognised`, and exit `5` immediately |
| Missing or empty state, or undecodable job response | Emit `sidecar_response_invalid` and exit `6` | Emit `sidecar_response_invalid` and exit `6` |

The version-skew message names the job, state, and vocabulary mismatch. A failed job continues to carry the sidecar's supplied error text.

Treating every unfamiliar state as malformed was rejected because it hides which component is newer and discards a valid state value needed for diagnosis.

### The shell callers route only by exit code

Each of the two consumers uses the same enumerated routing table for both `rem-start` and `rem-status`. Known codes have one fate each; an absent code fails the persona and is logged. Both runners retain their existing JSON usage but never parse message prose to decide. Each currently originates `exit 2` for both usage and rejected-operation failures, so both move those paths to `20` and `21`; codes `0` through `9` can only be propagated from the tool.

Routing on message text was rejected because prose is diagnostic, not a stable machine interface. Reusing the tool's range for runner failures was rejected because the same number would have two owners.

### Traces record an honest decision tuple

Every final CLI and runner outcome record written to an available REM trace will carry `raw_state`, `state_unavailable_reason`, `outcome_class`, and `exit_code`. Exactly one of `raw_state` and `state_unavailable_reason` is non-null. If a decoded job record exists, `raw_state` contains its state byte-for-byte and the absence reason is null. If no job state exists, `raw_state` is null and the reason is the machine-readable error code, or `job-state-not-returned` for a successful `rem-start` response. A trace sink that cannot be opened or written cannot record its own `rem_trace_error`; that error remains the explicit observable failure.

The current CLI trace has a state only on successful `command_emitted` status rows, while both runners record only stdout and exit code on `harness_cli_received`. The implementation must therefore add a final outcome row on failure paths rather than merely renaming existing success fields. These are extensions of the existing JSONL trace, not a new tracing subsystem. Trace extraction may observe structured output for recording, but routing remains exclusively exit-code based.

### Cross-repository edits remain independently landable

The CLI declaration and messages land before the runner routing tables so the runners do not freeze today's collapsed behavior. In the main runner's validation block, `rem-operation-switches` owns the accepted names and message while this change owns the exit number. The lines share a block but neither edit depends on the other, and both landing orders must pass.

## Risks / Trade-offs

- **A new Rust error code is raised without a mapping** → An exhaustive mapping test fails the build; runtime still fails closed through exit `1`.
- **Either runner sees a future exit code** → Its default arm fails the persona and logs the unmatched code.
- **README behavior drifts from the declaration** → A declaration-to-document consistency test blocks the change.
- **A captured stdout path loses the unfamiliar state** → Integration coverage invokes the same command-substitution boundary used by both runners.
- **Cross-repository changes land in the wrong order** → The CLI classification lands first, and an independent landing-order test covers the main runner with each change present alone.

## Migration Plan

1. Add the declaration, all existing REM error mappings, uniqueness checks, and both error-to-exit lookups.
2. Separate unfamiliar-state handling from malformed-response handling.
3. Assign and verify the ten outcome exits and their diagnostic messages.
4. Update the README table and protect non-REM behavior.
5. After the tool emits the new codes, add the routing table and fail-closed arm to both runners, and move each runner's own usage and rejected-operation exits to `20` and `21`. In the main runner, leave the sibling-owned accepted names and message unchanged while changing the exit number in that block.
6. Extend both existing traces and run the real installed-tool, live-sidecar, real-store acceptance path.

Rollback is a single coordinated revert before release. No stored data or external published contract requires migration.

## Open Questions

None. The outcome matrix, fail-closed behavior, landing order, and sibling boundary are owner-settled.
