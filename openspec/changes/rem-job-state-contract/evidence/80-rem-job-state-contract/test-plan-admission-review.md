# Owner final admission ruling

Verdict: proceed with Section 2 only.

The owner final ruling supersedes the rejected premise and material-blocker disposition from all
three prior Codex Reviewer passes. No fourth admission review is authorized or required.

The owner rejected all three proposed alternatives and fixed the released-PRD interpretation:

- REQ-1: each declaration entry holds class name, machine-readable error code membership, and
  process exit; both codes are read from that entry and are not authored at raise sites.
- REQ-5: exactly ten named classes use exits 0 through 9 as published.
- REQ-11, REQ-13, and REQ-14 fix the named behavior for `rem_state_unrecognised`,
  `sidecar_response_invalid`, and `rem_job_failed`.
- REQ-7 and REQ-8 reserve exit `1` for absent mappings only; no named class owns it.
- QCG-1, QCG-2, QCG-6, and QCG-7 are admitted before a product declaration skeleton exists. Their
  current RED is the missing product contract, not a test or environment defect.

Authorized test scope: one independent Rust integration target covering only QCG-1, QCG-2, QCG-6,
and QCG-7. Product source, README, Cargo files, runner files, and `tasks.md` remain forbidden.

Reviewed/admitted plan SHA-256 is recorded in `test-plan.sha256` after this ruling is applied.
