# Codex Adversarial Review

Target: `.github/workflows/release-preflight.yml`, `.github/workflows/release.yml`, `scripts/check-release-workflow.sh`, `scripts/test-release-workflow-policy.sh`
Verdict: needs-attention

The workflow publishes the final release before its final public install check, and it can delete a successfully published release when post-publication verification is merely ambiguous.

Findings:
- [high] Final public installer failure has no containment path (`.github/workflows/release.yml:409-504`, confidence 0.92)
  Trigger: Draft and candidate checks pass, the final draft is promoted and confirmed immutable, then an anonymous install from the final tag fails. The subsequent public installer smoke reports that failure after publication.
  Impact: Users receive a known-bad immutable release; no job consumes the smoke failure or removes the release.
  Recommendation: Add failure cleanup for the final public smoke, or make the candidate smoke exercise the exact final installer resolution so it is a true promotion gate.

- [high] A transient verification failure can delete a successfully published release (`.github/workflows/release.yml`:421-462, confidence 0.91)
  Trigger: `gh release edit --draft=false` succeeds, but the following immutable-status API request fails or returns stale state. The publish job fails and the cleanup job deletes the release.
  Impact: A valid public release can disappear, causing user download failures and requiring manual reconstruction.
  Recommendation: Do not delete after an ambiguous post-publish verification failure; retain the release and require a confirmed draft state or explicit operator action before cleanup.

Next steps:
- Make post-publication validation and cleanup distinguish a failed promotion from an unavailable verification result.
