# Section 5 runner-contract probe results

Probe date: 2026-08-10 UTC. These commands were run before any Section 5 test file was written.

## Ordinary caller

```text
rg -n -C 6 'REM_RUNNER|run_rem_noop|run_rem\.sh' \
  /home/lh/code/sno-station-core-edge-rem-wave/evals \
  /home/lh/code/sno-station-core/evals --glob '*.sh'
```

Both ordinary harnesses assign their checkout's runner to `REM_RUNNER` and invoke it as
`"$REM_RUNNER" "$USER_ID" 2>&1 | tee -a "$LOG_FILE"`. A non-zero result marks the persona failed.
The Section 5 test can therefore invoke each runner directly without invoking or modifying either
dirty `run_memora_mem_claw.sh`.

## Frozen runner bytes and current exits

```text
sha256sum /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh \
  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
ae77cbeb852f23ae87f35cca8128d57fa3ece8456062fc6589f04493499cf084  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
e56ba30ccd0ef3488ad759febe0a069d365c1234c4dd3ad3b1ba416c2ab050d8  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
```

Each working-tree hash equals `git show HEAD:<path>` in its own checkout. Neither runner has a
working-tree diff.

```text
/home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
status=2
stderr=Usage: /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh <persona-scope>

OPENCLAW_STATE_DIR=/tmp/qcg-section5-probe-state \
SNO_PROFILE_DIR=/tmp/qcg-section5-probe-profile \
MEM_CLAW_REM_TYPE=definitely-not-declared \
/home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh persona:section5-probe
status=2
stderr=MEM_CLAW_REM_TYPE is not a declared REM operation: definitely-not-declared
```

This is the expected QCG-16 product RED: the required runner-owned results are `20` and `21`.

## Exit capture and routing baseline

```text
rg -n 'case .*EXIT|route|routing|START_EXIT|STATUS_EXIT|exit "\$' \
  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh \
  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
```

Both runners capture the status immediately after the real `SNO_CLI_BIN` command into
`START_EXIT` and `STATUS_EXIT`, then return those variables directly. Neither file contains an
enumerated router or an unmatched-code log. `run_rem.sh` still records `--json` in the existing
start argv trace and invokes `rem-start ... --json`.

## Real boundary and feasibility

The repository already has a production-entry fixture at
`/home/lh/code/sno-station-core-edge-rem-wave/tests/apps/mem-claw/helpers/rem-production-entry-fixture.ts`.
It launches the source sidecar over a real loopback socket, writes real discovery, uses a temporary
encrypted SQLite persona store, and exposes its profile/state roots. The existing QCG-5 test proves
the current compiled CLI can use that boundary and a transparent proxy after forwarding a real
upstream request. Required local tools are present: `jq`, `node`, `cargo`, `rustc`, `timeout`,
`sha256sum`, `shellcheck`, the checkout's `tsx`, the sidecar entry, and the generated operation list.

## Existing-coverage search

QCG-5 covers CLI outcome classification but does not invoke either runner. QCG-14 invokes only
`run_rem.sh` for one exit-5 trace tuple. No existing test invokes both named runners over exits
`0..9`, asserts an unmatched eleventh code is logged, proves `20/21`, or walks both landing orders
against one store. The pre-existing dirty operation-switch files are sibling-owned QCG-25 assets;
this Section 5 plan neither edits nor treats them as its proof.
