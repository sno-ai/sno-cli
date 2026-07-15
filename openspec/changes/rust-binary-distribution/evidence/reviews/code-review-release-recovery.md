# Codex Adversarial Review

Target: `.github/workflows/release-preflight.yml`, `.github/workflows/release.yml`, `scripts/check-release-workflow.sh`, `scripts/test-release-workflow-policy.sh`
Verdict: needs-attention

A tag can move after its commit receipt is checked but before the draft release is created, permanently associating verified artifacts with the wrong release commit.

Findings:
- [high] Tag movement can decouple published artifacts from the release commit (`.github/workflows/release.yml:313-323`, confidence 0.88)
  Trigger: A release starts for tag `vX` at approved commit A; before the host job creates the release, an operator or automation force-updates `vX` to commit B. The workflow still uploads artifacts built from A, but creates the release by the current tag name without re-resolving it.
  Impact: Users receive artifacts built from A under a release tag resolving to B. The receipt did not authorize B, and immutable publication makes the mismatch difficult or impossible to correct.
  Recommendation: Immediately before creating the draft release, resolve the remote tag target and fail unless it equals the event commit; repeat the check before publication.

Next steps:
- Add a release-policy fixture covering a tag move between preflight and release creation.
