# Independent test plan — REM job-state contract, owner R6

## Provenance and mode

- Mode: Test Implementation.
- This fresh independent test owner has not authored or edited product or runtime source.
- Assertions come first from the released PRD, then from the approved R6-aligned OpenSpec artifacts
  and probes. The ledger is exactly QCG-1 through QCG-17; there is no QCG-18.
- QCG-12 covers both ordinary numeric-contract consumers, `run_rem.sh` and `run_rem_noop.sh`.
  QCG-16 and trace expansion apply only to `run_rem.sh`; the noop runner gains no private exit or
  trace contract.
- Test-owned files and evidence are the only writable scope. Product source, README, manifests,
  runner source, and OpenSpec proposal/design/spec/tasks remain product-owned.
- Existing Rust integration tests are the Section 2 harness. Later shell and real-sidecar checks use
  the repository's existing Bash boundary. No new runner or dependency is introduced.
- Mock inventory: empty. Section 2 uses the real compiled `sno` process and a loopback HTTP protocol
  fixture only to deterministically supply REM error responses; it does not claim E2E sidecar proof.

## Section 2 oracle under owner final ruling

The owner final ruling supersedes the rejected premise recorded by the three prior admission
reviews. No fourth review is run. Tests remain derived from the released PRD, not from a proposed
implementation shape.

One Rust integration target covers only QCG-1, QCG-2, QCG-6, and QCG-7:

1. QCG-1 reads the two released process-exit call sites and requires both to invoke the same
   resolver rather than directly reading `error.exit_code`. It also requires one grouped declaration
   containing the exact ten named classes, exits, and machine-code memberships. The resolver name,
   declaration file, Rust type, constructor, iterator, accessor spelling, and formatting are not
   fixed.
2. QCG-2 validates the exact PRD rows, then makes test-owned in-memory duplicate-exit and
   duplicate-error variants. Each must be rejected independently. Product files are never mutated.
3. QCG-6 sends each of the thirteen released REM error codes through the compiled `rem-start --json`
   process boundary and asserts its exact preserved machine code and mapped exit.
4. QCG-7 sends one mapped code and one absent future code through that same real process. The mapped
   code must not exit `1`; the absent code must exit exactly `1`. The declared rows must assign no
   error code and no named failure class to exit `1`.

The checked-in loopback HTTP server is a deterministic protocol fixture, not a substitute product
implementation or an E2E claim. Mock inventory remains empty.

## Scope admission

Every row below is screened as one whole row. Rows after Section 2 are frozen plan-only until their
named product section and real-boundary prerequisites exist.

| Row | Changed guarantee and expected RED | Realistic reachability | Lowest sufficient proof and observable oracle | Existing coverage search | Exact command / environment | Self-screen |
|---|---|---|---|---|---|---|
| QCG-1 | Both process-exit call sites resolve through the same declaration; error and exit values are entry-owned. Expected RED: both current sites directly read `error.exit_code`, and no declaration exists. | Every CLI/service error reaches one of the two named process boundaries. | Source contract test dynamically discovers a shared resolver and grouped semantic declaration without fixing its name, file, Rust type, or formatting. | No declaration-ownership proof exists. | `cargo test --test rem_job_state_contract qcg_1_single_declaration_owns_codes -- --exact` | admit — owner final ruling |
| QCG-2 | Duplicate exit and duplicate error membership each fail independently. Expected RED: no declaration exists to validate. | Both are ordinary declaration edit mistakes. | Exact PRD row validator plus separate in-memory duplicate-exit and duplicate-error controls; no product mutation. | No collision proof exists. | `cargo test --test rem_job_state_contract qcg_2_duplicate_exit_and_error_are_rejected -- --exact` | admit — owner final ruling |
| QCG-3 | README and declaration have identical normalized semantic rows. | Code and docs can land separately. | A representation-neutral README comparison has no executable source until Section 2 establishes the shipped source representation; it must receive a new reviewed plan before implementation. | Existing README test checks old prose only. | Not executable in this turn. | reject — later admission required |
| QCG-4 | Released non-REM actors `sno account` and `sno service` retain runtime exit `1`; stopped-sidecar `rem-start` and `rem-status` both exit `7`. | All four named actors are required, but exact repeatable runtime-failure argv for both non-REM actors is not yet frozen. | Binary integration is the right boundary; usage failures are not substitutes. | Existing cases do not cover the exact four-actor discriminator. | Not executable until actor probes exist. | reject — actor probes required |
| QCG-5 | The exact ten outcomes produce exits `0..9` and matching codes over ten repetitions. | Real-sidecar scenario producers and Chapter 0 are absent. | Installed CLI/live sidecar/store remains required. | Existing test covers old `0/1/2` only. | Not executable in this turn. | reject — Chapter 0 and fixtures required |
| QCG-6 | Every one of the thirteen released REM codes has its exact mapping. Expected RED: the compiled process currently exits `1` for every injected known code. | Existing `rem-start` accepts and preserves a sidecar-supplied machine code. | Thirteen real compiled-process cases assert exact JSON code and process exit. | No exhaustive mapping test exists. | `cargo test --test rem_job_state_contract qcg_6_all_raisable_rem_codes_are_mapped -- --exact` | admit — owner final ruling |
| QCG-7 | No named outcome or mapped error owns exit `1`; an absent future code exits exactly `1`. Expected RED: `rem_timeout` still exits `1`, so known and absent codes are indistinguishable. | Both known and absent codes cross the same supported compiled-process boundary. | Exact known-versus-absent process comparison plus declaration invariant. | Existing tests prove collapsed `1`, not fallback-only behavior. | `cargo test --test rem_job_state_contract qcg_7_exit_one_is_unclassified_only -- --exact` | admit — owner final ruling |
| QCG-8 | Non-waiting queued/running/done print state and exit `0`; failed exits `3`; unfamiliar non-empty state prints and exits `5`. Expected RED: failed/unfamiliar are still collapsed or invalid. | Ordinary polling and version skew are supported paths. | Reuse existing running/done Rust proof; add only missing queued, failed, and unfamiliar binary assertions at the same layer. | `rem_one_shot_status_reads_running_then_stable_done` already supplies running/done. | `cargo test --test rem_job_state_contract qcg_8_non_waiting_status_outcomes -- --exact` after Section 3 | admit |
| QCG-9 | Waiting unfamiliar state exits immediately; captured stdout preserves the state byte-for-byte before the error; the message names job and version skew. | A newer sidecar can return an unknown state, but no real-sidecar producer or Chapter 0 receipt exists. | Installed CLI and deliberate real-sidecar scenario build remain required. | No equivalent proof. | Not executable in this turn. | reject — Chapter 0 and fixture required |
| QCG-10 | Invalid JSON and empty state yield `sidecar_response_invalid/6`; unfamiliar non-empty state does not. Expected RED: unfamiliar state shares the invalid-response class. | Corruption, empty state, and vocabulary skew are supported protocol cases. | Response-boundary Rust integration asserts exact code and exit for all three. | Existing truncated retry test does not prove this classification. | `cargo test --test rem_job_state_contract qcg_10_invalid_response_is_narrow -- --exact` after Section 3 | admit |
| QCG-11 | Failed job preserves supplied sidecar error text and omits it only when absent. Expected RED: new exit `3` is absent and the no-detail case is uncovered. | The job record's error is optional. | Binary integration asserts exact exit and message presence/absence. Existing supplied-detail coverage is reused, not duplicated. | Existing human-output test covers supplied text only. | `cargo test --test rem_job_state_contract qcg_11_failed_job_keeps_sidecar_error -- --exact` after Section 3 | admit |
| QCG-12 | Both ordinary runners route tool exits `0..9`; an eleventh code fails closed and is logged. | Both are consumers, but 22 real-boundary cases lack Chapter 0 and scenario producers. | Both ordinary runners, installed binary, live socket, and real store remain mandatory; noop gets no private exit or trace. | No dual-runner E2E proof exists. | Not executable in this turn. | reject — Chapter 0 and fixtures required |
| QCG-13 | Exit `5` fails the persona and logs raw state/version skew, never invalid-response prose. | Real version skew is supported but its producer is absent. | Runner capture/log boundary after QCG-12 fixture admission. | QCG-12 does not prove text. | Not executable in this turn. | reject — fixture required |
| QCG-14 | CLI and `run_rem.sh` traces contain raw state, class, and exit. | Depends on the absent unfamiliar-state journey. | Parse both JSONL records; no noop trace obligation. | Existing tests assert fragments only. | Not executable in this turn. | reject — fixture required |
| QCG-15 | `run_rem.sh` keeps JSON flags and routes identically when only prose changes. | No two source-labelled builds differing only in prose are frozen. | Ordinary runner boundary is correct once those builds exist. | Existing argv proof is insufficient. | Not executable in this turn. | reject — controlled builds required |
| QCG-16 | `run_rem.sh` owns exits `20/21`, no runner-owned literal `0..9`, and every returned tool-range code is immediate `sno` propagation. | The two owned failures are reachable, but a shape-neutral all-path data-flow oracle is not yet frozen. | Black-box branch matrix or a mechanically constrained narrow propagation structure must be reviewed before implementation. | Existing lifecycle tests prove only non-zero. | Not executable in this turn. | reject — provenance oracle required |
| QCG-17 | Either cross-repo change may land first against the same store while preserving names/messages and exit routing. | Rollout is realistic, but revisions, sibling patch, shared store, and Chapter 0 commands are not frozen. | Two recorded temporary checkouts against one real store remain required. | No landing-order proof exists. | Not executable in this turn. | reject — revisions and Chapter 0 required |

## Bug-pattern screen

- Input validation applies only to `run_rem.sh`'s two owned failures under QCG-16.
- Retry exhaustion is already represented by timeout classification under QCG-5; no duplicate
  retry test is added.
- Error propagation applies to unknown REM codes under QCG-7 and unknown runner codes under QCG-12.
- Resource lifecycle applies to QCG-4 stop/restore and QCG-17's shared-store walk.
- Concurrency, numeric edges, partial persistence, and background lifecycle are unchanged and get no
  quota-driven tests.

## Stage and freeze rule

The whole seventeen-row plan and its hash receipt must receive row-by-row Codex Reviewer PASS with
zero material blockers before any test file changes. After PASS, this turn implements only QCG-1,
QCG-2, QCG-6, and QCG-7. Each focused RED record contains the exact command, expected product
failure, observed product failure, test SHA-256, reviewed plan SHA-256, and proof that product source
did not change. Later rows remain plan-only.
