# Installer private release check

Host: gpt1. This is a source merge checkpoint, not completed deployment acceptance.

- GitHub Actions is disabled on sno-ai/sno-cli. No new hosted CI was requested.
- The final pre-disable PR run passed native Linux x64/ARM64, macOS ARM64/x64, Windows x64, both static Linux builds, and quality policy on commit 2e79434.
- The subsequent patch restricts Reach selection to the actual published linux-x86_64 and macos-aarch64 pairs. The two absent pairs now return usage exit 2 before fetching.
- Release assets use their GitHub API URL with binary Accept headers. GH_TOKEN is attached only to api.github.com requests. Reqwest handles redirects with its default sensitive-header removal.
- The production GithubSource downloaded the private heartbeat archive on gpt1: SHA-256 5fbeb9222fd522d5fe3b7182712877b27d8d5a16d453aecc76a282a6b1597a96, 14986 bytes. This matches the actual published input.
- The real binary in a fresh HOME reached private release resolution and correctly returned exit 3: no published skills release matches the selected tag. No placement success is claimed.
- Five focused platform/checksum tests pass. The new unsupported-pair test failed with exit 3 before the fix and passes with exit 2 afterward. Formatting and focused library/binary Clippy pass.

## Owner intent check

The owner requested working production code and a prompt merge into main. The patch fixes the concrete private download failure without a new bootstrap command or a second installation path. The current source change is ready for merge. End-to-end delivery is still pending: final S publication, actual installer-owned placement, the live seat/hook journey, and refreshed PRD receipts. Historical receipts are not proof of that unfinished boundary.

## Pre-install gpt1 placement (superseded below)

All 42 regular Reach files match published reach-2.0-linux-x86_64.tar.gz, SHA-256 b087a730097f16f7f115b33257da4783d0b7c927b91a17551fbc78245aafbc34; its installed command prints 2.0. Existing heartbeat and subscription-quota-check files differ from final utility releases. These pre-existing paths are unowned by assemble and must not be silently adopted or overwritten by the installer.

Reference: https://docs.github.com/en/rest/releases/assets#get-a-release-asset

## Completed bootstrap placement

The later explicit bootstrap order authorized a local skills archive before the final skills tag. `sno assemble --reach-version 2.0 --skills-archive /home/lh/code/sno-station-skills/staging/s-category-8474baf96195.tar.gz --json` completed on gpt1 at HOME=/home/lh using the normal release binary. The local option shares archive and contract checks with published installation, records `bootstrap-<sha256>`, and conflicts with a skills tag. Core downloads remain published artifacts.

Permanent receipt: `/home/lh/.local/state/sno-install-backups/20260915T033508Z/receipt.json`. Old program roots and command links were moved into that backup before installation. Prior sno and hook configuration copies are preserved there. The installer now owns `/home/lh/.config/sno/assemble.json`. All 42 Reach files and six files from each utility match their published archives; `sno reach --version` prints 2.0 and both utility help commands exit 0. Installed sno SHA-256: `b91a90d3ed6e12eedf86e7a7d1c632ff96ffa56e1e101598caad3ea1a98f930c`.

The two focused local-archive cases pass, including malformed-archive refusal without program placement. The existing install/repeat/update/remove journey and five platform cases pass. Final published skills acceptance and actual prompt injection remain open. The bootstrap receipt was sent to the publishing lanes so their cleanup can proceed.
