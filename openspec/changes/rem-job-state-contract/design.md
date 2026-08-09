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

The Rust tool will define one declaration that owns each outcome class name, its process exit code, and the error codes assigned to it. Several error codes may fold into one outcome class. Success and unclassified have no assigned error code; invalid usage owns exactly `usage_error`; every other named failure class owns one or more error codes.

Classification is bounded to the REM command family. Generic `CliError::runtime` remains unclassified and keeps exit `1`, including the existing non-REM `profile_error` raise in `src/state.rs`. Both process-conversion sites derive a classified REM error's process exit from the single declaration while preserving the existing generic exit for every error not classified by a REM command. The Rust module, type, constructor, and accessor layout are implementation choices, not part of QCG-1.

Independent tests compare the declaration's observable semantic rows and exercise real REM error-to-process conversion. They also prove that a raise site cannot independently choose either machine code. README verification compares normalized semantic rows rather than Rust or Markdown formatting. No test requires a particular module, type, constructor, iterator, or accessor layout.

Exit-code uniqueness and global error-code uniqueness are separate invariants. Separate negative tests duplicate an exit code and place one error code in two outcome classes. Other tests reject missing mappings, an error assigned to unclassified, a named outcome assigned exit `1`, and README drift.

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

Each of the two consumers uses the same enumerated routing table for both `rem-start` and `rem-status`. Known codes have one fate each; an absent code fails the persona and is logged. Both runners retain their existing JSON usage. `run_rem.sh` never parses message prose to decide, moves its own usage and rejected-operation failures from `2` to `20` and `21`, and only propagates codes `0` through `9` from the tool.

QCG-16 exercises the two runner-owned failures, searches the script for any literal runner exit in `0` through `9`, and walks every path returning a code in that range back to the immediately captured result of a real `sno` invocation. The shell function layout is an implementation choice, not an additional contract.

Routing on message text was rejected because prose is diagnostic, not a stable machine interface. Reusing the tool's range for runner failures was rejected because the same number would have two owners.

### Traces record an honest decision tuple

When an unrecognised state is reported, the existing CLI and `run_rem.sh` traces will record the raw state byte-for-byte, the outcome class, and the exit code. This extends the existing JSONL traces rather than creating a new tracing subsystem. Trace extraction may observe structured output for recording, but routing remains exclusively exit-code based.

### Cross-repository edits remain independently landable

The CLI declaration and messages land before the runner routing tables so the runners do not freeze today's collapsed behavior. In the main runner's validation block, `rem-operation-switches` owns the accepted names and message while this change owns the exit number. The lines share a block but neither edit depends on the other, and both landing orders must pass.

## Risks / Trade-offs

- **A new Rust error code is raised without a mapping** → An exhaustive mapping test fails the build; runtime still fails closed through exit `1`.
- **Either runner sees a future exit code** → Its default arm fails the persona and logs the unmatched code.
- **README behavior drifts from the declaration** → A declaration-to-document consistency test blocks the change.
- **A captured stdout path loses the unfamiliar state** → Integration coverage invokes the same command-substitution boundary used by both runners.
- **Cross-repository changes land in the wrong order** → The CLI classification lands first, and an independent landing-order test covers the main runner with each change present alone.
- **The real E2E cannot naturally emit a synthetic fallback or future code** → QCG-12 uses isolated, scenario-specific `sno` builds for the PRD-authorized unclassified and eleventh-code controls, installs each selected build into the Chapter 0 path, and records its source revision before invoking both ordinary runners. All sidecar-originated outcomes still travel through the live sidecar and real store.

## Migration Plan

1. Add the declaration, all existing REM error mappings, uniqueness checks, and both error-to-exit lookups.
2. Separate unfamiliar-state handling from malformed-response handling.
3. Assign and verify the ten outcome exits and their diagnostic messages.
4. Update the README table and protect non-REM behavior.
5. After the tool emits the new codes, add the routing table and fail-closed arm to both runners. In `run_rem.sh`, move its own usage and rejected-operation exits to `20` and `21`, leaving the sibling-owned accepted names and message unchanged while changing the exit number in that block.
6. Extend the existing CLI and `run_rem.sh` traces for the unrecognised-state outcome, then run the real installed-tool, live-sidecar, real-store acceptance path.

Rollback is a single coordinated revert before release. No stored data or external published contract requires migration.

## Open Questions

None. The outcome matrix, fail-closed behavior, landing order, and sibling boundary are owner-settled.
