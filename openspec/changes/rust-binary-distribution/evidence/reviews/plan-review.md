# Codex Adversarial Plan Review

Target: `ai-doc/ACTIVE/PRD/sno-cli-initial-release.md` and `openspec/changes/rust-binary-distribution`

Verdict: needs-attention

Do not execute the release plan as written: it requires re-publishing an already-public immutable crate version, while the Rust binary, matching-native-runner capability, channel hard cut, and public-release transition remain unproven or unenforced.

Load-bearing claims:
- [CONTRADICTED] `cargo publish` will publish `sno` `0.1.0` under `SnoInfo` (ai-doc/ACTIVE/PRD/sno-cli-initial-release.md:254-254)
  Probe evidence says `sno` `0.1.0` is already public on crates.io (ai-doc/ACTIVE/PRD/PROBE-RESULTS-sno-cli-initial-release.md:53-70).
- [UNVERIFIED] The published Rust crate is usable through Cargo and supports the required local Station workflow (openspec/changes/rust-binary-distribution/proposal.md:1-3)
  Settle with a clean-root `cargo install sno --version 0.1.0`, then capture `sno --version`, `sno --help`, and a fresh-profile `sno station telemetry consent get` from that installed executable. The supplied test evidence covers only the legacy Node CLI.
- [UNVERIFIED] All seven target tuples can be built and executed on matching native architectures, with Alpine execution for musl artifacts (openspec/changes/rust-binary-distribution/design.md:30-36, 57-62)
  Settle with seven CI evidence records containing runner architecture, extracted-archive checksum, and successful version/help/fresh-profile Station smoke; musl records must additionally show execution in the pinned Alpine image.
- [UNVERIFIED] The Rust-only release-channel hard cut covers every active release surface and no npm or Python publishing path remains (openspec/changes/rust-binary-distribution/specs/rust-binary-distribution/spec.md:3-8)
  Settle with a checked-in manifest of active release surfaces plus CI output checking workflow publish steps, package manifests, installers, and docs—not an unspecified document search.
- [UNVERIFIED] The repository will be public and release immutability enabled before binary assets are created (openspec/changes/rust-binary-distribution/specs/rust-binary-distribution/spec.md:60-65)
  Settle with dated GitHub settings evidence immediately before the tag-triggered release, tied to the exact workflow run. Current probe evidence says the repository is private.

Findings:
- [critical] The required crates.io publication cannot succeed (ai-doc/ACTIVE/PRD/sno-cli-initial-release.md:254-254, confidence 0.99)
  The plan requires publishing `sno` `0.1.0`, but the authoritative probe already records that exact package version as public. This bricks the release gate or forces an undocumented version divergence.
  Recommendation: Before implementation, choose one path: release a new version and update every version/tag/package gate, or treat this as GitHub-only distribution for the existing version and replace the re-publication requirement with proof that the tag reconstructs the already-published crate.

- [high] The distribution plan assumes a working Rust CLI without evidence (openspec/changes/rust-binary-distribution/proposal.md:1-3, ai-doc/ACTIVE/PRD/PROBE-RESULTS-sno-cli-initial-release.md:99-114, confidence 0.96)
  The only runtime test evidence is for `@snoai/nodix`; no supplied evidence proves that the published Rust binary exposes Account, Station, and Starport or can complete the required local Station smoke. Discovery is deferred until release-pipeline work is already complete.
  Recommendation: Add a prerequisite baseline gate before the channel hard cut: clean-install the published crate and record version, help tree, and fresh-profile Station behavior.

- [high] Native-target feasibility is deferred without runner-proof or architecture enforcement (openspec/changes/rust-binary-distribution/design.md:30-36, openspec/changes/rust-binary-distribution/tasks.md:9-18, confidence 0.91)
  The requirements demand matching-architecture execution, but the tasks neither identify available runners nor require the workflow to assert its actual architecture. A literal implementation can cross-compile successfully, then fail late when native execution is unavailable or not actually native.
  Recommendation: Run and retain a feasibility job for each tuple before freezing the matrix; make each release job compare its observed architecture with its declared target and fail before packaging on mismatch.

- [high] The release-channel hard cut is not mechanically scoped and invites stale-doc gaming (openspec/changes/rust-binary-distribution/tasks.md:3-5, openspec/changes/rust-binary-distribution/specs/rust-binary-distribution/spec.md:3-8, confidence 0.95)
  “Active release surfaces” is undefined, so a negative search can omit a live publish workflow or instead rewrite historical probe evidence to obtain a green result. The stated requirement forbids release channels, while the planned check is document-oriented and does not inspect publishing behavior.
  Recommendation: Define a versioned manifest of release workflows, package manifests, installer sources, and active docs; fail CI on prohibited publish actions or wrapper definitions in that manifest. Preserve probe records and correct them only through a new dated probe.

- [high] Public visibility and immutable-release status are manual intentions, not release-blocking preconditions (openspec/changes/rust-binary-distribution/tasks.md:24-25, openspec/changes/rust-binary-distribution/specs/rust-binary-distribution/spec.md:60-65, confidence 0.90)
  The plan sequences the settings change before tagging, but no workflow or protected gate fails before release creation when the repository remains private or assets remain mutable. Recorded release evidence is a partial mitigation, not a pre-upload enforcement point.
  Recommendation: Add a named pre-tag release gate with dated settings evidence, and a release-workflow preflight that verifies public visibility before creating or uploading assets; require the immutable-release setting receipt in that same gate.

Coverage:
- Checked: registry/version claims against probe evidence; Rust-binary baseline premise; native target and archive-runtime gates; release-channel hard cut; private-to-public sequencing and enforcement; reality-to-plan omissions exposed by the probe.
- Not checkable from here: current Rust source and `Cargo.toml`; generated workflow behavior; actual runner availability; GitHub visibility/immutability settings; README and contributor files; crate, archive, installer, checksum, and attestation runtime behavior.

Next steps:
- Resolve the already-published-version contradiction before any release work.
- Establish clean-install and per-target native-runner evidence before freezing the target matrix.
- Replace the document-only channel search with a versioned release-surface enforcement gate.
- Add a release-blocking public-visibility and immutability preflight.