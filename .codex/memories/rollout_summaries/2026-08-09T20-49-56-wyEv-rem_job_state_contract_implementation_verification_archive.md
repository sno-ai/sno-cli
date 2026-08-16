thread_id: 019fe84a-130c-7c51-af95-f31dd2f57e47
updated_at: 2026-08-10T03:35:05+00:00
rollout_path: /home/lh/.codex/sessions/2026/08/09/rollout-2026-08-09T20-49-56-019fe84a-130c-7c51-af95-f31dd2f57e47.jsonl
cwd: /home/lh/code/sno-cli
git_branch: main

# REM job-state contract implemented, verified, and archived

Rollout context: In `/home/lh/code/sno-cli`, the user requested implementation of OpenSpec change `rem-job-state-contract`, prioritizing the first sections initially, then explicitly approved a closeout without creating or rerunning redundant tests.

## Task 1: Implement and verify REM job-state contract

Outcome: success

Preference signals:
- The user objected to jargon and asked for plain-language explanation of value: “什么五个 runner 入口，你说人话行吗？” Future reports should explain product value directly and distinguish real verification from bookkeeping.
- The user explicitly approved stopping five wrapper scripts that all invoked the same underlying 24-observation test: redundant wrappers should not be rerun merely to satisfy separate ledger rows when the underlying proof already exists.
- The user required evidence content audits, not file-existence checks: inspect recorded command output anchors and use existing output rather than inventing or rerunning tests.
- The user requested direct execution without intermediate confirmation and asked not to report progress unless blocked.

Key steps:
- Read the OpenSpec artifacts, PRD, review findings, repository instructions, and Rust/test-writing skills.
- Identified and resolved the contract discrepancy between the change’s earlier QCG-18/noop framing and the released PRD’s QCG-1 through QCG-17 ledger; final artifacts use 21 requirements and 17 acceptance rows.
- Implemented a single Rust REM outcome declaration (`src/rem_outcome.rs`) with stable exit codes 0–9 and complete error-code mapping; updated error resolution and REM behavior.
- Added distinct handling for unfamiliar non-empty states (`rem_state_unrecognised`, exit 5), malformed/empty responses (exit 6), sidecar failures (exit 7), local failures (exit 8), and unknown jobs (exit 9).
- Updated README exit-code documentation and both Memora runners’ numeric routing, fail-closed unknown-code behavior, and runner-owned exits 20/21.
- Added trace outcome data and independent Rust/shell evidence.
- Audited all 17 QCG evidence files for non-empty recorded command output and anchors; the task-evidence gate passed before commit.
- Archived with OpenSpec to `openspec/changes/archive/2026-08-10-rem-job-state-contract/` and published `openspec/specs/rem-job-state-contract/spec.md`.

Failures and how to do differently:
- The initial evidence gate rejected QCG-5 because its anchor was prose rather than a recorded command output region; adding a proper command/status/output block fixed it.
- The final evidence gate must be run before archive with the active change ID. After archive, `task-evidence-gate --change rem-job-state-contract` correctly returns “target change does not exist”; this is expected because the change is archived. A full-tree gate then inspected an unrelated legacy change and failed due to missing evidence clauses, so do not interpret that as a failure of the archived change.
- A remote-tag query initially hit DNS/network issues, then succeeded with elevated networking; it confirmed only v0.1.7 was released and REM work was not included in that release.

Reusable knowledge:
- Repository uses Rust edition 2024, MSRV 1.85, and CI-style checks: `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and targeted Cargo tests.
- Final contract mapping: 0 success; 1 unclassified fallback; 2 usage; 3 job failed; 4 timeout; 5 state vocabulary mismatch; 6 malformed/truncated response; 7 sidecar failure; 8 local environment/trace failure; 9 unknown job.
- Runner-owned exits are 20 for usage and 21 for rejected operation; tool codes 0–9 are only propagated from the CLI. Routing is numeric, not message-text based, and unknown codes fail closed.
- Verified evidence included Rust contract tests 14/14, CLI regression 1/1, QCG-5 live-sidecar-backed matrix 80/80 with no interchange, QCG-14 trace proof, and Section 5 dual-runner real-boundary proof with 24/24 observations.
- OpenSpec strict validation passed before and after archive; post-archive validation reported 2 items passed, 0 failed.

References:
- `/home/lh/code/sno-cli/src/rem_outcome.rs`
- `/home/lh/code/sno-cli/src/error.rs`, `src/rem.rs`, `src/cli.rs`, `src/service.rs`
- `openspec/changes/archive/2026-08-10-rem-job-state-contract/evidence/section-8-final-verification.md`
- `openspec/specs/rem-job-state-contract/spec.md`
- Commits: `550301a Verify the complete REM job state contract`; `187d004 Archive the REM job state contract`
- Final status: local `main` clean and ahead of `origin/main` by 2 commits; not pushed.
