# Sno CLI

Official public downloads and product configuration for the Sno CLI. The product source remains private. Compiled executables and the canonical installer are attached to GitHub Releases; they are never committed to this repository.

## Install

Linux x86-64/ARM64 GNU, WSL2 x86-64, and macOS Intel/Apple silicon use the same installer:

```sh
sh -c 'sno_installer_body=$(curl -fsSL https://sno.ai/install) && printf "%s\n" "$sno_installer_body" | sh'
```

The command runs the installer only after its download succeeds. The website must serve the canonical script for an eligible published release. Until that route is available, a published release's `sno-installer.sh` can be downloaded directly from its GitHub asset URL using the same command structure. No release or working website route is claimed by these instructions.

Installation puts `sno` in `~/.local/bin`, repairs the active supported shell's login and interactive startup files, initializes the independent CLI, and prints its absolute executable path and core usage. The current caller's PATH cannot be changed by a child installer; use the printed absolute path in the current conversation. A future shell resolves the new binary ahead of an older Cargo installation.

CLI installation does not install Station or register a machine. To choose a product explicitly:

```sh
~/.local/bin/sno skills get core
~/.local/bin/sno setup          # selects products.json defaultProduct
~/.local/bin/sno setup <product-id>
```

`products.json` provides the public product catalog. Product installation and updates use each product's configured action. Public data adds products without publishing their private source. Only an actual user selection or recorded yes answer installs an offered product.

The installed CLI's manual or automatic update uses the same canonical script with `--update-only`. That mode replaces only the binary, without shell-profile changes, skills, initialization, product setup or registration. The script uses embedded release URLs and GitHub native digests, then executes the candidate's real `--version` before atomically replacing a working binary. A failed candidate leaves the old executable usable.

## Release contents

Every release contains exactly five assets:

- `sno-x86_64-unknown-linux-gnu.tar.gz`
- `sno-aarch64-unknown-linux-gnu.tar.gz`
- `sno-aarch64-apple-darwin.tar.gz`
- `sno-x86_64-apple-darwin.tar.gz`
- `sno-installer.sh`

Native Windows and musl Linux are unsupported. Each archive contains one target-named directory holding only `README.md`, `LICENSE`, and `sno`. No generated PowerShell installer, checksum sidecar, Rust source, Cargo package, private repository archive or build log is published. GitHub-generated source archives contain only this public distribution repository.

## Publishing

The private source repository and owned build hosts compile and test the four archives. The private repository also owns the single canonical installer template and its data-filling script. This repository never builds product source or carries a second installer template.

```sh
scripts/release.sh check 1.0.0 /absolute/path/to/four-archives
scripts/release.sh stage 1.0.0 <private-source-commit> /absolute/path/to/four-archives /absolute/path/to/fill-installer-release-data.sh
scripts/release.sh publish 1.0.0
```

`stage` creates a draft with the four archives, finds it in the authenticated release list and reads actual native digests. The private script translates any temporary `untagged-*` draft download URL into its final version-tag URL, populates the canonical installer, then uploads that fifth asset. The fill script contract is `FILL_SCRIPT VERSION RELEASE_JSON OUTPUT`. Native installation checks against a draft need publisher access; anonymous users consume published releases.

`check` accepts either four archives before staging or the final five assets after download. `publish` downloads the draft, verifies the five-asset contract and actual native archive digests, and publishes it. Prereleases remain prereleases. A published release is never replaced. External staging/publication requires explicit release authorization.

## License

The distributed Sno CLI binaries are licensed under Apache License 2.0.
