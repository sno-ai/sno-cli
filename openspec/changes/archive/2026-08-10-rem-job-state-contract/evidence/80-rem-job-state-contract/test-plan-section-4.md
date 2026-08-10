# Independent test plan — Section 4 REM CLI contract

## Provenance and mode

- Mode: Test Implementation.
- The independent test owner did not author or edit product/runtime source for this change.
- Assertions are derived from released REQ-3, REQ-4, QCG-3, and QCG-4 before inspecting product implementation rationale.
- Writable scope is limited to `tests/rem_job_state_contract.rs`, the stale README assertion inside
  `src/cli.rs`'s `#[cfg(test)]` module, and Section 4 test evidence.
- Product source, `README.md`, `tasks.md`, and Cargo files remain product-owned.
- Mock inventory: empty. The tests execute the compiled `sno` binary, a real temporary external
  executable, real loopback sockets, and temporary on-disk profile/discovery state.

## Scope admission

| Row | Changed guarantee and expected RED | Realistic trigger | Lowest sufficient proof and oracle | Existing coverage | Exact command | Self-screen |
|---|---|---|---|---|---|---|
| QCG-3 | The published REM exit-code documentation and the one declaration normalize to the same semantic rows; an in-memory documentation-only mutation is rejected. Expected RED: the current README says failed jobs and timeouts both exit `1` and has no ten-row contract. | Product declaration and documentation can land separately during this section. | One representation-tolerant source contract test extracts outcome name, exit, and error-code membership by semantic tokens rather than exact Rust or Markdown text, compares sorted normalized rows, and applies one test-owned README mutation as a negative control. | The existing `src/cli.rs` test pins the obsolete sentence only; QCG-1/2 prove declaration ownership/collisions but do not compare documentation. | `cargo test --test rem_job_state_contract qcg_3_readme_matches_declaration_semantic_rows_and_detects_drift -- --exact` | admit |
| QCG-4 | Generic runtime failures outside `station rem-*` remain exit `1`; both `rem-start` and `rem-status` against a stopped sidecar exit `7`. Expected RED: Section 2 changed REM mappings, while this four-actor family boundary has not been frozen. | An unreachable account service, a failing installed external command, and a stopped local sidecar are ordinary supported failure paths. | One compiled-binary integration journey: `account machine register --json` targets an unlistened loopback URL; `service fail-runtime` invokes a real executable that exits `1`; a temporary discovery file points to a loopback listener that is then stopped before both REM calls; a fresh loopback sidecar is restored and exercised before temporary state is dropped. Assert exact exits plus machine errors/sentinels, not implementation calls. | Existing tests separately cover an HTTP registration error, generic external exit preservation, and missing discovery. None proves these exact actors together or the stopped-sidecar equality after the new mapping. | `cargo test --test rem_job_state_contract qcg_4_non_rem_runtime_and_stopped_sidecar_exits_remain_distinct -- --exact` | admit |

## Bug-pattern screen

- Error propagation applies: two non-REM runtime failures must remain in the generic exit bucket,
  while stopped-sidecar failures must leave it.
- Resource lifecycle applies: the temporary sidecar listener is stopped before the assertions, a
  fresh sidecar is restored and exercised, and `TempDir` scopes cleanup to this test run.
- Input validation, retry exhaustion, concurrency, partial persistence, numeric edges, and
  background lifecycle are unchanged and receive no quota-driven tests.

## Freeze contract

- Both rows require an independent Codex Reviewer `admit` verdict against this exact plan hash
  before test-file edits.
- RED is valid only if the focused test compiles and QCG-3 reaches the current README mismatch, or
  QCG-4 reaches an observed product exit mismatch. Harness, syntax, fixture, and environment
  failures are not RED.
- After RED, the exact test bytes and exact focused commands are frozen. Product fixes may change
  only product-owned files. GREEN must rerun the same commands against the same test hash.
