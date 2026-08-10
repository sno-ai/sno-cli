# QCG-14 Section 6 probe results

Date: 2026-08-10

These are read-only facts gathered after deriving REQ-18/QCG-14 from the released PRD. They define
the final one-row test plan's feasibility and expected RED; they do not prescribe a product-source
implementation shape.

## Ordinary boundary and current artifacts

Commands:

```sh
target/debug/sno --version
sha256sum target/debug/sno \
  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
jq -c '.operations' \
  /home/lh/code/sno-station-core-edge-rem-wave/packages/rem-core/generated/rem-operations.json
```

Observed:

```text
sno 0.1.7
907189b03d0453c7975947583bf015948dd59cbb57b33d50bfb1e78276626667  target/debug/sno
8148307bf4178ff56637e634bbc66067a58e5f7155ff916ddec5c3ec08d0d1c7  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
["rem-update","rem-replace","rem-distill","rem-retire"]
```

`run_rem.sh` accepts `SNO_CLI_BIN`, `OPENCLAW_STATE_DIR`, and `SNO_PROFILE_DIR`; it invokes
`rem-start --json`, extracts the returned job id, then invokes waiting `rem-status`. It captures and
propagates both command exits. The focused fixture can therefore use the current compiled binary and
the ordinary runner without modifying either.

## Reachable loopback response

Commands:

```sh
sed -n '270,370p' src/rem.rs
sed -n '400,480p' src/rem.rs
```

Observed contract:

- Discovery is read from `$SNO_PROFILE_DIR/station/sidecar.json` as a port and non-empty token.
- Start sends authenticated `POST /rem/run` with `type` and `scope`, and accepts a JSON object with
  a non-empty `job_id`.
- Status sends authenticated `GET /rem/jobs/<job-id>` and decodes a production-shaped job object.
- Any non-empty state outside `queued`, `running`, `done`, and `failed` prints the raw state and
  produces `rem_state_unrecognised`, outcome class `state vocabulary mismatch`, and exit `5`.

A two-response loopback fixture is therefore the smallest realistic producer for QCG-14. It does
not replace runner or CLI logic, and it makes no production-sidecar-native-fault claim.

## Existing trace streams and missing tuple

Commands:

```sh
rg -n 'RemTrace|trace.append|command_emitted|poll_transient' src/rem.rs
rg -n 'harness_cli_received|exit_code|stdout' \
  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
rg -n 'raw_state|outcome_class' src/rem.rs \
  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
```

Observed:

- The CLI appends JSONL records with `component: "sno_cli"` to
  `$OPENCLAW_STATE_DIR/mem-claw/rem-trace.jsonl`.
- The runner appends JSONL records with `component: "memora_harness"` to that same durable JSONL
  path. These are the two existing logical trace streams required by REQ-18.
- The CLI writes `state` only in `command_emitted` after a successful status return. The unfamiliar
  branch returns before that record.
- The runner's `harness_cli_received` row records `exit_code` and captured stdout, but does not
  record a structured raw state or outcome class.
- Searching both implementations finds no `raw_state` or `outcome_class` field. Therefore the
  expected RED is a missing product tuple after the runner has genuinely propagated exit `5`, not a
  missing fixture route or unsupported CLI outcome.

## Existing coverage

`tests/rem-status-exit-code-matrix.mts` already proves exit `5` and
`rem_state_unrecognised` with a live-sidecar-backed response fault. It does not invoke the ordinary
runner and does not assert a CLI-side or runner-side trace tuple. Existing `tests/cli.rs` trace
coverage checks successful state and diagnostic fragments. QCG-14 is not duplicated or subsumed.
