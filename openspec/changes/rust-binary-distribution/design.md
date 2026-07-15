## Context

The crate is already published and installs through Cargo, but the repository has no GitHub Release artifacts. Existing CI covers one runner per operating-system family and therefore does not prove Linux ARM64 or both macOS architectures. The release design must serve users without a language runtime, keep the Rust crate as the sole implementation, and make every advertised artifact independently verifiable.

## Goals / Non-Goals

**Goals:**

- Produce native, tested artifacts for Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64.
- Produce static musl artifacts for Linux x64/ARM64 and verify them in Alpine.
- Publish Shell and PowerShell installers, Cargo Binstall-compatible metadata, SHA-256 checksums, and GitHub build provenance.
- Fail before publication when a tag, crate version, target build, archive smoke, or integrity check is wrong.
- Keep release configuration generated from one pinned Rust release tool.

**Non-Goals:**

- npm packages, PyPI packages, Node.js wrappers, Python wrappers, or alternate CLI implementations.
- Windows ARM64 until GitHub's hosted runner is generally available and native runtime evidence exists.
- Linux 32-bit, macOS universal binaries, MSI, Homebrew formulae, self-update, or automatic crates.io publication.
- Changes to CLI commands, state, telemetry, or service protocols.

## Decisions

### Use cargo-dist 0.32.0 as the release authority

`cargo-dist` generates archives, checksums, installers, manifests, Cargo Binstall metadata, provenance steps, and the GitHub Release transaction from one configuration. The version is exact. Its generated workflow is then security-hardened to use repository-pinned tool hashes, read-only default permissions, and a pre-host installer dependency that version 0.32.0 cannot express; CI checks those invariants plus the exact seven-target plan.

Alternative rejected: hand-written matrix and archive scripts. They duplicate target selection, archive naming, installer selection, checksums, and release upload behavior across shells.

### Support five native targets and two musl variants

The five native targets match the settled product scope. Linux musl x64/ARM64 are added because static artifacts materially improve container, Alpine, and older-distribution portability without adding a new product runtime. Windows ARM64 remains excluded while its standard hosted runner is public preview and outside GitHub's service-level guarantee.

### Runtime-test before packaging and after extraction

Native runner jobs must execute the release binary with `--version`, `--help`, and a fresh-profile local Station workflow. Musl jobs additionally run the artifact inside a pinned Alpine container. The generated archive is extracted and tested again so packaging errors cannot hide behind a successful build.

Shell and PowerShell installers are first executed against the exact staged archives by overriding their supported download base URL to the local artifact directory. The workflow then uploads the same bytes to a mutable draft release, downloads them back from GitHub on every native platform, and repeats installer execution. It publishes those exact draft bytes under a one-use public candidate tag, runs both installers anonymously on every native platform, and deletes the candidate release and tag before final publication. Only that verified draft can be published and frozen as the immutable release. The workflow finally repeats installation from the final anonymous public URLs; no identity receipt is issued until every stage passes. A later failure is corrected only by a new patch release; immutable assets are never replaced.

### Publish a synchronized patch release from a semantic version tag

Version `0.1.0` is already immutable on crates.io and cannot be republished. Version `0.1.1` was reviewed and published byte-identically, but its GitHub workflow failed closed before building or hosting assets because the immutable-release settings endpoint requires repository Administration read permission that a standard workflow token cannot receive. Version `0.1.2` was also published byte-identically, but its workflow failed closed during cross-platform cargo-dist bootstrap: standard macOS rejected GNU-only checksum flags and Windows rejected ZIP extraction through tar. Version `0.1.3` passed all seven builds but failed closed before hosting when strict platform-specific checksum parsers rejected cargo-dist's valid trailing blank line. Version `0.1.4` passed the corrected builds, archive checks, software bill of materials, and staged installers, created a draft, then failed closed because GitHub CLI cannot download an unpublished draft by tag; cleanup deleted the draft. All tags remain fixed at their reviewed commits. The forward-only recovery is `0.1.5`: the workflow carries the exact numeric draft release ID through authenticated download, candidate copying, publication, and cleanup.

### Use checksums, provenance, and immutable releases

Every archive and installer has SHA-256 metadata. GitHub artifact attestations bind artifacts to the repository, workflow, commit, and event when repository visibility and plan support them. Repository release immutability must be enabled before the first public binary release so tags and assets cannot be replaced after publication.

## Runtime Surface Inventory

| Surface | Decision | Required proof |
|---|---|---|
| `cargo install sno` | reproduce unchanged | clean registry install and real CLI smoke |
| GitHub archive download | new | extract and execute the packaged binary on its native architecture |
| Shell installer | new | install to a temporary directory and execute installed `sno` |
| PowerShell installer | new | install to a temporary directory and execute installed `sno.exe` |
| Cargo Binstall discovery | new | generated manifest maps the exact version and target archives |
| npm/Python entry points | intentionally omit | negative search across active product and release docs |

Environment parity:

- Linux GNU binaries run on their build runners; musl binaries also run in pinned Alpine.
- macOS binaries run on Intel and Apple Silicon hosted runners respectively.
- Windows x64 binaries run on a Windows x64 hosted runner.
- Local Station smoke uses a fresh temporary profile, real filesystem state, and bundled SQLite on every target.

## Risks / Trade-offs

- GitHub attestations may be unavailable while the repository is private on a non-Enterprise plan -> keep attestation enabled but do not call a private release publicly supported; make the repository public before the first binary release.
- Hosted runner image changes can raise the GNU glibc floor -> ship musl assets as the portable Linux path and record runtime conditions in generated release metadata.
- Third-party release tooling can change -> pin `cargo-dist` binaries by version and repository-recorded SHA-256, commit the hardened workflow, and make `dist plan` plus security-invariant checks CI gates.
- Unsigned operating-system binaries can trigger reputation warnings -> publish SHA-256 and GitHub provenance now; add vendor code signing only when company signing identities exist, without weakening current integrity checks.

## Migration Plan

1. Replace npm/PyPI requirements in the active PRD and docs with GitHub Release assets.
2. Add `dist-workspace.toml` and generate the release workflow with pinned `cargo-dist`.
3. Run a non-publishing workflow on every final runner label; each job records its observed architecture and passes the real-binary smoke before the matrix is frozen.
4. Merge one reviewed candidate commit and wait for all local, native, archive, staged-installer, and release-policy checks to pass.
5. Make the repository public and enable immutable releases. Create an active tag ruleset for `refs/tags/v*` that restricts creation, update, and deletion to organization administrators. `scripts/authorize-release.sh` then verifies public visibility, immutable releases, that ruleset, the remote `main` commit, and a successful CI run before writing the reviewed commit to repository variable `SNO_RELEASE_AUTHORIZED_SHA`. The workflow receives no administration token.
6. From a clean checkout of the reviewed commit, package and publish crate `0.1.5`; download the registry archive and require its SHA-256 to match the local package archive.
7. Create tag `v0.1.5` at that exact reviewed commit. The tag-triggered workflow requires public visibility and the commit-bound administrator receipt, repeatedly checks that the remote tag still resolves to the event commit, tests every extracted archive and staged installer, downloads and retests a GitHub draft by its exact numeric release ID, anonymously tests the same bytes through a one-use public candidate, deletes that candidate, and publishes the final draft by that ID. Immutable-state verification uses bounded retries: confirmed mutable state triggers release cleanup, while an unavailable or ambiguous API result retains the release and blocks for operator inspection. A confirmed final anonymous-installer failure deletes the release. Every cleanup preserves the final version tag and requires a forward patch.
8. After installer checks pass, record a release-identity receipt containing the reviewed commit, tag commit, local and registry crate hashes, target archive hashes, and GitHub workflow run.

Rollback is forward-only after an immutable release: yank or deprecate the affected version where supported, fix the workflow, increment the crate version, and publish a new tag. Never replace an existing asset or move a released tag.

## Open Questions

None. Platform expansion beyond these seven artifacts requires a new native-runner proof and an explicit contract update.
