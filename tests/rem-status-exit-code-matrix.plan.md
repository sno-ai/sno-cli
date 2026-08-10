# QCG-5 live-sidecar exit-code matrix plan

Behavior contract: QCG-5 in `80-rem-job-state-contract-prd.md` requires eight
observable `sno station rem-status` outcomes, ten repetitions each, with no
exit-code or machine-code interchange.

Test boundary: the current `sno` binary talks to the real Station Core sidecar
entry at `apps/mem-claw/src/sidecar/main.ts`, with a real temporary encrypted
SQLite persona store and temporary profile. A transparent test-only proxy is
admitted only for the two responses the current sidecar cannot natively emit.
It must forward the original GET to the live sidecar before changing the
response, and must retain hashes of both sides. This proves CLI classification;
it does not claim that the production sidecar natively emits either fault.

| Row | Changed guarantee | Realistic trigger | Lowest sufficient layer | Observable oracle | Existing coverage | Exact environment | Disposition |
|---|---|---|---|---|---|---|---|
| done | A completed job remains success | Real switched-off operation closes a durable live-sidecar job as `done` | E2E | exit 0, state `done`, no machine error | Unit fixtures do not cross the real entry/current binary boundary | real source entry, real temp store/profile, `--wait --json` | admit |
| failed | Sidecar-owned job failure maps only to its class | Missing temporary accepted grammar makes a real job terminal `failed` | E2E | exit 3 and `rem_job_failed` | Existing Rust fixture uses a substitute service | real source entry, real temp store/profile, `--wait --json` | admit |
| timeout | An unfinished real job maps only to deadline expiry | `SNO_REM_TEST_HOLD_MS=2000` keeps the real job running past one second | E2E | exit 4 and `rem_timeout` | Existing Rust fixture does not cross the real entry | real source entry, real temp store/profile, `--wait --timeout 1 --json` | admit |
| unrecognised | A well-formed unfamiliar state is distinct from malformed data | Proxy first receives a complete terminal `done` response from the live sidecar, then changes only `state` | E2E with explicit response fault injection | exit 5 and `rem_state_unrecognised`; one upstream call and one injection | Existing fixture uses a substitute service; sidecar schema makes native trigger unreachable | real source entry/store/profile plus transparent proxy, `--wait --json` | admit |
| truncated | A physically truncated body maps only to malformed/truncated response | Proxy first receives a complete terminal response, advertises the complete length, sends a strict prefix, then closes | E2E with explicit transport fault injection | exit 6 and `sidecar_response_truncated`; sent bytes less than declared | Existing fixture uses a substitute service; live server always sends complete JSON | real source entry/store/profile plus transparent proxy, non-waiting `--json` | admit |
| stopped | A stopped real sidecar maps only to unreachable | Start the real sidecar, record its PID/discovery, stop it, then query its profile | E2E | exit 7 and `sidecar_not_running`; recorded PID is gone | Existing fixture uses a substitute service | real source entry and temp profile, non-waiting `--json` | admit |
| profile | An unresolvable local profile maps only to environment error | Keep a real sidecar alive while removing `SNO_PROFILE_DIR`, `SNO_HOME`, `HOME`, and `USERPROFILE` from the CLI | E2E | exit 8 and `profile_error`; live PID remains alive | Unit coverage does not cross the current binary | real source entry/store plus deliberately unresolvable CLI environment | admit |
| unknown | A missing identifier maps only to not-found | Query ten unique absent IDs through the live sidecar | E2E | exit 9 and `rem_job_not_found`; upstream HTTP 404 trace | Existing fixture uses a substitute service | real source entry/store/profile, non-waiting `--json` | admit |

Execution contract:

- Smoke: at most ten observations, one per row plus two repeated fault rows.
- Full: eighty observations, ten per row, six sidecar starts.
- Freeze the current binary by path and SHA-256 in the run manifest.
- Expected full throughput: 1.45 observations/second; expected wall time: 55 seconds.
- Print one progress line per observation. At observation 20, stop below
  0.97 observations/second. Stop if projected total exceeds 82 seconds. Hard
  stop at 110 seconds.
- All runtime writes stay under one printed `mktemp` root. Evidence is copied
  before fixture cleanup; the temporary root is then removed.

Mock inventory: none. The transparent proxy is a declared fault injector after
a real upstream response, not a replacement sidecar or repository/client fake.
