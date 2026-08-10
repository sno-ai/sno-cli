#!/usr/bin/env bash
set -Eeuo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ACCEPTANCE_ORACLE="${REM_ACCEPTANCE_ORACLE:-/bin/true}"
proof_execution_count=0

if [[ ! -x "$ACCEPTANCE_ORACLE" ]]; then
	printf 'REM_ACCEPTANCE_ORACLE is not executable: %s\n' "$ACCEPTANCE_ORACLE" >&2
	exit 1
fi

(
	cd "$REPO_ROOT"
	cargo test --test rem_job_state_contract \
		qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture -- \
		--exact --nocapture --test-threads=1
)
proof_execution_count=$((proof_execution_count + 1))
printf 'proof_marker=%s\n' \
	qcg_9_unfamiliar_state_precedes_error_and_survives_shell_capture
printf 'proof_execution_count=%d\n' "$proof_execution_count"
"$ACCEPTANCE_ORACLE"
