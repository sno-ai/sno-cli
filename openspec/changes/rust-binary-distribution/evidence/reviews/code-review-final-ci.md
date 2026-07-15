# Codex Adversarial Review

Target: `.github/workflows/ci.yml`, `policy/test-substitutes.json`, `scripts/check-test-substitutes.sh`, `scripts/check-release-surfaces.sh`, `tests/support/sno_service_server.rs`  
Verdict: needs-attention

The release-policy check can be bypassed by moving publishing logic into a local composite action or normally named helper script, so prohibited release paths can reach `main` without detection.

Findings:
- [medium] Release publishing logic can evade policy scanning (`scripts/check-release-surfaces.sh`:18-52, confidence 0.96)
  Trigger: A contributor adds `npm publish` or Python publishing to `.github/actions/publish/action.yml` or `scripts/publish.sh`, then invokes it from an existing workflow. The workflow itself contains only the invocation, while these paths are not release-surface candidates.
  Impact: The policy passes while an unintended non-Rust package release executes on a trusted `main` push. This can publish incorrect or unauthorized artifacts externally and requires a corrective release or unpublish.
  Recommendation: Scan all local GitHub action definitions and helper scripts for prohibited publishing commands, independent of filename; retain the manifest only for disposition and review ownership.

Next steps:
- Add a policy test proving that a local composite action and a non-`release`-named script containing a prohibited publisher both fail the check.