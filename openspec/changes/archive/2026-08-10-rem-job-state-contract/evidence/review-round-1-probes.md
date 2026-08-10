# Review round 1 probes

Date: 2026-08-09

These are read findings from the repositories, not compatibility rulings. No product source was changed while gathering them.

## Repository-wide caller search

Command run exactly:

```sh
rg -n --hidden --glob '!target/**' 'sno station rem-(start|status)|station rem-(start|status)' /home/lh/code
```

The search returned runtime invocations, tests, documentation, archived evidence, temporary clones, and Git rerere records. Reading every current runtime hit produced this contract-consumer inventory:

| Path and lines | Read finding | Numeric-contract status |
|---|---|---|
| `/home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh:49-53,74-75,98-102,123` | Captures `rem-start` into `START_EXIT`, captures `rem-status` into `STATUS_EXIT`, branches on the start code, and propagates both codes. | Consumer; included. |
| `/home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh:51-55,76-77,99-103,124` | Has the same exit-capture, branch, and propagation shape as the main runner. | Consumer; included with its own acceptance. |
| `/home/lh/code/sno-station-core-edge-rem-wave/evals/sno-memory-bench/deploy-mem-claw.sh:649-656` | Captures successful JSON, reads `.job_id` and `.state` with `jq`, requires `state == "done"`, and dies on any command failure under `set -e`. It never stores, compares, routes, or propagates the numeric exit code. | Command invoker, not a numeric-contract consumer; excluded on this read finding. |

The matching `deploy-mem-claw.sh` in `/home/lh/code/sno-station-core` has the same lines and behavior. Other current-tree matches are tests or prose. Matches under `.dingo-*`, `.git/rr-cache`, `openspec/changes/archive`, and evidence directories are snapshots or records, not additional live callers.

Source revisions read:

- `/home/lh/code/sno-cli`: `53369f0f63c7ae8adafb87705326436cead9e1b2`
- `/home/lh/code/sno-station-core`: `5d75c4d61f426dd5a83d540e78f38a3428688204`
- `/home/lh/code/sno-station-core-edge-rem-wave`: `594eab20eaf479ddb2babf7b6177110ec7ceadaf`

## Runner-owned exits

Both consumers currently originate the tool-owned code `2`:

- `run_rem.sh:6` uses `exit 2` for its usage error and `run_rem.sh:18` uses `exit 2` for a rejected operation.
- `run_rem_noop.sh:6` uses `exit 2` for its usage error and `run_rem_noop.sh:19` uses `exit 2` for a rejected operation.

Therefore both need the same disjoint runner-owned exits: usage `20`, rejected operation `21`. The accepted operation names and validation messages are not derived by this finding and remain outside this change's ownership.

## Authoritative state vocabulary

Commands run:

```sh
rg -n 'REM_JOB_STATES|RemJobState|remJobStateSchema' packages/rem-core/src/types.ts apps/mem-claw/src/sidecar/rem-job-store.ts
rg -n 'pub state: String|match job.state.as_str' /home/lh/code/sno-cli/src/rem.rs
```

Read findings:

- `packages/rem-core/src/types.ts:18-19` declares exactly `queued`, `running`, `done`, and `failed`.
- `apps/mem-claw/src/sidecar/rem-job-store.ts:12,41-46` declares and validates the same four values.
- `sno-cli/src/rem.rs:46-47` deliberately receives state as `String`, and `src/rem.rs:425-443` classifies it locally.
- No authoritative terminal-state field exists. The implementable fail-closed rule is therefore mode-independent: every unfamiliar non-empty state exits `5`; missing, empty, or undecodable state exits `6`.

## Existing trace structure and write points

Commands run:

```sh
rg -n 'RemTrace|trace.append|command_emitted|poll_transient' /home/lh/code/sno-cli/src/rem.rs
rg -n 'harness_cli_received|exit_code|stdout' /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
```

Read findings:

- `sno-cli/src/rem.rs:58-123` defines an extensible JSONL trace record and writer.
- `sno-cli/src/rem.rs:239-243` records `state` only after a successful status result. Failure returns at `src/rem.rs:418-454` occur before that row, and discovery, authentication, transport, and parse failures at `src/rem.rs:466-546` may never obtain a job state.
- `run_rem.sh:56-70,105-119` and `run_rem_noop.sh:58-72,106-120` record stdout and exit code but no outcome class or normalized state availability.

These read findings establish that the unrecognised-state outcome needs a new trace record rather than a rename of the success-only field. The released PRD limits QCG-14 to the raw state, outcome class, and exit code in the CLI and `run_rem.sh` traces for that path; this probe does not extend the trace contract beyond it.
