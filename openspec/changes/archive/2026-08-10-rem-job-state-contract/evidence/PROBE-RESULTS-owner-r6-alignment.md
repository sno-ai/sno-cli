# Probe results — owner R6 alignment

Date: 2026-08-09

## Released gate ledger

Command:

```sh
rg -o '\[QCG-[0-9]+\]' \
  '/home/lh/code/sno-station-core-edge-rem-wave/ai-doc/ACTIVE/PRD/[IMP]-edge-rem/80-rem-job-state-contract-prd.md' | sort -u
```

Result: exactly QCG-1 through QCG-17. No QCG-18 exists.

Command:

```sh
rg -n 'QCG-18|qcg-18|all 18|18 acceptance' \
  openspec/changes/rem-job-state-contract/proposal.md \
  openspec/changes/rem-job-state-contract/design.md \
  openspec/changes/rem-job-state-contract/specs/rem-job-state-contract/spec.md \
  openspec/changes/rem-job-state-contract/tasks.md
```

Result: no matches.

## Both callers under QCG-12

The released PRD QCG-12 names `run_rem.sh` and `run_rem_noop.sh`, requires each of the ten tool exits to drive the declared routing fate in both runners, and requires the unknown-code negative control. Task 7.12 names the same two ordinary callers, ten exits per runner, routing fate, real installed binary, live sidecar, real store, and the unknown-code control.

## Exact mapping

The delta specification records these exact tuples:

```text
0 success                           []
1 unclassified failure              []
2 invalid usage                     [usage_error]
3 job failed                        [rem_job_failed]
4 wait deadline passed              [rem_timeout]
5 state vocabulary mismatch         [rem_state_unrecognised]
6 malformed or truncated response   [sidecar_response_invalid, sidecar_response_truncated]
7 sidecar failure                    [sidecar_not_running, sidecar_unauthorized, sidecar_client_error, sidecar_discovery_error, sidecar_discovery_invalid, sidecar_response_error]
8 local environment failure          [profile_error, rem_trace_error]
9 unknown job identifier             [rem_job_not_found]
```

This matches PRD section 5.5 byte-for-byte on exit and error-code membership.

## Current exit sites

Commands:

```sh
rg -n 'error\.exit_code' src/cli.rs src/service.rs
rg -n 'CliError::runtime\(' src/rem.rs
```

Results: process conversion reads `error.exit_code` at `src/cli.rs:387` and `src/service.rs:97`. REM raise sites still call `CliError::runtime`, so section 2 has not been implemented yet.

Command:

```sh
rg -n 'profile_error|rem_job_failed|rem_job_not_found|rem_timeout|rem_trace_error|sidecar_client_error|sidecar_discovery_error|sidecar_discovery_invalid|sidecar_not_running|sidecar_response_error|sidecar_response_invalid|sidecar_response_truncated|sidecar_unauthorized' src --glob '*.rs' --glob '!rem.rs'
```

Result: `src/state.rs:110` raises the mapped string `profile_error` on a non-REM path. A global lookup by string would therefore violate QCG-4. The reconciled design requires a REM-specific typed constructor and leaves generic `CliError::runtime` unclassified.

## Numeric-contract consumers

The complete runtime-hit classification is preserved in `review-round-1-probes.md`. Its exact search covered current source plus historical and evidence matches. Reading the live hits found:

```text
/home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh:49-53,74-75,98-102,123
  Captures, branches on, and propagates rem-start/rem-status exit codes. Numeric consumer.
/home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh:51-55,76-77,99-103,124
  Same numeric capture and propagation shape. Numeric consumer.
/home/lh/code/sno-station-core-edge-rem-wave/evals/sno-memory-bench/deploy-mem-claw.sh:649-656
  Reads successful JSON job_id/state under set -e and never captures or branches on the numeric exit. Not a numeric consumer.
```

The harness selects the main runner at `evals/memora/evals/agent_eval/run_memora_mem_claw.sh:81`. Other search matches are tests, prose, archives, review evidence, or generic shell status handling unrelated to REM.

## Authoritative state vocabulary and writes

Commands:

```sh
rg -n 'REM_JOB_STATES|RemJobState|remJobStateSchema' \
  /home/lh/code/sno-station-core-edge-rem-wave/packages/rem-core/src/types.ts \
  /home/lh/code/sno-station-core-edge-rem-wave/apps/mem-claw/src/sidecar/rem-job-store.ts
rg -n "state: ['\"](queued|running|done|failed)|\.state = ['\"](queued|running|done|failed)|state:\s*RemJobState" \
  /home/lh/code/sno-station-core-edge-rem-wave/packages/rem-core/src \
  /home/lh/code/sno-station-core-edge-rem-wave/apps/mem-claw/src --glob '*.ts'
```

Results:

```text
packages/rem-core/src/types.ts:18  REM_JOB_STATES = [queued, running, done, failed]
packages/rem-core/src/types.ts:19  RemJobState derives from that tuple
apps/mem-claw/src/sidecar/rem-job-store.ts:12,55  the store type and schema use the same four values
apps/mem-claw/src/sidecar/rem-job-store.ts:180  writes queued
apps/mem-claw/src/sidecar/server.ts:284  writes running
apps/mem-claw/src/sidecar/server.ts:364,404,462  write done
apps/mem-claw/src/sidecar/server.ts:445,470  write failed
```

No fifth state write was returned.

## Existing trace schema, write points, and real sample

Command:

```sh
rg -n 'struct RemTrace|trace\.append|command_emitted|harness_cli_received|REM_TRACE_FILE|rem-trace' \
  src/rem.rs \
  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh \
  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
```

Results: `src/rem.rs:58-123` owns the CLI JSONL writer; final successful status currently writes `command_emitted` at `src/rem.rs:240-243`. `run_rem.sh:65,114` writes `harness_cli_received`. The noop runner has separate writes at lines 67 and 115 but no PRD-authorized trace change.

A real current binary probe used an isolated profile and state directory:

```sh
SNO_PROFILE_DIR=/tmp/rem-trace-probe.HAcZIB \
OPENCLAW_STATE_DIR=/tmp/rem-trace-probe.HAcZIB \
SNO_REM_CORRELATION_ID=corr-probe-019f8da3 \
target/debug/sno station rem-status job-probe-019f8da3 --json
```

It exited `1` with `sidecar_not_running` and wrote:

```json
{"timestamp":"2026-08-09T21:43:14.278627965+00:00","component":"sno_cli","event":"trace_opened","correlation_id":"corr-probe-019f8da3","trace_file":"/tmp/rem-trace-probe.HAcZIB/mem-claw/rem-trace.jsonl"}
{"timestamp":"2026-08-09T21:43:14.279210457+00:00","component":"sno_cli","event":"command_received","correlation_id":"corr-probe-019f8da3","command":"rem-status","job_id":"job-probe-019f8da3","wait":false,"timeout_seconds":60,"json":true}
{"timestamp":"2026-08-09T21:43:14.279479957+00:00","component":"sno_cli","event":"discovery_read","correlation_id":"corr-probe-019f8da3","discovery_file":"/tmp/rem-trace-probe.HAcZIB/station/sidecar.json","poll_index":1}
{"timestamp":"2026-08-09T21:43:14.279746967+00:00","component":"sno_cli","event":"discovery_failed","correlation_id":"corr-probe-019f8da3","discovery_file":"/tmp/rem-trace-probe.HAcZIB/station/sidecar.json","poll_index":1,"transient_class":"missing_discovery"}
```

## Release ancestry

Commands and results are preserved in `release-contract-probe.md`. The newest remote tag is `v0.1.7`; it predates commit `5130fee`, which introduced the REM commands, and no release tag contains that commit.

## Mechanical validation

Commands:

```sh
openspec validate rem-job-state-contract --strict
git diff --check
```

Results: both commands passed after the R6 reconciliation edits.
