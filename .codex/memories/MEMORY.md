# Repository Memory



Repository: `/home/lh/code/sno-cli`



This file contains repository-scoped history only. Do not copy it into user-level memory or another repository.



# Task Group: /home/lh/code/sno-cli REM status JSON forward-field preservation

scope: Keep `sno station rem-status --json` forward-compatible by retaining raw sidecar JSON while using a minimal typed control view.
applies_to: cwd=/home/lh/code/sno-cli on main; reuse_rule=use the real writable sno-cli workspace and current sidecar test environment; partial integration evidence must not be treated as full verification.

## Task 1: Identify the raw-response design but fail outside the writable workspace, outcome fail

### rollout_summary_files

- rollout_summaries/2026-08-14T20-15-34-YEYB-rem_status_verbatim_json_blocked_workspace.md (cwd=/home/lh/code/sno-station-core, rollout_path=/home/lh/.codex/sessions/2026/08/14/rollout-2026-08-14T20-15-34-01a001ea-67f2-70c3-af81-b77249150e8d.jsonl, updated_at=2026-08-14T20:17:55+00:00, thread_id=01a001ea-67f2-70c3-af81-b77249150e8d, design only; no sno-cli write access)

### keywords

- patch rejected: writing outside of the project, sno-cli, src/rem.rs, RemJob, serde_json::Value, fetch_status_at, run_status, by_operation

## Task 2: Preserve complete sidecar status records, outcome partial

### rollout_summary_files

- rollout_summaries/2026-08-14T20-19-01-LCM0-rem_status_verbatim_json_unknown_field_preservation.md (cwd=/home/lh/code/sno-cli, rollout_path=/home/lh/.codex/sessions/2026/08/14/rollout-2026-08-14T20-19-01-01a001ed-8f17-72b3-9ad9-3399553a37c0.jsonl, updated_at=2026-08-14T20:23:26+00:00, thread_id=01a001ed-8f17-72b3-9ad9-3399553a37c0, implementation and unit checks passed; loopback proof blocked)

### keywords

- RemStatus, raw: serde_json::Value, rem-status --json, requested_operations, unknown by_operation, cargo clippy, bind loopback service, Operation not permitted

## User preferences

- “Do NOT simply add `job_id` and `requested_operations` as fields” -> make field survival structural: parse raw JSON once, keep typed fields only for control flow, and print the raw value for `--json`. [Task 1][Task 2]
- “Do NOT commit”, “Do NOT create a branch or worktree”, and “Do NOT install the built binary anywhere” -> keep the minimal in-place change uncommitted and do not alter installed binaries. [Task 2]
- the user prohibited fabricated RED/GREEN evidence -> report loopback-blocked integration checks as unverified. [Task 2]

## Reusable knowledge

- `RemStatus { job: RemJob, raw: Value }` is the correct boundary: reduce `RemJob` to `state`/`error`; `fetch_status_at` parses to `Value` then deserializes it; polling uses `job`, while `run_status --json` prints `raw`. Non-JSON output remains `job_id: state`. [Task 2]
- The real integration test fixture asserts JSON equality including `job_id`, `requested_operations`, and unknown `by_operation`. `cargo fmt --all --check`, strict Clippy, `cargo build`, and `cargo test --lib rem::tests -- --nocapture` (2 passed) ran successfully. [Task 2]

## Failures and how to do differently

- A session rooted in sno-station-core cannot write sno-cli even if its commands use sno-cli -> start in `/home/lh/code/sno-cli` before patching. The exact blocker was `patch rejected: writing outside of the project; rejected by user approval settings`. [Task 1]
- Loopback integration tests fail before assertions with `bind loopback service ... Operation not permitted`; do not weaken tests or call them product failures. Missing external `tsx`/tsconfig/`run_rem.sh` also blocked the trace script; task-evidence-gate had unrelated 21 ledger failures. [Task 2]

# Task Group: /home/lh/code/sno-cli REM job-state contract and Section 7 acceptance scripts

scope: Rust REM exit/routing contract, evidence-bound OpenSpec archival, and shell acceptance scripts with real RED/GREEN controls.
applies_to: cwd=/home/lh/code/sno-cli on main; reuse_rule=re-check active/archive change names and shared-tree ownership; do not rerun redundant proof wrappers during closeout.

## Task 1: Implement, evidence-audit, and archive `rem-job-state-contract`, outcome success

### rollout_summary_files

- rollout_summaries/2026-08-09T20-49-56-wyEv-rem_job_state_contract_implementation_verification_archive.md (cwd=/home/lh/code/sno-cli, rollout_path=/home/lh/.codex/sessions/2026/08/09/rollout-2026-08-09T20-49-56-019fe84a-130c-7c51-af95-f31dd2f57e47.jsonl, updated_at=2026-08-10T03:35:05+00:00, thread_id=019fe84a-130c-7c51-af95-f31dd2f57e47, archived; local main ahead two commits)

### keywords

- rem-job-state-contract, rem_outcome.rs, exit-codes, QCG-5, QCG-14, task-evidence-gate, openspec archive, fail-closed

## Task 2: Implement eight Section 7 REM shell acceptance scripts, outcome success

### rollout_summary_files

- rollout_summaries/2026-08-10T02-26-45-niuE-sno_cli_section7_rem_acceptance_scripts.md (cwd=/home/lh/code/sno-cli, rollout_path=/home/lh/.codex/sessions/2026/08/10/rollout-2026-08-10T02-26-45-019fe97e-7183-7982-82f6-2d1a3fbe7e7c.jsonl, updated_at=2026-08-10T03:25:03+00:00, thread_id=019fe97e-7183-7982-82f6-2d1a3fbe7e7c, eight scripts verified and individually committed)

### keywords

- ShellCheck, SC2155, QCG-8, QCG-9, QCG-11, QCG-12, QCG-13, QCG-15, QCG-16, QCG-17, negative control, observations=24/24

## User preferences

- “什么五个 runner 入口，你说人话行吗？” -> explain product value plainly; evidence audits must inspect command/status/output anchors, not merely files. “不要再造或重跑任何测试” -> preserve valid proof and skip redundant wrappers. [Task 1]
- For acceptance scripts, every named script needs real commands, non-empty marker, RED negative control that exits 1, retained RED evidence, and its own commit. “不要中途问我” -> proceed one-by-one without batching. [Task 2]

## Reusable knowledge

- CLI exit codes: 0 success, 1 unclassified fallback, 2 usage, 3 job failed, 4 timeout, 5 vocabulary mismatch, 6 malformed/truncated response, 7 sidecar failure, 8 local environment/trace failure, 9 unknown job. Runners own 20 usage/21 rejected operation, route numerically, and fail closed for unknown codes. [Task 1]
- Before archive, `task-evidence-gate --mode audit --change rem-job-state-contract` must see actual anchored output; then use strict OpenSpec validation. After archive, a narrowed active-name gate returning exit 2 is expected. [Task 1]
- Section 5 GREEN and Chapter 0 preflight are prerequisites for the five real runner E2E scripts. Final evidence had `observations=24/24`; validate scripts with `bash -n`, ShellCheck, non-empty/executable/mock checks, and zero HEAD diff. [Task 2]

## Failures and how to do differently

- A prose `QCG-5 PASS` is not evidence; record command, `status=0`, and output anchor. Do not judge an archived change by a full-tree gate that fails on unrelated active changes. [Task 1]
- Reviewer inputs need proof bodies, routing oracles, fixture data, and exact mappings; fix ShellCheck SC2155 by splitting assignment from `readonly`. Shared-tree modifications outside owned paths must be reported, excluded, and never claimed absent. [Task 2]
