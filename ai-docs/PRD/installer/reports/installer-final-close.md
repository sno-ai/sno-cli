# Installer final completion report

Status: complete on gpt1.

PRD body SHA-256: 7b4b5ef589359768f6e3a2d1cf498b84ba01108da3da7b7a7991cbd301eaf834

Acceptance: passed

Required work remaining: 0

Owner approval: larry authorized implementation, merge, publication, final acceptance, and completion in this journey.

Verification: `python3 /home/lh/.agents/skills/prd-creator/scripts/prd-lint.py ai-docs/PRD/installer/sno-assemble-PRD.md --delivery` returned `prd-lint: clean`.

The production installer is merged to `main`, published as `v0.1.9`, and installed at `/home/lh/.cargo/bin/sno`. GitHub Actions is disabled. Linux work ran on gpt1. macOS ARM64 work ran on labmba.

The final published skills release is `s-category-v1.0.0`. The installer downloaded the exact `final-skills.tar.gz` release asset and its checksum after resolving the tag object. The published archive SHA-256 is `563b87bf64324f789cb47adcf7132dff351a4f0d552867caf5ca2c0391186ed7`, equal to the tested distribution archive. The real gpt1 ownership manifest records the published tag. Reach 2.0 and both utilities 1.0 match their published hashes and their entry commands run.

QCG-9 passed through the canonical verifier. A new isolated home installed all three published programs and eligible skill text. Two fresh Codex seats passed Reach doctor. The receiver accepted before completing the same nonce-bearing question. The sender's one bounded wait returned the terminal answer, and both reports were acknowledged. Export preserved the accepted-before-completed chain. A real Codex terminal explicitly trusted the installed hook and returned the injected reminder card ID from an addressed prompt. A wrong Reach current link failed before seat setup; restoration passed. Remove deleted owned commands and text while preserving user Reach state. Separate focused installation discovered Claude, Codex, Hermes, and a custom OpenClaw root.

QCG-1 through QCG-10 now have current passing receipts for the same PRD body. QCG-2 uses the final published core candidates and final text-only distribution bytes. No historical pass was inherited for changed bytes.

Release tag target: `ed04cf3cf6b1b90e0eef610597c1e48c414a2d8c`. Published Linux x64 archive SHA-256: `2bf0561701075abd9a008cb4199f7c6f940c607a659feb33ab1f61cd96c50222`. Published macOS ARM64 archive SHA-256: `46cbcfd1e19c4bdb1b519035af1177db026e7db6396be5bb38a45b1bfc382545`. On labmba, macOS 15.7.9 ARM64 with Rust 1.85.0 passed all six installer platform/source tests, both focused install lifecycle tests, the release build, the downloaded archive checksum, and execution of `sno 0.1.9`. Installed gpt1 `sno 0.1.9` binary SHA-256: `8d2721888b2eced65c4a606875c34b31c4628eb23025b28537e40edc4d68f73a`. Prior binaries and the pre-installer program trees remain recoverable under `/home/lh/.local/state/sno-install-backups/20260915T033508Z/`.
