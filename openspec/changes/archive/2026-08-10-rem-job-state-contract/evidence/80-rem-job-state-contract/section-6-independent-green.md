# Section 6 independent GREEN

## Verdict

PASS. The same independent test owner reran the byte-frozen QCG-14 integration test after the
product handback. The exact RED command is now GREEN without changing the plan, shell test, or
TypeScript helper.

## Frozen chain

| Artifact | SHA-256 | Result |
|---|---|---|
| `tests/rem-trace-records-state-class-and-exit-code.plan.md` | `27536c728d7733338ac557531f1ec7b58f8f02db180cc835dd313655a2092a48` | Matches reviewed receipt and RED freeze |
| `tests/rem-trace-records-state-class-and-exit-code.plan.sha256` | `e108424a108c01777d040d856ad94f500ea9387ddc79bdb902f0eb90ede671d0` | Unchanged |
| `tests/rem-trace-records-state-class-and-exit-code.sh` | `c271ebc024cc4aef53ed85b0e483adc0e8d09fbba21780a489ba6d37c70293b9` | Unchanged |
| `tests/rem-trace-records-state-class-and-exit-code.mts` | `0e9988bd645348bdaa278641bf2beea51c917adf103954783181275701263618` | Unchanged |

## Observable GREEN

Command:

```sh
bash tests/rem-trace-records-state-class-and-exit-code.sh
```

```text
$ bash tests/rem-trace-records-state-class-and-exit-code.sh
status=0
QCG-14 expected_observations=1 estimated_rate=1_obs/s estimated_wall=1s hard_timeout=20s
QCG-14 boundary_reached requests=2 runner_exit=5 trace_records=18
QCG-14 PASS raw_state="future terminal/β: sidecar_response_invalid; exit 0" outcome_class="state vocabulary mismatch" exit_code=5
QCG-14 cleaned runtime_root=/tmp/sno-qcg14-trace-tuple.MSQjie
```

Exit: `0`

```text
QCG-14 expected_observations=1 estimated_rate=1_obs/s estimated_wall=1s hard_timeout=20s
QCG-14 boundary_reached requests=2 runner_exit=5 trace_records=18
QCG-14 PASS raw_state="future terminal/β: sidecar_response_invalid; exit 0" outcome_class="state vocabulary mismatch" exit_code=5
QCG-14 cleaned runtime_root=/tmp/sno-qcg14-trace-tuple.MSQjie
```

The ordinary runner reached both protocol requests and propagated tool exit `5`. The persisted
`sno_cli` and `memora_harness` trace records each carried the byte-identical raw state, outcome
class `state vocabulary mismatch`, and numeric exit code `5` together. This satisfies REQ-18 and
QCG-14 at the admitted external boundary.

## Test quality checks

- Shell syntax: PASS.
- Strict TypeScript typecheck: PASS.
- Biome check: PASS.
- Forbidden mock/fake/stub search: zero matches.
- Mock Inventory: empty.
- Full command output and boundary hashes: `qcg-14.txt`.
- Original genuine RED: `qcg-14-red.txt`.
