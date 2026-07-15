# Codex Adversarial Review

Target: `.github/workflows/release-preflight.yml`, `.github/workflows/release.yml`, `scripts/check-release-workflow.sh`, `scripts/test-release-workflow-policy.sh`
Verdict: needs-attention

The release path has a self-contradiction: it requires a disposable public candidate to be immutable, then blocks final publication on deleting that candidate. The policy checker also permits removal of the administrator authorization gate while reporting success.

Findings:
- [critical] Immutable candidate cannot satisfy required cleanup (`.github/workflows/release.yml`:360-371, confidence 0.84)
  Trigger: A release reaches candidate hosting; the workflow creates a public prerelease, asserts its `immutable` field is true, then the required cleanup job deletes that release and its tag before publication can run.
  Impact: On repositories enforcing immutable releases, cleanup cannot remove the candidate, so the final release is never published; the supposedly one-use public candidate may also remain exposed.
  Recommendation: Run anonymous-install verification from a dedicated public location where candidate artifacts can be deleted, and do not make final publication depend on deleting a release verified as immutable.

- [high] Policy checker does not verify that authorization preflight is reachable (`scripts/check-release-workflow.sh`:51-62, confidence 0.94)
  Trigger: A workflow edit removes `custom-release-preflight` and its dependency from the artifact build; the standalone preflight file remains unchanged, so this checker still passes.
  Impact: The administrator-bound receipt and tag/version checks disappear from the actual release path; anyone able to push a qualifying tag can publish an official release.
  Recommendation: Parse the workflow and require the reusable preflight job plus its dependency in the release path; add a mutation test that removes that job or dependency and expects failure.

Next steps:
- Redesign the disposable public-candidate flow before enabling immutable releases.
- Close the checker bypass and add the missing mutation test.
