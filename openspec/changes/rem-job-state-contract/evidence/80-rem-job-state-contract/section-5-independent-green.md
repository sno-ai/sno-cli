# Section 5 independent test GREEN freeze

## Authority and scope

- Independent test owner; no product-source authorship.
- Final Test Quality Gate after the product-source handback.
- Frozen plan SHA-256:
  `2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69`.
- Reviewer approval is not claimed. The bounded reviewer failures remain recorded separately;
  `test-plan-section-5-owner-ruling.md` authorized execution from the frozen plan.
- The test plan, shell harness, TypeScript helper, and fixture exactly match the Section 5 RED
  freeze. No test, runner, product, task, or README file was edited by the independent test owner.

## Frozen test artifact hashes

```text
2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69  openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.md
ad3c53e76024ae7237a9fb1e663aecc84ccc6ffec5175d066e1851ae48ae87ed  openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5-owner-ruling.md
acb6fa2e094bf7cd6c06738f40fbe4d5a52490fb28397640c4328ce6fe377af0  tests/rem-runner-routing-contract.sh
c6e9d792f3221b51d0100912ebe576f78ed5f5ad06d4a8e0e4e766f78c16206b  tests/rem-runner-routing-contract.mts
9d1e734f43fadb5a65f477e79237fb122d94b94efe3e4a94b7219fdcf04a8bde  tests/fixtures/rem-runner-routing-cases.json
2c5a1ce7c52bb016ab88632ff4a3dd38288b97e112ce8bf22697a1c088e4a647  openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/section-5-independent-red.md
```

The Section 5 RED evidence records the same four frozen test hashes. Their exact bytes did not
change between RED and GREEN.

## Product handback hashes exercised by GREEN

```text
36efcb14b5437f845370e2b9b3cf05e77d63943a447c4c615175d8971781dc6c  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
c1f66c318e6b1fd9e16db2113480f399f4dd05dac6cb7ca4dfab836b5ac449e7  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
```

## Ringing negative controls

Exact command:

```bash
set +e
for control in negative-qcg12 negative-qcg13 negative-qcg15 negative-qcg16 negative-qcg17; do
    echo "COMMAND: bash tests/rem-runner-routing-contract.sh $control"
    bash tests/rem-runner-routing-contract.sh "$control" 2>&1
    control_status=$?
    echo "EXIT: $control_status"
    if [ "$control_status" -ne 1 ]; then
        echo "NEGATIVE CONTROL HARNESS FAILURE: $control expected exit 1" >&2
        exit 97
    fi
done
exit 0
```

Outer command exit: `0`.

Exact final output:

```text
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg12
Section5 mode=negative-qcg12 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG12: known code 3 was logged as unmatched
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.de8cQX status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg13
Section5 mode=negative-qcg13 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG13: raw state missing: future terminal/β
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.Bnof9n status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg15
Section5 mode=negative-qcg15 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG15: same exit class changed fate after prose replacement: 5 != 1
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.n1nIAH status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg16
Section5 mode=negative-qcg16 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG16: runner originates tool-range literal: exit 7
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.iFzJKd status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg17
Section5 mode=negative-qcg17 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG17: landing orders used different stores: /tmp/store-a/persona.sqlite != /tmp/store-b/persona.sqlite
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.7gjEZ6 status=1
EXIT: 1
```

All five deliberately wrong inputs reached the intended behavior oracle and exited `1`. None
failed on build, import, environment, symbol resolution, timeout, throughput, or cleanup.

## Product-baseline GREEN

The exact frozen focused command was run first with unfiltered output:

```bash
bash tests/rem-runner-routing-contract.sh baseline
```

It exited `0`, completed `24/24` observations, and printed
`Section5 PASS QCG-12 QCG-13 QCG-15 QCG-16 QCG-17`.

The same focused command was then replayed on unchanged bytes for the compact evidence below.
`pipefail` preserves the focused command's exit while `sed` removes only incidental live-sidecar
INFO lines from the displayed evidence:

```bash
set -o pipefail
bash tests/rem-runner-routing-contract.sh baseline 2>&1 | sed -n -e '/^Section5 mode=/p' -e '/^future-tool-build /p' -e '/^progress /p' -e '/^Section5 boundary_reached /p' -e '/^Section5 hashes /p' -e '/^PRODUCT RED:/p' -e '/^Section5 PASS /p' -e '/^Section5 cleaned /p'
```

Exit: `0`.

Exact filtered output:

```text
Section5 mode=baseline expected_observations=24 estimated_rate_min=0.5_obs/s estimated_wall_max=90s hard_timeout=120s
future-tool-build mutation_sha256=98cc2637b5fbe44946e31708599f38d3ecaf4160d37b84a887b91e2bf5caf0ad binary_sha256=1a0a4d21363bf4b4da21fdae9901a99f37292d02f3486937c3dcef50b232beda
progress 1/24 case=section-first-code-0 exit=0 rate=0.18_obs/s eta=129.6s
progress 2/24 case=section-first-code-1 exit=1 rate=0.35_obs/s eta=62.3s
progress 3/24 case=section-first-code-2 exit=2 rate=0.53_obs/s eta=39.9s
progress 4/24 case=section-first-code-3 exit=3 rate=0.70_obs/s eta=28.6s
progress 5/24 case=section-first-code-4 exit=4 rate=0.87_obs/s eta=21.9s
progress 6/24 case=section-first-code-5 exit=5 rate=1.02_obs/s eta=17.7s
progress 7/24 case=section-first-code-6 exit=6 rate=1.18_obs/s eta=14.4s
progress 8/24 case=section-first-code-7 exit=7 rate=1.34_obs/s eta=11.9s
progress 9/24 case=section-first-code-8 exit=8 rate=1.50_obs/s eta=10.0s
progress 10/24 case=section-first-code-9 exit=9 rate=1.66_obs/s eta=8.4s
progress 11/24 case=section-first-code-10 exit=10 rate=1.82_obs/s eta=7.2s
progress 12/24 case=sibling-first-code-0 exit=0 rate=1.93_obs/s eta=6.2s
progress 13/24 case=sibling-first-code-1 exit=1 rate=2.08_obs/s eta=5.3s
progress 14/24 case=sibling-first-code-2 exit=2 rate=2.23_obs/s eta=4.5s
progress 15/24 case=sibling-first-code-3 exit=3 rate=2.38_obs/s eta=3.8s
progress 16/24 case=sibling-first-code-4 exit=4 rate=2.52_obs/s eta=3.2s
progress 17/24 case=sibling-first-code-5 exit=5 rate=2.61_obs/s eta=2.7s
progress 18/24 case=sibling-first-code-6 exit=6 rate=2.75_obs/s eta=2.2s
progress 19/24 case=sibling-first-code-7 exit=7 rate=2.89_obs/s eta=1.7s
progress 20/24 case=sibling-first-code-8 exit=8 rate=3.03_obs/s eta=1.3s
progress 21/24 case=sibling-first-code-9 exit=9 rate=3.17_obs/s eta=0.9s
progress 22/24 case=sibling-first-code-10 exit=10 rate=3.30_obs/s eta=0.6s
progress 23/24 case=qcg16-usage exit=20 rate=3.44_obs/s eta=0.3s
progress 24/24 case=qcg16-rejected exit=21 rate=3.58_obs/s eta=0.0s
Section5 boundary_reached observations=24/24 forwarded_requests=30 sidecar_pid=179473 store=/mnt/ramdisk/tmp/mem-claw-test-Ruphqn/test.sqlite
Section5 hashes plan=2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69 fixture=9d1e734f43fadb5a65f477e79237fb122d94b94efe3e4a94b7219fdcf04a8bde current_cli=e672158b54a190b9e71c4acb32f046c6d573e12b24b74efa31abb4b508b2f7f8 future_cli=1a0a4d21363bf4b4da21fdae9901a99f37292d02f3486937c3dcef50b232beda run_rem=36efcb14b5437f845370e2b9b3cf05e77d63943a447c4c615175d8971781dc6c run_rem_noop=c1f66c318e6b1fd9e16db2113480f399f4dd05dac6cb7ca4dfab836b5ac449e7
Section5 PASS QCG-12 QCG-13 QCG-15 QCG-16 QCG-17
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.MmjyiI status=0
```

At observation 20 the measured rate was `3.03 obs/s`, above the required `0.5 obs/s` kill
line. The complete run finished at `3.58 obs/s` and did not approach the hard timeout.

## Gate interpretation

| Gate | Final result |
|---|---|
| QCG-12 | Met: both runners propagated known `0..9`, classified future `10` as unmatched, and exposed identical enumerated `0..9` plus unknown routing tables used after both CLI captures. |
| QCG-13 | Met: both exit-5 status paths retained the raw unfamiliar state and version-mismatch sentence without invalid-response reclassification. |
| QCG-15 | Met: unrelated state prose changes did not alter the exit-5 fate, and each start trace retained `--json`. |
| QCG-16 | Met: runner-owned usage and rejected-operation failures returned `20` and `21`; tool-range statuses came from the real CLI captures and were preserved. |
| QCG-17 | Met: both absolute runner paths completed against one real sidecar PID and one encrypted persona store; old and new operation identifiers were observed separately. |

The real boundary comprised one source-sidecar fixture, one real encrypted persona store, a
temporary real CLI install, both absolute runner paths, `30` forwarded sidecar requests, and a
future CLI build that emitted status `10` only after a successful real `rem-start` request.

## Final test-owned checks

Commands:

```bash
bash -n tests/rem-runner-routing-contract.sh
shellcheck tests/rem-runner-routing-contract.sh
git diff --check -- tests/rem-runner-routing-contract.sh tests/rem-runner-routing-contract.mts tests/fixtures/rem-runner-routing-cases.json
/home/lh/code/sno-station-core-edge-rem-wave/node_modules/.bin/tsc --project /tmp/section5-rem-runner-tsconfig.json
rg -n "mock|Mock|monkeypatch|patch|Fake|stub|SimpleNamespace" tests/rem-runner-routing-contract.sh tests/rem-runner-routing-contract.mts tests/fixtures/rem-runner-routing-cases.json
```

The first four commands exited `0` with no diagnostic output. The search returned no matches;
the changed tests contain no mock, fake, stub, substitute driver, or substitute repository. The
temporary TypeScript configuration inherited the real Mem Claw application configuration, set
`rootDir` to `/`, enabled importing TypeScript extensions, disabled emit/declarations, and named
only `/home/lh/code/sno-cli/tests/rem-runner-routing-contract.mts`.

## Test Writer Gate: PASS

**Mode:** Final Test Quality

**Scope Reviewed:** Frozen Section 5 plan, shell harness, TypeScript helper, fixture, RED/GREEN
evidence, and QCG-12/QCG-13/QCG-15/QCG-16/QCG-17 runtime observations.

### Blockers

| Severity | Behavior Claim | Problem | Required Fix |
|----------|----------------|---------|--------------|
| None | None | None | None |

### Scope Admission

| Plan Row | Changed Guarantee | Realistic Reachability | Proof Layer | Existing Coverage | Feasibility | Test Writer | Codex Reviewer |
|----------|-------------------|------------------------|-------------|-------------------|-------------|-------------|----------------|
| QCG-12/13/15/16/17 combined runner journey | Deterministic routing, raw-state preservation, prose independence, exit provenance, and one-store landing orders | Both real runners, real CLI, real sidecar, real encrypted store, known and future statuses | External integration | Frozen Section 5 journey is the admitted proof | 24 observations completed under the bound | Admit; GREEN | Reviewer timed out; owner ruling superseded this repo gate |

Reviewed plan SHA-256:
`2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69`

### Coverage Map

| Behavior Claim | Test/Eval | Proof Type | Real Dependency | Observable Assertion | Status |
|----------------|-----------|------------|-----------------|----------------------|--------|
| QCG-12 | `baseline`, `negative-qcg12` | External integration plus ringing negative | Both runners, real CLI and sidecar | Known/future status routing and identical routing tables | PASS |
| QCG-13 | `baseline`, `negative-qcg13` | External integration plus ringing negative | Real exit-5 status responses and trace | Raw unfamiliar state and version sentence survive | PASS |
| QCG-15 | `baseline`, `negative-qcg15` | External integration plus ringing negative | Two real runner invocations | Prose replacement leaves fate unchanged; `--json` retained | PASS |
| QCG-16 | `baseline`, `negative-qcg16` | External integration plus ringing negative | Real runner-owned and CLI-owned failures | `20/21` locally; `0..10` captured and preserved | PASS |
| QCG-17 | `baseline`, `negative-qcg17` | External integration plus ringing negative | One real sidecar and encrypted store | Both landing orders share PID/store and preserve operation identifiers | PASS |

### Mock Inventory

| Mock Target | Why All 4 Conditions Are Met | Human Approval / Follow-up |
|-------------|------------------------------|-----------------------------|
| None | No mocks, fakes, stubs, substitute drivers, or substitute repositories | Not applicable |

### Required Commands

- `bash tests/rem-runner-routing-contract.sh baseline` -> proves the complete frozen GREEN journey.
- The five-mode negative-control loop above -> proves every principal oracle can ring on a known-wrong input.
- The four static commands above -> prove shell syntax/lint, patch cleanliness, and TypeScript validity.
