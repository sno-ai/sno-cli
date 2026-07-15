# Codex Adversarial Plan Review

Target: `sno-cli-initial-release` PRD and `rust-binary-distribution` OpenSpec change
Verdict: needs-attention

Do not execute publication yet. The published Rust baseline is verified, but the `0.1.1` release sequence remains contradictory and source synchronization is not mechanically proved; native runners, manifest completeness, and immutable-release API enforcement remain unverified.

Load-bearing claims:
- [VERIFIED] Public `sno` `0.1.0` installs from crates.io, exposes the intended command tree, and completes a fresh-profile local Station smoke. (ai-doc/ACTIVE/PRD/PROBE-RESULTS-sno-cli-initial-release.md:88-106)
- [UNVERIFIED] The `0.1.1` crate, tag, and native assets will derive from one reviewed source commit. (openspec/changes/rust-binary-distribution/specs/rust-binary-distribution/spec.md:36-41; openspec/changes/rust-binary-distribution/tasks.md:28)
  Settle with: a release-identity receipt from a clean checkout that records the reviewed commit and package source manifest, verifies the downloaded crates.io archive matches it, and proves `v0.1.1` resolves to that same commit.
- [UNVERIFIED] Final runner labels can provide native Linux ARM64 and both macOS architectures. (openspec/changes/rust-binary-distribution/design.md:57-62; openspec/changes/rust-binary-distribution/tasks.md:12)
  Settle with: a non-publishing workflow using each final runner label that records `uname -m`, target triple, and successful real-binary smoke results.
- [UNVERIFIED] The versioned release-surface manifest will completely cover active release paths. (openspec/changes/rust-binary-distribution/tasks.md:3-6)
  Settle with: a generated tracked-file inventory showing every release workflow, package manifest, installer definition, and active release document is either governed by the manifest or explicitly excluded.
- [UNVERIFIED] GitHub’s API exposes an enforceable immutable-releases predicate for this repository. (openspec/changes/rust-binary-distribution/specs/rust-binary-distribution/spec.md:61-62; openspec/changes/rust-binary-distribution/tasks.md:26-27)
  Settle with: the exact authenticated API requests, response fields, and failing false-predicate tests used by the release workflow.

Findings:
- [high] `0.1.1` synchronization still has contradictory ordering and no source-identity gate (openspec/changes/rust-binary-distribution/design.md:38-40; openspec/changes/rust-binary-distribution/design.md:71-77; openspec/changes/rust-binary-distribution/tasks.md:28, confidence 0.99)
  The migration plan requires the tag only after all seven artifacts pass, while the task creates the tag before verifying those artifacts; the plan also says “existing version tag,” which conflicts with the required patch release. Semantic-version matching does not prove the published crate and tagged native artifacts contain identical source.
  Recommendation: define one candidate-build → public/immutable preflight → clean-checkout crate publish → tag-exact-reviewed-commit → verification-gated upload sequence, enforced by a release-identity receipt.

- [high] Native architecture enforcement is specified but its required runners are not proven available (openspec/changes/rust-binary-distribution/design.md:57-62; openspec/changes/rust-binary-distribution/tasks.md:11-12, confidence 0.95)
  Host-architecture comparison correctly rejects cross-compilation masquerading as native verification, but no evidence identifies working final runner labels for every required architecture. This can block release only after configuration work is complete.
  Recommendation: make successful no-publish native-runner probes a prerequisite to freezing the matrix and generated workflow.

- [high] The release-surface manifest can certify an incomplete scope (openspec/changes/rust-binary-distribution/tasks.md:3-6, confidence 0.99)
  The proposed check rejects prohibited actions only “within that explicit scope,” but neither the manifest schema nor a completeness rule is defined. A literal implementation can omit an active workflow or package definition and still pass while violating the Rust-only distribution contract.
  Recommendation: require candidate discovery from tracked files and fail when each candidate is not exactly covered or explicitly excluded with a reason.

- [high] The public-and-immutable preflight lacks an executable GitHub API predicate (openspec/changes/rust-binary-distribution/specs/rust-binary-distribution/spec.md:61-62; openspec/changes/rust-binary-distribution/tasks.md:26-27, confidence 0.98)
  The plan requires API enforcement but names no endpoint, response field, accepted value, or negative test. The repository being private is evidenced, but immutable-release state and API observability are not.
  Recommendation: specify and test the exact API calls and JSON predicates; if immutable status cannot be queried, replace this requirement with a vendor-enforced control that can be independently verified before release creation.

Coverage:
- Checked: prior critical/high closure areas only—`0.1.1` synchronization, published Rust baseline, native architecture enforcement, release-surface manifest coverage, and API-enforced public immutable preflight.
- Not checkable from here: live GitHub runner availability, GitHub API semantics and repository settings, actual generated workflow/configuration, release-surface inventory, and post-release crates.io or GitHub assets.

Next steps:
- Resolve the single release sequence and add the source-identity receipt.
- Run no-publish native-runner and immutable-preflight probes before implementation freeze.
- Define manifest discovery and completeness enforcement before relying on its policy check.