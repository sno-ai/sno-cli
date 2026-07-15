# Codex Adversarial Review

Target: `.github/workflows/release-preflight.yml`, `.github/workflows/release.yml`, `scripts/check-release-workflow.sh`, `scripts/test-release-workflow-policy.sh`
Verdict: needs-attention

The release path can publish a compromised release if tag creation is not externally restricted, and it makes releases irreversible before the only public-install test runs. Do not ship this as the sole release authorization boundary.

Findings:
- [critical] The authorization gate is defined by the tag it authorizes (`.github/workflows/release.yml`:40-90, confidence 0.91)
  Trigger: Assuming no external GitHub ruleset restricts release tags to protected commits, a writer creates a version tag pointing at a commit that removes or bypasses the local preflight call; the tag-triggered workflow then runs that altered workflow and reaches its write-capable publishing job.
  Impact: An actor who cannot set the admin verification receipt can publish a malicious release, defeating the intended software-supply-chain boundary.
  Recommendation: Run release authorization and publication from a protected, trusted workflow revision, and enforce matching tag creation and tag targets with a GitHub ruleset outside the tagged repository contents.

- [high] Public installation is tested only after irreversible publication (`.github/workflows/release.yml`:339-375, confidence 0.98)
  Trigger: Draft smoke passes through authenticated release-asset download, publication succeeds, and the public release-download path then fails for consumers. The public smoke workflow runs only after `publish-release` succeeds.
  Impact: All users installing that version can receive a broken release; immutable publication prevents repairing or removing its assets, requiring a replacement release.
  Recommendation: Validate the unauthenticated public download path with identical artifacts in a public release-candidate stage before creating the final immutable release.

- [high] A failed immutability check leaves a public release behind (`.github/workflows/release.yml`:339-390, confidence 0.99)
  Trigger: `gh release edit --draft=false` publishes the draft, but the subsequent immutable-state query returns false—for example after an immutable-release setting change or propagation failure.
  Impact: The workflow fails after exposing a release that violates the immutable-release guarantee; the cleanup job only handles draft-smoke failure, so it does not remove this release.
  Recommendation: Add an always-run cleanup path for publication or immutability-verification failure that deletes the release without deleting the tag.

Next steps:
- Make tag authorization and release workflow provenance enforceable outside this repository revision.
- Add a public release-candidate validation stage and cleanup for failed publication verification.
