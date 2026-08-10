#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_repo="/home/lh/code/sno-station-core-edge-rem-wave"
tsx="$sidecar_repo/node_modules/.bin/tsx"
tsx_config="$sidecar_repo/apps/mem-claw/tsconfig.json"
helper="$repo_root/tests/rem-trace-records-state-class-and-exit-code.mts"
plan="$repo_root/tests/rem-trace-records-state-class-and-exit-code.plan.md"
receipt="$repo_root/tests/rem-trace-records-state-class-and-exit-code.plan.sha256"
admitted_plan_sha256="27536c728d7733338ac557531f1ec7b58f8f02db180cc835dd313655a2092a48"

test -x "$repo_root/target/debug/sno"
test -x "$tsx"
test -f "$helper"
test -f "$plan"
test -f "$receipt"

actual_plan_sha256="$(sha256sum "$plan" | awk '{print $1}')"
receipt_plan_sha256="$(awk 'NR == 1 {print $1}' "$receipt")"
if [[ "$actual_plan_sha256" != "$admitted_plan_sha256" || "$receipt_plan_sha256" != "$admitted_plan_sha256" ]]; then
	echo "reviewed plan hash mismatch: actual=$actual_plan_sha256 receipt=$receipt_plan_sha256 admitted=$admitted_plan_sha256" >&2
	exit 1
fi

run_root="$(mktemp -d /tmp/sno-qcg14-trace-tuple.XXXXXX)"
case "$run_root" in
	/tmp/sno-qcg14-trace-tuple.*) ;;
	*)
		echo "refusing unexpected temporary root: $run_root" >&2
		exit 1
		;;
esac

cleanup() {
	rm -rf -- "$run_root"
}
trap cleanup EXIT HUP INT TERM

export QCG14_RUN_ROOT="$run_root"
echo "QCG-14 expected_observations=1 estimated_rate=1_obs/s estimated_wall=1s hard_timeout=20s"

set +e
timeout --signal=TERM --kill-after=5s 20s \
	"$tsx" --tsconfig "$tsx_config" "$helper"
status=$?
set -e

cleanup
trap - EXIT HUP INT TERM
if [[ -e "$run_root" ]]; then
	echo "temporary root cleanup failed: $run_root" >&2
	exit 1
fi
echo "QCG-14 cleaned runtime_root=$run_root"
exit "$status"
