# Section 5 independent test RED freeze

## Authority and scope

- Independent test owner; no product-source authorship.
- Frozen plan SHA-256:
  `2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69`.
- Reviewer approval is not claimed. The bounded reviewer failures are recorded separately; the
  owner ruling in `test-plan-section-5-owner-ruling.md` authorized this implementation.
- Neither runner, product source, tasks, README, ordinary harness, nor dirty sibling test asset was
  edited.

## Frozen test artifact hashes

```text
acb6fa2e094bf7cd6c06738f40fbe4d5a52490fb28397640c4328ce6fe377af0  tests/rem-runner-routing-contract.sh
c6e9d792f3221b51d0100912ebe576f78ed5f5ad06d4a8e0e4e766f78c16206b  tests/rem-runner-routing-contract.mts
9d1e734f43fadb5a65f477e79237fb122d94b94efe3e4a94b7219fdcf04a8bde  tests/fixtures/rem-runner-routing-cases.json
2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69  openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.md
ad3c53e76024ae7237a9fb1e663aecc84ccc6ffec5175d066e1851ae48ae87ed  openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5-owner-ruling.md
ae77cbeb852f23ae87f35cca8128d57fa3ece8456062fc6589f04493499cf084  /home/lh/code/sno-station-core-edge-rem-wave/evals/memora/scripts/run_rem.sh
e56ba30ccd0ef3488ad759febe0a069d365c1234c4dd3ad3b1ba416c2ab050d8  /home/lh/code/sno-station-core/evals/memora/scripts/run_rem_noop.sh
```

The final `git diff --quiet -- <runner>` status was `0` in both external checkouts.

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

Exact final output:

```text
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg12
Section5 mode=negative-qcg12 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG12: known code 3 was logged as unmatched
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.A0wU2S status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg13
Section5 mode=negative-qcg13 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG13: raw state missing: future terminal/β
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.4lhhiF status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg15
Section5 mode=negative-qcg15 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG15: same exit class changed fate after prose replacement: 5 != 1
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.NYrGys status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg16
Section5 mode=negative-qcg16 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG16: runner originates tool-range literal: exit 7
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.kwHL7J status=1
EXIT: 1
COMMAND: bash tests/rem-runner-routing-contract.sh negative-qcg17
Section5 mode=negative-qcg17 expected_observations=1 estimated_wall_max=5s hard_timeout=120s
NEGATIVE CONTROL RED QCG17: landing orders used different stores: /tmp/store-a/persona.sqlite != /tmp/store-b/persona.sqlite
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.3BuszX status=1
EXIT: 1
```

All five known-wrong inputs reached their intended oracle. None failed on build, import,
environment, symbol resolution, timeout, or cleanup.

## Product-baseline RED

Exact command. `pipefail` retains the test's exit while `sed` removes only incidental live-sidecar
INFO lines from the displayed evidence:

```bash
set -o pipefail
bash tests/rem-runner-routing-contract.sh baseline 2>&1 | sed -n -e '/^Section5 mode=/p' -e '/^future-tool-build /p' -e '/^progress /p' -e '/^Section5 boundary_reached /p' -e '/^Section5 hashes /p' -e '/^PRODUCT RED:/p' -e '/^Section5 PASS /p' -e '/^Section5 cleaned /p'
```

Exit: `1`.

Exact filtered output:

```text
Section5 mode=baseline expected_observations=24 estimated_rate_min=0.5_obs/s estimated_wall_max=90s hard_timeout=120s
future-tool-build mutation_sha256=98cc2637b5fbe44946e31708599f38d3ecaf4160d37b84a887b91e2bf5caf0ad binary_sha256=6ca648549b8c26f4420a9be9a3063cf03b37d088078c60e8155ecacfcf7d656d
progress 1/24 case=section-first-code-0 exit=0 rate=0.17_obs/s eta=132.9s
progress 2/24 case=section-first-code-1 exit=1 rate=0.34_obs/s eta=64.0s
progress 3/24 case=section-first-code-2 exit=2 rate=0.51_obs/s eta=41.0s
progress 4/24 case=section-first-code-3 exit=3 rate=0.68_obs/s eta=29.5s
progress 5/24 case=section-first-code-4 exit=4 rate=0.84_obs/s eta=22.6s
progress 6/24 case=section-first-code-5 exit=5 rate=0.98_obs/s eta=18.3s
progress 7/24 case=section-first-code-6 exit=6 rate=1.14_obs/s eta=14.9s
progress 8/24 case=section-first-code-7 exit=7 rate=1.30_obs/s eta=12.3s
progress 9/24 case=section-first-code-8 exit=8 rate=1.45_obs/s eta=10.3s
progress 10/24 case=section-first-code-9 exit=9 rate=1.60_obs/s eta=8.7s
progress 11/24 case=section-first-code-10 exit=10 rate=1.75_obs/s eta=7.4s
progress 12/24 case=sibling-first-code-0 exit=0 rate=1.87_obs/s eta=6.4s
progress 13/24 case=sibling-first-code-1 exit=1 rate=2.01_obs/s eta=5.5s
progress 14/24 case=sibling-first-code-2 exit=2 rate=2.16_obs/s eta=4.6s
progress 15/24 case=sibling-first-code-3 exit=3 rate=2.30_obs/s eta=3.9s
progress 16/24 case=sibling-first-code-4 exit=4 rate=2.44_obs/s eta=3.3s
progress 17/24 case=sibling-first-code-5 exit=5 rate=2.53_obs/s eta=2.8s
progress 18/24 case=sibling-first-code-6 exit=6 rate=2.66_obs/s eta=2.3s
progress 19/24 case=sibling-first-code-7 exit=7 rate=2.80_obs/s eta=1.8s
progress 20/24 case=sibling-first-code-8 exit=8 rate=2.93_obs/s eta=1.4s
progress 21/24 case=sibling-first-code-9 exit=9 rate=3.06_obs/s eta=1.0s
progress 22/24 case=sibling-first-code-10 exit=10 rate=3.19_obs/s eta=0.6s
progress 23/24 case=qcg16-usage exit=2 rate=3.33_obs/s eta=0.3s
progress 24/24 case=qcg16-rejected exit=2 rate=3.46_obs/s eta=0.0s
Section5 boundary_reached observations=24/24 forwarded_requests=30 sidecar_pid=79335 store=/mnt/ramdisk/tmp/mem-claw-test-lyhmZ0/test.sqlite
Section5 hashes plan=2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69 fixture=9d1e734f43fadb5a65f477e79237fb122d94b94efe3e4a94b7219fdcf04a8bde current_cli=e672158b54a190b9e71c4acb32f046c6d573e12b24b74efa31abb4b508b2f7f8 future_cli=6ca648549b8c26f4420a9be9a3063cf03b37d088078c60e8155ecacfcf7d656d run_rem=ae77cbeb852f23ae87f35cca8128d57fa3ece8456062fc6589f04493499cf084 run_rem_noop=e56ba30ccd0ef3488ad759febe0a069d365c1234c4dd3ad3b1ba416c2ab050d8
PRODUCT RED: QCG-12 section-first code=10: future code 10 was not logged as unmatched; output="{\"job_id\":\"rem-wave-c2db7b0f-cf38-43e2-ac0f-ba02cf64bbee\",\"correlation_id\":\"section5-section-first-10-80fda6bf-9d10-4b93-81da-014a4736d175\"}\n\n"
PRODUCT RED: QCG-12 sibling-first code=10: future code 10 was not logged as unmatched; output="{\"job_id\":\"rem-wave-5b35005d-52ac-4fc0-aad6-f825f28f5ddb\",\"correlation_id\":\"section5-sibling-first-10-8b503047-7f74-4e8e-89ee-1b69b8e10cd6\"}\n\n"
PRODUCT RED: QCG-16 usage exit=2, expected=20
PRODUCT RED: QCG-16 rejected-operation exit=2, expected=21
PRODUCT RED: QCG-16 provenance: runner originates tool-range literal: exit 2
PRODUCT RED: QCG-12 routing table: run_rem.sh has no one table enumerating 0..9 plus unknown
Section5 cleaned runtime_root=/tmp/sno-section5-runner-contract.81JBCG status=1
```

## Gate interpretation

| Gate | Baseline result |
|---|---|
| QCG-12 | Genuine RED: both runners propagate known `0..9`, but neither logs future `10` as unmatched; the enumerated table is absent. |
| QCG-13 | Covered and met on both exit-5 status paths: raw states and the version-mismatch sentence survived; no invalid-response classification. |
| QCG-15 | Covered and met: distinct unrelated state prose produced the same exit-5 fate, and every start trace retained `--json`. |
| QCG-16 | Genuine RED: usage and rejected operation are both `2`, not `20/21`; the runner therefore still originates a tool-range literal. |
| QCG-17 | Covered and met: both absolute runner paths completed their success case through one sidecar PID and one real store, with old/new operation identifiers observed separately. |

The RED is caused by missing product behavior, not a test, build, fixture, environment, symbol,
timeout, throughput, or cleanup failure.

## Final test-owned checks

Commands:

```bash
bash -n tests/rem-runner-routing-contract.sh
shellcheck tests/rem-runner-routing-contract.sh
git diff --check -- tests/rem-runner-routing-contract.sh tests/rem-runner-routing-contract.mts tests/fixtures/rem-runner-routing-cases.json
/home/lh/code/sno-station-core-edge-rem-wave/node_modules/.bin/tsc --project /tmp/section5-rem-runner-tsconfig.json
```

All four final commands exited `0` with empty output. The temporary TypeScript configuration
inherited the real Mem Claw app configuration, set `rootDir` to `/`, disabled emit/declarations,
and named only `/home/lh/code/sno-cli/tests/rem-runner-routing-contract.mts`; it was removed after
the check.
