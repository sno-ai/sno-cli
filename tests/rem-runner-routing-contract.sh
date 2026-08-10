#!/usr/bin/env bash
set -euo pipefail

mode="${1:-baseline}"
case "$mode" in
    baseline|negative-qcg12|negative-qcg13|negative-qcg15|negative-qcg16|negative-qcg17) ;;
    *)
        echo "usage: $0 baseline|negative-qcg12|negative-qcg13|negative-qcg15|negative-qcg16|negative-qcg17" >&2
        exit 2
        ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_repo="/home/lh/code/sno-station-core-edge-rem-wave"
tsx="$sidecar_repo/node_modules/.bin/tsx"
tsx_config="$sidecar_repo/apps/mem-claw/tsconfig.json"
helper="$repo_root/tests/rem-runner-routing-contract.mts"
fixture="$repo_root/tests/fixtures/rem-runner-routing-cases.json"
plan="$repo_root/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.md"
receipt="$repo_root/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5.sha256"
owner_ruling="$repo_root/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/test-plan-section-5-owner-ruling.md"
admitted_plan_sha256="2b8aab7093536b3ee54a08b18db35b9ca9a6cc9938f3d8d93e65cfadaa3bbd69"

for required in "$tsx" "$helper" "$fixture" "$plan" "$receipt" "$owner_ruling" "$repo_root/target/debug/sno"; do
    test -e "$required"
done

actual_plan_sha256="$(sha256sum "$plan" | awk '{print $1}')"
receipt_plan_sha256="$(awk 'NR == 1 {print $1}' "$receipt")"
if [[ "$actual_plan_sha256" != "$admitted_plan_sha256" || "$receipt_plan_sha256" != "$admitted_plan_sha256" ]]; then
    echo "frozen plan hash mismatch: actual=$actual_plan_sha256 receipt=$receipt_plan_sha256 expected=$admitted_plan_sha256" >&2
    exit 1
fi

run_root="$(mktemp -d /tmp/sno-section5-runner-contract.XXXXXX)"
case "$run_root" in
    /tmp/sno-section5-runner-contract.*) ;;
    *)
        echo "refusing unexpected temporary root: $run_root" >&2
        exit 1
        ;;
esac

cleanup() {
    rm -rf -- "$run_root"
}
trap cleanup EXIT HUP INT TERM

export SECTION5_RUN_ROOT="$run_root"
if [[ "$mode" == "baseline" ]]; then
    echo "Section5 mode=baseline expected_observations=24 estimated_rate_min=0.5_obs/s estimated_wall_max=90s hard_timeout=120s"
else
    echo "Section5 mode=$mode expected_observations=1 estimated_wall_max=5s hard_timeout=120s"
fi

set +e
timeout --signal=TERM --kill-after=5s 120s \
    "$tsx" --tsconfig "$tsx_config" "$helper" "$mode"
status=$?
set -e

cleanup
trap - EXIT HUP INT TERM
if [[ -e "$run_root" ]]; then
    echo "temporary root cleanup failed: $run_root" >&2
    exit 1
fi
echo "Section5 cleaned runtime_root=$run_root status=$status"
exit "$status"
