#!/usr/bin/env bash
set -Eeuo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly BASE_RUNNER="$REPO_ROOT/tests/rem-runner-routing-contract.sh"
readonly GREEN_ARTIFACT="$REPO_ROOT/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/section-5-independent-green.md"
readonly GREEN_RECEIPT="$REPO_ROOT/openspec/changes/rem-job-state-contract/evidence/80-rem-job-state-contract/section-5-independent-green.sha256"
readonly REVIEWED_PLAN_SHA256="dd4eace80536f11c1d446a36c04bc604d4e5dbb4463d26ebf28f837fef9091ec"
readonly EXPECTED_GREEN_SHA256="db25eac132ad648b9cbe3e653b08bf68df9c20857393a3738161d6656a3f3e66"

verify_green_receipt() {
	git -C "$REPO_ROOT" ls-files --error-unmatch \
		"${GREEN_ARTIFACT#"$REPO_ROOT/"}" \
		"${GREEN_RECEIPT#"$REPO_ROOT/"}" >/dev/null
	git -C "$REPO_ROOT" diff --quiet -- \
		"${GREEN_ARTIFACT#"$REPO_ROOT/"}" \
		"${GREEN_RECEIPT#"$REPO_ROOT/"}"
	local actual receipt
	actual="$(sha256sum "$GREEN_ARTIFACT" | awk '{print $1}')"
	receipt="$(awk 'NR == 1 {print $1}' "$GREEN_RECEIPT")"
	if [[ "$actual" != "$EXPECTED_GREEN_SHA256" || "$receipt" != "$EXPECTED_GREEN_SHA256" ]]; then
		printf 'Section 5 GREEN hash mismatch: actual=%s receipt=%s expected=%s\n' \
			"$actual" "$receipt" "$EXPECTED_GREEN_SHA256" >&2
		exit 1
	fi
}

run_negative_control() { bash "$BASE_RUNNER" negative-qcg15; }

case "${1:-baseline}" in
	negative-control)
		verify_green_receipt
		run_negative_control
		exit $?
		;;
	baseline) ;;
	*) printf 'usage: %s [baseline|negative-control]\n' "$0" >&2; exit 2 ;;
esac

verify_green_receipt
output_file="$(mktemp /tmp/rem-qcg15-baseline.XXXXXX)"
negative_file="$(mktemp /tmp/rem-qcg15-negative.XXXXXX)"
cleanup() { rm -f -- "$output_file" "$negative_file"; }
trap cleanup EXIT HUP INT TERM

set +e
bash "$BASE_RUNNER" baseline 2>&1 | tee "$output_file"
baseline_status="${PIPESTATUS[0]}"
set -e
if [[ "$baseline_status" -ne 0 ]]; then exit "$baseline_status"; fi

test "$(rg -c '^progress [0-9]+/24 ' "$output_file")" -eq 24
rg '^Section5 boundary_reached observations=24/24 ' "$output_file" >/dev/null
rg -Fx 'Section5 PASS QCG-12 QCG-13 QCG-15 QCG-16 QCG-17' "$output_file" >/dev/null
line20="$(rg '^progress 20/24 ' "$output_file")"
rate="$(printf '%s\n' "$line20" | sed -E 's/.* rate=([0-9.]+)_obs\/s.*/\1/')"
eta="$(printf '%s\n' "$line20" | sed -E 's/.* eta=([0-9.]+)s.*/\1/')"
awk -v rate="$rate" -v eta="$eta" 'BEGIN { exit !(rate >= 0.5 && eta <= 20) }'

set +e
run_negative_control 2>&1 | tee "$negative_file"
negative_status="${PIPESTATUS[0]}"
set -e
if [[ "$negative_status" -ne 1 ]]; then
	printf 'negative-qcg15 returned %s, expected 1\n' "$negative_status" >&2
	exit 1
fi
rg '^NEGATIVE CONTROL RED QCG15:' "$negative_file" >/dev/null

printf 'reviewed_plan_sha256=%s\n' "$REVIEWED_PLAN_SHA256"
printf 'proof_execution_count=24\n'
printf 'proof_marker=qcg15_message_text_independent_routing\n'
printf 'negative_control_red_observed=1\n'
