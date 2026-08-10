#!/usr/bin/env bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly REPO_ROOT
readonly ACCEPTANCE_ORACLE="${REM_ACCEPTANCE_ORACLE:-/bin/true}"
proof_execution_count=0

run_proof() {
	local marker="$1"
	shift
	(
		cd "$REPO_ROOT"
		"$@"
	)
	proof_execution_count=$((proof_execution_count + 1))
	printf 'proof_marker=%s\n' "$marker"
}

if [[ ! -x "$ACCEPTANCE_ORACLE" ]]; then
	printf 'REM_ACCEPTANCE_ORACLE is not executable: %s\n' "$ACCEPTANCE_ORACLE" >&2
	exit 1
fi

run_proof qcg_8_waiting_and_nonwaiting_known_states_succeed \
	cargo test --test rem_job_state_contract \
	qcg_8_waiting_and_nonwaiting_known_states_succeed -- \
	--exact --nocapture --test-threads=1
run_proof rem_one_shot_status_reads_running_then_stable_done \
	cargo test --test cli \
	rem_one_shot_status_reads_running_then_stable_done -- \
	--exact --nocapture --test-threads=1
run_proof qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture \
	cargo test --test rem_job_state_contract \
	qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture -- \
	--exact --nocapture --test-threads=1
run_proof qcg_11_failed_job_preserves_only_the_supplied_sidecar_sentinel \
	cargo test --test rem_job_state_contract \
	qcg_11_failed_job_preserves_only_the_supplied_sidecar_sentinel -- \
	--exact --nocapture --test-threads=1

printf 'proof_execution_count=%d\n' "$proof_execution_count"
"$ACCEPTANCE_ORACLE"
