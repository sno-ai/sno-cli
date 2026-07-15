## ADDED Requirements

### Requirement: Rust-only distribution channels
The project SHALL distribute the `sno` implementation only through crates.io and GitHub Releases. It MUST NOT create npm, PyPI, Node.js, or Python wrapper packages.

#### Scenario: Active release documentation is inspected
- **WHEN** release requirements, workflows, and installation instructions are searched
- **THEN** they contain no npm or Python package distribution path and identify the Rust binary as the sole implementation

#### Scenario: Release-surface inventory is incomplete
- **WHEN** tracked-file discovery finds a workflow, package manifest, installer definition, or active release document that is neither governed nor explicitly excluded with a reason
- **THEN** the release policy check fails before build or publication

### Requirement: Production target matrix
Each release SHALL produce archives for `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, and `x86_64-pc-windows-msvc`.

#### Scenario: Release plan is generated
- **WHEN** the release tool plans the current crate version
- **THEN** exactly the seven supported target triples are represented and no unsupported target is advertised

### Requirement: Native runtime verification
Every advertised target SHALL execute the compiled binary on its matching operating system and architecture before publication. Each job MUST compare the observed host architecture with its declared target and fail on mismatch. The verification MUST cover `sno --version`, `sno --help`, and a fresh-profile local Station workflow.

#### Scenario: A target binary is built
- **WHEN** its build completes on the target runner
- **THEN** all required commands execute successfully using the release binary and real local state

#### Scenario: A musl binary is built
- **WHEN** a Linux musl build completes
- **THEN** the binary also executes successfully inside the pinned Alpine runtime image for the same architecture

### Requirement: Packaged artifact verification
Every release archive SHALL be extracted into a clean directory and the extracted binary SHALL pass the version, help, and local Station smoke before upload.

#### Scenario: Archive contents are wrong
- **WHEN** the expected binary is absent, renamed incorrectly, not executable, or fails after extraction
- **THEN** the release workflow fails before creating a GitHub Release

### Requirement: Version and tag integrity
The production binary release SHALL use forward-only version `0.1.2` after the `0.1.1` workflow failed closed before artifact creation. The release workflow SHALL accept only semantic version tags matching the crate version and SHALL use the committed lockfile for all builds. The crates.io package and GitHub tag SHALL identify the same reviewed source version. The existing `v0.1.1` tag MUST NOT be moved.

#### Scenario: Tag and crate version differ
- **WHEN** a release tag does not select the exact version in `Cargo.toml`
- **THEN** release planning fails before any asset is published

#### Scenario: Crate, tag, and reviewed source differ
- **WHEN** the registry crate SHA-256 differs from the clean-checkout package archive or the version tag resolves to another commit
- **THEN** GitHub artifact publication is blocked and the release-identity receipt is not issued

### Requirement: Integrity and provenance metadata
Every published archive and installer SHALL have SHA-256 integrity metadata. When GitHub artifact attestations are available for the repository, each published asset SHALL have build provenance bound to the source repository, commit, workflow, and triggering event.

#### Scenario: User verifies a downloaded artifact
- **WHEN** the user recomputes its SHA-256 or verifies its GitHub attestation
- **THEN** the result matches the immutable release asset and its recorded build origin

### Requirement: Native installers without language runtimes
The release SHALL include Shell and PowerShell installers that select only a matching supported target archive and install the real Rust binary without requiring Rust, Node.js, or Python.

#### Scenario: Installer runs on a supported target
- **WHEN** a user invokes the installer in a clean environment
- **THEN** it installs a runnable `sno` binary for that exact operating system and architecture

#### Scenario: Installer runs on an unsupported target
- **WHEN** no supported target archive matches the host
- **THEN** installation fails explicitly without downloading a substitute architecture

#### Scenario: Installers are staged
- **WHEN** archives and installers have been generated but are not public
- **THEN** every native platform installs from the exact staged archive through the generated installer and GitHub Release creation remains blocked until all checks pass

#### Scenario: Candidate assets are uploaded
- **WHEN** all local artifact checks pass
- **THEN** the workflow creates a draft release, downloads the exact draft assets from GitHub on every native platform, repeats installer execution, publishes the same bytes as a one-use public candidate, verifies anonymous installation on every native platform, deletes the candidate, and publishes the final immutable release only after every check passes

#### Scenario: Immutable release assets become downloadable
- **WHEN** the release transaction completes
- **THEN** the workflow repeats installation from the real public release URL on every native platform and withholds the release-identity receipt until all checks pass

### Requirement: Immutable public releases
The repository SHALL be public and GitHub release immutability SHALL be enabled before a GitHub binary release is represented as publicly available. An active external ruleset MUST restrict creation, update, and deletion of `refs/tags/v*` to organization administrators. Before tagging, `scripts/authorize-release.sh` MUST verify public visibility, immutable releases, the active ruleset, the exact remote `main` commit, and successful CI before writing that commit to repository variable `SNO_RELEASE_AUTHORIZED_SHA`. Before hosting assets, the release workflow MUST require `.visibility == "public"` and require that variable to equal `GITHUB_SHA`. It MUST re-resolve the remote version tag immediately before draft creation and final publication. After publishing the verified draft, it MUST call the release-by-tag API with bounded retries and require `.immutable == true` before announcement. Confirmed mutable state or a confirmed final anonymous-installer failure MUST delete the release without deleting the final tag. An unavailable or ambiguous post-publication API result MUST retain the release, block the workflow, and require explicit operator inspection. A failed candidate MUST be deleted with its unique candidate tag. Administration credentials MUST NOT enter the workflow. Released tags and assets MUST NOT be moved, replaced, or deleted as an update mechanism.

#### Scenario: A released binary needs correction
- **WHEN** a defect is discovered after publication
- **THEN** the project increments the version and publishes a new immutable release instead of replacing the old asset or moving its tag

### Requirement: Unsupported target promotion
An additional target SHALL NOT be advertised until it builds and passes the same native runner, extracted archive, installer, and local Station checks as existing supported targets.

#### Scenario: Windows ARM64 runner remains preview
- **WHEN** the available standard runner lacks a service-level guarantee
- **THEN** Windows ARM64 remains unsupported even if cross-compilation succeeds
