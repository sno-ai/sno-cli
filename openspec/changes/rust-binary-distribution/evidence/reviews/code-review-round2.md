# Codex Adversarial Review — Round 2

Verdict: needs-attention

## Authoritative Findings

### High — Broken public installer can be published irreversibly

- Trigger: Local `file://` smoke passes but GitHub download availability or redirects fail after publication.
- Impact: The primary installation path fails on an immutable release.
- Recommendation: Validate candidate assets through GitHub before final immutable publication.

### High — macOS bootstrap invokes an unavailable checksum utility

- Trigger: Standard macOS reaches `sha256sum --check`.
- Impact: macOS release bootstrap stops before installing `cargo-dist`.
- Recommendation: Use a Darwin-compatible SHA-256 command.

### High — Job-level token escalation bypasses the least-privilege check

- Trigger: A job adds write scope while the root remains read-only.
- Impact: The checker passes while release code receives repository-writing credentials.
- Recommendation: Validate every job-level permission block against a minimal allowlist.

### High — Mutable GitHub Action references pass validation

- Trigger: An action uses a mutable ref not covered by the original short denylist.
- Impact: Changed upstream code executes while the security check passes.
- Recommendation: Require a full immutable commit SHA for every remote action.

### High — Installer and smoke gates can be satisfied by unrelated YAML text

- Trigger: Required strings appear in unrelated or disabled steps.
- Impact: Unverified code can run while the gate reports success.
- Recommendation: Verify the actual named jobs, steps, and dependencies.

### High — Excluding a workflow disables prohibited-publisher detection

- Trigger: A workflow is excluded with a reason and contains an unmanaged publisher.
- Impact: The policy reports success while the workflow can publish an npm or Python package.
- Recommendation: Always scan executable workflow candidates regardless of inventory disposition.

All findings are resolved in `code-review-resolution.md`; none was dismissed.
