# Section 3 Independent Test Plan

Mode: Test Implementation (RED freeze only)

Independent provenance: this plan was derived by the fresh test owner from
REQ-5, REQ-9 through REQ-14, QCG-5, and QCG-8 through QCG-11 before reading
product implementation rationale or source. Product source is out of test
ownership.

## Scope admission

| Row | Changed guarantee | Realistic trigger / reachability | Lowest sufficient proof layer | Observable oracle | Existing coverage search | Exact command / environment | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S3-1 / QCG-5 local state rows | Across ten repetitions, section 3's reachable status outcomes do not interchange: done exits 0, failed is `rem_job_failed` exit 3, a live loopback job that remains running reaches `rem_timeout` exit 4, a non-empty unfamiliar state is `rem_state_unrecognised` exit 5, and malformed state responses are `sidecar_response_invalid` exit 6. | Invoke the real compiled `sno` binary with production-shaped responses over the existing loopback TCP/HTTP fixture. Run each of these five status-path rows ten times. The other QCG-5 classes are outside this section's causal RED and stay with the already frozen QCG-1/2/6/7 and later release acceptance. | CLI contract integration at the sno-cli state-handling boundary. This is not the cross-repository live-sidecar E2E named by QCG-5. | For every repetition, assert the exact exit and machine error code (or successful state for exit 0), and assert no observed result belongs to another admitted state row. | `qcg_1`, `qcg_2`, `qcg_6`, and `qcg_7` already prove declaration ownership, mutation rejection, mapping completeness, and exit-1 semantics. `tests/cli.rs::rem_exit_codes_are_stable` covers done/failed/timeout once but does not repeat or prove the unfamiliar/invalid split. | `cargo test --test rem_job_state_contract qcg_5_ -- --nocapture`; local filesystem and loopback only, no external repo mutation. | admit only for the five section-3 state rows; do not add tests for the other five classes here |
| S3-2 / QCG-8 | Non-waiting queued/running/done and waiting queued/running/done remain successful and print their states. | A real compiled `sno` process reads a sequence of production-shaped job records from the loopback socket. | CLI contract integration. | Non-waiting queued exits 0 and prints queued. Reuse `tests/cli.rs::rem_one_shot_status_reads_running_then_stable_done` for non-waiting running/done. A waiting call observes queued then running then done, exits 0, and prints done. | Existing CLI test exactly covers non-waiting running/done; no existing test covers queued or the waiting transition sequence. | `cargo test --test rem_job_state_contract qcg_8_ -- --nocapture` plus `cargo test --test cli rem_one_shot_status_reads_running_then_stable_done -- --exact`. | admit only for missing queued/wait coverage; reuse existing running/done coverage |
| S3-3 / QCG-9 | In both modes, any non-empty unfamiliar state is immediately classified as `rem_state_unrecognised` exit 5; the raw byte sequence reaches stdout before the error, and the message names the job, state, and version skew. Command substitution retains stdout. | A real compiled `sno` process receives a well-formed job containing a non-ASCII, whitespace-bearing unfamiliar state. Separate real invocations cover non-wait, wait, merged-stream ordering, JSON machine code, and `/bin/sh` command substitution. | CLI contract integration plus caller-boundary shell capture. | Assert exit 5; unfamiliar state on stdout; merged output positions state before error; JSON output contains `rem_state_unrecognised` and not `sidecar_response_invalid`; message contains the job id, byte-identical state, and the sentence that the sidecar reported a state this build does not know; shell-captured stdout contains the state and preserves status 5. | No existing test covers an unfamiliar state, cross-stream order, or command substitution. | `cargo test --test rem_job_state_contract qcg_9_ -- --nocapture`; `/bin/sh` only on Unix. | admit |
| S3-4 / QCG-10 | Invalid JSON, absent state, and empty state are `sidecar_response_invalid` exit 6, while every non-empty unfamiliar state is not invalid. | A real compiled `sno` process receives each malformed or well-formed response over the loopback socket in both waiting and non-waiting mode where applicable. | CLI contract integration. | Assert exact exit and machine code for malformed JSON, missing state, and empty state. Assert unfamiliar-state results contain `rem_state_unrecognised`, exit 5, and never contain `sidecar_response_invalid`. | Existing truncated-response retry coverage tests restart recovery, not terminal classification; no existing missing/empty state coverage. | `cargo test --test rem_job_state_contract qcg_10_ -- --nocapture`. | admit |
| S3-5 / QCG-11 | `rem_job_failed` preserves the sidecar-provided error string, and that exact string is absent only when the sidecar omits the field. | Two real compiled `sno` invocations receive otherwise identical failed records, one carrying a unique sentinel error and one omitting `error`. Waiting and non-waiting permutations are not added because both use the same failed-state branch. | CLI contract integration. | Both results carry job id, `rem_job_failed`, and exit 3. The provided result contains the sentinel byte-for-byte; the omitted result does not contain that same sentinel. No assertion constrains a generic fallback that REQ-14/QCG-11 do not specify. | Full-body probe confirms an existing test covers one provided-error example, but the paired same-sentinel present/absent comparator required by the owner is not present. Source probe confirms no mode permutations are needed. | `cargo test --test rem_job_state_contract qcg_11_ -- --nocapture`. | admit the paired same-sentinel comparator by owner contract ruling; no mode permutations |

## Bug-pattern screen

- Input validation: admitted in S3-4 because response parsing/state validation is the changed mechanism and the loopback responses reach it directly.
- Error propagation: admitted in S3-1, S3-3, S3-4, and S3-5 because the changed CLI boundary owns the exact machine code, message, and process exit.
- Retry/deadline behavior: admitted only where REQ-5 and the waiting state matrix require it; no general retry tests are added.
- Resource lifecycle, concurrency, partial failure, numeric edges, and background lifecycle: rejected because section 3 does not change those mechanisms.

## Mock / substitute inventory

| Boundary | Status and reason | Claim limit |
| --- | --- | --- |
| Existing `SnoServiceServer` loopback socket fixture | Owner-directed reuse of the existing deterministic HTTP fixture. It is a substitute for the external sidecar and therefore is not presented as live-sidecar E2E. | Proves only the real compiled sno CLI's wire-response handling and observable process behavior. It cannot close QCG-5's cross-repository live-sidecar release gate. |

No product module, symbol, function, clock, process result, filesystem, or `sno`
binary is mocked. No new dependency or test runner is introduced.

## Expected genuine RED before product change

- A non-empty unfamiliar state is currently expected to be reported as
  `sidecar_response_invalid` exit 6, or to omit the required stdout/message
  behavior, instead of `rem_state_unrecognised` exit 5.
- At least one invalid-response case may be conflated with the unfamiliar-state
  path.
- The frozen tests must compile and start the real `sno` binary; a missing
  symbol/module/fixture failure is a test defect, not accepted RED.

QCG-8 and QCG-11 may start GREEN as coverage-only rows. They are not presented
as TDD RED when the current product already satisfies their exact oracle.

## External boundary

The outer QCG-5 phrase "against a live sidecar" and its complete ten-class
matrix require the real sidecar in
the owning external repository. This section does not edit or launch that
repository. The sno-cli RED can be frozen here; release-level QCG-5 remains
uninstantiated until a later independent E2E run crosses that boundary.
