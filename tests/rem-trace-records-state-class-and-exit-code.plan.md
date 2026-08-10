# QCG-14 trace tuple integration plan

## Provenance and scope

- Mode: Test Implementation, RED freeze before Section 6 product edits.
- Behavior source: released PRD REQ-18/QCG-14 and OpenSpec tasks 6.1-6.2.
- Writable scope: this test plan, its receipt, the QCG-14 test/helper, and QCG-14 evidence in
  `sno-cli`. Product source, runner source, README, tasks, Cargo files, and the external repository
  remain read-only.
- The test crosses the ordinary external `run_rem.sh` entry with the current compiled
  `target/debug/sno`. A loopback HTTP protocol fixture supplies one valid start response and one
  well-formed unfamiliar-state response. It is a deterministic response producer, not a claim that
  the production sidecar natively emits unfamiliar states.
- Mock inventory: empty. The asserted systems are the runner, compiled CLI, and their real JSONL
  trace writes. The loopback server supplies the sidecar protocol input authorized by the caller;
  no runner or CLI behavior is replaced.

## Scope admission

| Row | Changed guarantee and expected RED | Realistic reachability | Lowest sufficient proof | Observable oracle | Existing coverage | Exact command/environment | Self-screen |
|---|---|---|---|---|---|---|---|
| QCG-14 | After an unfamiliar state, both existing trace streams record the byte-identical raw state, outcome class `state vocabulary mismatch`, and numeric exit `5`. Expected RED: the ordinary runner already exits `5`, but neither the `sno_cli` nor `memora_harness` JSONL record contains the complete tuple. | A newer sidecar may return any non-empty state unknown to this CLI. The fixture returns a production-shaped job with a UTF-8 state containing misleading prose (`sidecar_response_invalid` and `exit 0`) so routing cannot correctly succeed by parsing message text. | Cross-process integration through the unchanged external `run_rem.sh`, current compiled `sno`, real filesystem traces, and a loopback HTTP response producer. A live-sidecar E2E is unnecessary because QCG-14 tests CLI/runner recording after the response is received, and the production sidecar schema cannot natively emit the target state. | Assert POST start then GET status reached the fixture with the expected token; runner invocation exits exactly `5`; parse the JSONL by `component`; require one `sno_cli` record and one `memora_harness` record each containing `raw_state` byte-identical to the fixture value, `outcome_class` exactly `state vocabulary mismatch`, and `exit_code` exactly `5`; require the runner still exits `5` despite the misleading prose. Missing tuple fields are the expected product RED. | QCG-5 proves unfamiliar-state CLI classification through a live-sidecar-backed fault injector but does not invoke `run_rem.sh` or assert either trace tuple. Existing trace tests assert successful state or trace fragments only. | `bash tests/rem-trace-records-state-class-and-exit-code.sh`; local loopback and temporary profile/state directories; no external writes or credentials. | admit |

## Bug-pattern screen

- Error propagation applies: exit `5` must survive the runner boundary and be recorded on both
  sides.
- Input validation applies only as a negative routing control: misleading state/message prose must
  not change the exit-code decision.
- Resource cleanup applies to the temporary server and directories; the shell wrapper owns bounded
  cleanup and a hard timeout.
- Concurrency, retries, numeric edges, partial persistence, and background lifecycle are unchanged
  and receive no quota-driven tests.

## Freeze contract

The plan hash receipt and Codex Reviewer row verdict must match these exact bytes before the test or
helper is created. RED is valid only if the fixture receives both ordinary requests, the runner
exits `5`, both JSONL streams parse, and the failure names missing tuple fields rather than syntax,
build, environment, or harness setup. The test/helper hashes and exact RED output are then frozen in
the QCG-14 evidence file before any Section 6 product edit.
