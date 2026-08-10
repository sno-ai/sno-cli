#!/usr/bin/env bash
set -euo pipefail

mode="${1:-full}"
case "$mode" in
	smoke | full) ;;
	*)
		echo "usage: $0 [smoke|full]" >&2
		exit 2
		;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_repo="/home/lh/code/sno-station-core-edge-rem-wave"
tsx="$sidecar_repo/node_modules/.bin/tsx"
tsx_config="$sidecar_repo/apps/mem-claw/tsconfig.json"
runner="$repo_root/tests/rem-status-exit-code-matrix.mts"

test -x "$repo_root/target/debug/sno"
test -x "$tsx"
test -f "$sidecar_repo/apps/mem-claw/src/sidecar/main.ts"

run_root="$(mktemp -d /tmp/sno-qcg5-live-sidecar.XXXXXX)"
case "$run_root" in
	/tmp/sno-qcg5-live-sidecar.*) ;;
	*)
		echo "refusing unexpected temporary root: $run_root" >&2
		exit 1
		;;
esac

cleanup() {
	rm -rf -- "$run_root"
}
trap cleanup EXIT HUP INT TERM

mkdir -p "$run_root/tmp"
export TMPDIR="$run_root/tmp"
export QCG5_RUN_ROOT="$run_root"

echo "QCG-5 mode=$mode runtime_root=$run_root"
echo "QCG-5 expected_observations=$([[ "$mode" == "full" ]] && echo 80 || echo 10) estimated_rate=1.45_obs/s estimated_wall=55s hard_timeout=110s"

timeout --signal=TERM --kill-after=5s 110s "$tsx" --tsconfig "$tsx_config" "$runner" "$mode"

cleanup
trap - EXIT HUP INT TERM
if [[ -e "$run_root" ]]; then
	echo "temporary root cleanup failed: $run_root" >&2
	exit 1
fi
echo "QCG-5 cleaned runtime_root=$run_root"
