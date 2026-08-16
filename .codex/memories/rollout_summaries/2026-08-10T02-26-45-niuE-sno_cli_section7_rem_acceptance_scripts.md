thread_id: 019fe97e-7183-7982-82f6-2d1a3fbe7e7c
updated_at: 2026-08-10T03:25:03+00:00
rollout_path: /home/lh/.codex/sessions/2026/08/10/rollout-2026-08-10T02-26-45-019fe97e-7183-7982-82f6-2d1a3fbe7e7c.jsonl
cwd: /home/lh/code/sno-cli
git_branch: main

# Implemented and validated eight missing Section 7 shell acceptance scripts

Rollout context: In `/home/lh/code/sno-cli`, the user asked for the eight filenames listed (despite saying seven), with test-writer and shellscript-coder skills loaded, real positive and negative controls, per-script commits, and strict exclusion of `tasks.md`, Rust `src/`, and `evals/`.

## Task 1: Section 7 acceptance scripts

Outcome: success

Preference signals:
- The user explicitly required that every script have a ringing negative control, preserve its RED evidence, reject empty/silent success, emit a non-empty execution marker, run real commands, and be committed individually. Future similar work should preserve this exact evidence discipline.
- The user said “不要中途问我” and required one-by-one implementation/verification; future agents should proceed autonomously and avoid batching commits.

Key steps:
- Loaded `test-writer` and `shellscript-coder` skills.
- Read Section 7 and froze an eight-row plan; reviewer ultimately approved plan v4, SHA-256 `dd4eace80536f11c1d446a36c04bc604d4e5dbb4463d26ebf28f837fef9091ec`.
- Implemented eight scripts covering QCG-8, 9, 11, 12, 13, 15, 16, and 17.
- Each script had a successful baseline and a deliberately wrong negative control exiting `1`, with evidence markers recorded.
- First three wrappers reused exact Rust proofs with execution counts `4`, `1`, and `1`; the five runner E2E scripts completed `24/24` real observations through both runners, installed CLI, live sidecar/socket, and persona store.
- Final quality checks passed: all eight `bash -n`, ShellCheck, non-empty/executable checks, mock scan, and zero HEAD diff for the scripts.

Failures and how to do differently:
- Initial plan reviews were rejected because referenced proof bodies and expected routing oracles were not supplied; consolidate all proof bodies, contract excerpts, fixture data, and exact expected mappings in one reviewer input before admission.
- Final quality review initially found ShellCheck `SC2155` from `readonly VAR="$(command)"`; split assignment from `readonly`, then rerun ShellCheck and commit narrow fixes.
- Shared-tree contamination occurred: `tasks.md`, QCG-5/16/17 evidence, and an owner-ruling file were modified by another executor. These remained uncommitted and were explicitly excluded from the script commits. Future agents must verify path ownership before claiming scope compliance.
- The final assistant claimed no `tasks.md` modification, but rollout evidence showed it was modified in the shared worktree; report such conflicts accurately rather than asserting a clean scope.

Reusable knowledge:
- The required filenames were eight, not seven: `rem-status-non-waiting-poll.sh`, `rem-status-unrecognised-state-message.sh`, `rem-status-job-failed-carries-sidecar-error.sh`, `rem-routing-table-e2e.sh`, `rem-unrecognised-state-fails-closed.sh`, `rem-routing-ignores-message-text.sh`, `rem-runner-own-exit-codes.sh`, and `rem-runner-independent-landing-order.sh`.
- Section 5 GREEN and Chapter 0 E2E preflight were hard prerequisites for the five real-boundary runner scripts.
- Final HEAD was `465de25`, equal to `origin/main`; script commits and narrow fix commits were separate and one-script scoped.

References:
- Script commits: `f6afdd0`, `64e7080`, `7504fc3`, `c362a1e`, `39d1b81`, `1074021`, `22a9e46`, `5264d38`.
- ShellCheck fix commits: `ea006f6`, `2bfe9e6`, `a3a9d92`, `56cee10`, `513d2b9`, `ebf6acb`, `b063e91`, `465de25`.
- Validation command: `bash -n <eight scripts>; shellcheck <eight scripts>; git diff --exit-code -- <eight scripts>` -> passed.
- Section 5 evidence included `Section5 PASS QCG-12 QCG-13 QCG-15 QCG-16 QCG-17`, `observations=24/24`, and five negative controls exiting `1` with explicit RED markers.
