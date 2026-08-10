# Section 5 independent runner-contract test plan

## Provenance and boundary

- Mode: Test Implementation. This fresh test owner has not authored product source.
- Behavior source: released PRD REQ-14 through REQ-17 and REQ-19 through REQ-21, QCG-12,
  QCG-13, QCG-15, QCG-16, QCG-17, and OpenSpec tasks 5.1 through 5.4 only.
- Read-only systems under test:
  `/home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh` and
  `/home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh`.
- Planned test-owned files after admission:
  `tests/rem-runner-routing-contract.sh`, `tests/rem-runner-routing-contract.mts`, and
  `tests/fixtures/rem-runner-routing-cases.json`. Product source, both runners, both ordinary
  `run_memora_mem_claw.sh` files, and the existing dirty operation-switch test assets are forbidden.
- The main boundary directly invokes each named runner with a temporary installation of the current
  `sno` build. A transparent protocol fault injector first forwards every request to one live source
  sidecar using one real temporary encrypted SQLite persona store, then changes only the selected
  response for outcome production. The known-code matrix therefore exercises the real runner, real
  installed CLI, real socket, live sidecar, and real store. The unknown-code negative uses a
  temporary source snapshot of the same CLI with one recorded mutation that returns `10` only after
  a successful real `rem-start`; it is built and installed under the run root and never changes the
  repository.
- Mock Inventory: empty. The sidecar, CLI, runner, filesystem traces, and store are real. The proxy
  is a declared post-forward fault injector, and the future-code binary is the QCG-12 requested
  negative tool build; neither replaces an asserted runner decision.

## Scope admission

| Row | Changed guarantee and expected RED | Realistic reachability | Lowest sufficient proof | Observable oracle | Existing coverage | Exact command/environment | Self-screen |
|---|---|---|---|---|---|---|---|
| QCG-12 | Both runners use one enumerated table for captured `rem-start` and `rem-status` results, route every code `0..9`, and fail closed while logging unmatched `10`. Expected RED: known codes are merely propagated today, no enumerated router exists, and code `10` is not logged as unmatched. | Every named class is a published CLI result. A future CLI class is the explicit drift scenario. The injector forwards the real POST to the live sidecar/store before returning the selected published machine code; exit `5` is also produced at status by replacing only a completed job's state. | E2E at the two ordinary runner entrypoints. Static source reconciliation is added only for the explicit one-table/both-call-sites guarantee that black-box fates alone cannot distinguish from direct propagation. | For each runner, direct runs produce exact exits `0..9`; `0` proceeds and every other code fails the persona. Trace/request evidence proves the temporary installed real CLI reached the live sidecar/store. Source reconciliation finds one table covering exactly `0..9` plus unknown and both captured variables routed through it. A temporary future build emits `10` after a real start; each runner exits non-zero and logs `10` as unmatched, while that log is absent from every known-code run. | QCG-5 proves CLI codes, not either caller. QCG-14 proves one exit-5 trace through only `run_rem.sh`. No dual-runner router proof exists. | `bash tests/rem-runner-routing-contract.sh red`; one live sidecar/store; 24 direct runner observations plus structural reconciliation. | admit |
| QCG-13 | Exit `5` fails the persona and the log contains the byte-identical unfamiliar state and version-mismatch diagnosis, never invalid-response classification. Expected RED is coverage-only if current QCG-14 bytes already preserve the prose; it remains causally affected by adding the router. | A newer sidecar returning a non-empty terminal state unknown to this CLI is the published version-skew path. The proxy forwards a completed real job response, then changes only its state. | Same E2E status-path observation as QCG-12; no duplicate run. | Both runners exit `5`; combined output and trace contain `future terminal/β`, a sentence identifying sidecar/tool version mismatch, and no `sidecar_response_invalid` classification. | QCG-14 checks the tuple only for `run_rem.sh`; it does not cover noop routing or the QCG-13 operator log. | Reused exit-5 status observations inside `bash tests/rem-runner-routing-contract.sh red`. | admit |
| QCG-15 | Existing `--json` remains on the start call and message prose cannot affect routing. Expected RED is coverage-only if current behavior already satisfies it; the new router can make it red by inspecting prose or dropping the flag. | Error wording changes across versions while the stable exit code stays the same. | Runner integration inside the E2E fixture plus trace inspection; no second layer. | For each runner, two runs with the same exit code and unrelated message/state text have the same exit/fate. Parsed runner trace proves the start argv still contains `--json`, and the proxy transcript plus CLI result proves the only stable decision input is the exit code. | Existing trace tests observe argv fragments but do not replace prose and compare routing in both runners. | Reused matrix observations inside `bash tests/rem-runner-routing-contract.sh red`. | admit |
| QCG-16 | Only `run_rem.sh` owns usage `20` and rejected-operation `21`; it never originates `0..9`, and every returned code in that range is the immediately captured result of a real `sno` invocation. Expected RED: both owned paths currently exit `2`. | Missing arguments and operator typos are ordinary runner inputs. Every tool code is exercised by QCG-12. | Direct runner integration plus complete source/data-flow reconciliation. | No argument exits `20`; a rejected operation exits `21`; the accepted-name set and message bytes stay frozen. Shell source contains no literal runner exit in `0..9`. The full dynamic `0..9` matrix records Bash execution evidence showing each returned tool-range value comes from the adjacent installed-`sno` invocation and its immediate status capture, with no runner-originated path. | No current test proves either owned exit or all-path provenance. | Reused `bash tests/rem-runner-routing-contract.sh red`; `shellcheck` and deterministic source reconciliation on `run_rem.sh`. | admit |
| QCG-17 | Section-5-first keeps the sibling's old accepted names/message; sibling-first keeps every prior route; both landing orders run against the same store. Expected RED: the current section-5-first runner lacks routing, so the combined landing-order walk fails QCG-12 reconciliation even though its old names still validate. | The two documents modify disjoint lines of the same validation block and may merge in either order. The supplied dev and feature checkouts are the two real landing states. | Cross-checkout integration through the two named runners, one installed CLI, one live edge sidecar, and one real store. The transparent proxy translates only the old operation identifier before forwarding so the shared current sidecar can consume both published landing states; runner validation itself is untouched and observed before translation. | Directly invoke dev `run_rem_noop.sh` with its old accepted operation and feature `run_rem.sh` with the sibling operation against the same profile, sidecar PID, state root, and SQLite path. Both complete, create independently queryable jobs in that store, preserve the frozen accepted names/messages, and retain the QCG-12 routing result. | Dirty sibling QCG-25 files test operation switches, not Section 5 landing order or both runner consumers, and are not reused. | Reused success observations inside `bash tests/rem-runner-routing-contract.sh red`; one sidecar/store, both absolute runner paths. | admit |

## Bug-pattern screen and freeze contract

- Error propagation applies to all ten known results and the unmatched future result.
- Input validation applies only to `run_rem.sh`'s two owned failures.
- Resource lifecycle applies to the live sidecar, proxy, temporary installation, and store; the shell
  wrapper uses one validated `mktemp` root and removes only that root.
- Concurrency, retries, partial persistence, numeric values outside the declared code set, and
  background shutdown behavior are unchanged and receive no quota-driven tests.
- Production-shaped identifiers use unique `persona:section5-<uuid>` scopes and real UUID-backed
  correlations. No paid service or production credential is used.
- Expected full size: 24 direct runner observations, at least 24 forwarded sidecar requests, one
  sidecar start, and one temporary future-code build. Expected throughput is at least 0.5
  observations/second and wall time at most 90 seconds. The helper prints per-observation progress,
  checks throughput by 20 observations, kills if projected total exceeds 120 seconds, and the shell
  wrapper hard-stops at 120 seconds.
- RED is genuine only if preflight, temporary install, live sidecar/store, request forwarding, both
  absolute runner invocations, trace parsing, and cleanup succeed, then assertions fail specifically
  on missing enumerated routing/unmatched logging and current `2` versus required `20/21`.
- After RED, freeze the plan hash, fixture/helper/shell hashes, exact command, full output, runner
  hashes, installed binary hash, sidecar/store manifest, and expected product-failure fields. No
  GREEN is claimed until the product owner changes only runner source and the same frozen command is
  rerun.
