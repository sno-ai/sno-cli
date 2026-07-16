#!/usr/bin/env bash

set -euo pipefail

repo_root="${1:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}"
workflow="$repo_root/.github/workflows/release.yml"

fail() {
  printf 'release-workflow check failed: %s\n' "$1" >&2
  exit 1
}

job_block() {
  local job="$1"
  awk -v heading="  $job:" '
    $0 == heading { active = 1 }
    active && $0 ~ /^  [A-Za-z0-9_-]+:$/ && $0 != heading { exit }
    active { print }
  ' "$workflow"
}

[[ -f "$workflow" ]] || fail "release workflow is missing"
command -v cargo >/dev/null 2>&1 || fail "missing required command: cargo"
rg -q '^# SECURITY HARDENED:' "$workflow" || fail "security-hardening marker is missing"
rg -Uq '^permissions:\n  "contents": "read"$' "$workflow" || fail "root token is not read-only"

while IFS=: read -r line_number line; do
  reference="$(sed -E 's/^.*uses:[[:space:]]*//; s/[[:space:]]+#.*$//' <<<"$line")"
  [[ "$reference" = ./* ]] && continue
  [[ "$reference" =~ @([0-9a-f]{40})$ ]] || fail "mutable action reference at line $line_number: $reference"
done < <(rg -n --no-heading '^[[:space:]]+(-[[:space:]]+)?uses:' "$workflow")

cargo run --quiet --locked \
  --manifest-path "$repo_root/tools/release-policy/Cargo.toml" \
  -- "$workflow" || fail "workflow permission policy rejected the release workflow"

plan="$(job_block plan)"
preflight_job="$(job_block custom-release-preflight)"
local_build="$(job_block build-local-artifacts)"
host="$(job_block host)"
draft_smoke="$(job_block custom-release-draft-installer-smoke)"
candidate_host="$(job_block host-public-candidate)"
candidate_smoke="$(job_block custom-release-candidate-installer-smoke)"
candidate_cleanup="$(job_block cleanup-public-candidate)"
publish="$(job_block publish-release)"
verify_published="$(job_block verify-published-release)"
mutable_cleanup="$(job_block cleanup-confirmed-mutable-release)"
announce="$(job_block announce)"
public_smoke="$(job_block custom-release-public-installer-smoke)"
public_smoke_cleanup="$(job_block cleanup-failed-public-smoke)"
preflight="$repo_root/.github/workflows/release-preflight.yml"

[[ "$(rg -c 'run: scripts/install-cargo-dist\.sh' <<<"$plan")" -eq 1 ]] || fail "plan does not use the verified cargo-dist bootstrap exactly once"
rg -q 'uses: ./\.github/workflows/release-preflight\.yml' <<<"$preflight_job" || fail "release workflow does not call the authorization preflight"
rg -q '^      - custom-release-preflight$' <<<"$local_build" || fail "artifact builds do not depend on the authorization preflight"
[[ "$(rg -c 'run: scripts/install-cargo-dist\.sh' <<<"$local_build")" -eq 1 ]] || fail "local builds do not use the verified cargo-dist bootstrap exactly once"
if rg -q 'cargo-dist-installer\.(sh|ps1)|\|\s*(sh|iex)' "$workflow"; then
  fail "unverified remote bootstrap remains in release workflow"
fi

rg -q '^      - custom-release-installer-smoke$' <<<"$host" || fail "host does not depend on staged installer smoke"
rg -q 'test "\$resolved_commit" = "\$GITHUB_SHA"' <<<"$host" || fail "host does not revalidate the remote tag commit"
rg -q 'gh release create .* --draft ' <<<"$host" || fail "host does not create a draft release"
rg -q 'draft_id:.*steps\.create_release\.outputs\.release_id' <<<"$host" || fail "host does not export the numeric draft release ID"
rg -q '^      - host$' <<<"$draft_smoke" || fail "draft download smoke does not depend on uploaded draft assets"
rg -Uq '^    permissions:\n      contents: write$' <<<"$draft_smoke" || fail "draft download smoke does not receive push access for unpublished assets"
rg -q 'release_id:.*needs\.host\.outputs\.draft_id' <<<"$draft_smoke" || fail "draft download smoke does not receive the exact draft release ID"
rg -q '^      - custom-release-draft-installer-smoke$' <<<"$publish" || fail "release publication does not depend on draft download smoke"
rg -q '^      - custom-release-draft-installer-smoke$' <<<"$candidate_host" || fail "public candidate does not depend on draft installer smoke"
rg -q 'releases/\$\{DRAFT_ID\}/assets' <<<"$candidate_host" || fail "public candidate does not reuse draft assets by immutable release ID"
rg -q 'gh release create "\$candidate_tag"' <<<"$candidate_host" || fail "public candidate release is not created"
rg -q '^      - host-public-candidate$' <<<"$candidate_smoke" || fail "public candidate installer smoke does not depend on candidate hosting"
rg -q '^      - custom-release-candidate-installer-smoke$' <<<"$candidate_cleanup" || fail "public candidate cleanup does not wait for installer smoke"
rg -q 'gh release delete "\$CANDIDATE_TAG"' <<<"$candidate_cleanup" || fail "public candidate release is not cleaned up"
rg -q 'git/refs/tags/\$\{CANDIDATE_TAG\}' <<<"$candidate_cleanup" || fail "public candidate tag is not cleaned up"
rg -q '^      - custom-release-candidate-installer-smoke$' <<<"$publish" || fail "release publication does not depend on public candidate smoke"
rg -q '^      - cleanup-public-candidate$' <<<"$publish" || fail "release publication does not depend on public candidate cleanup"
rg -q 'test "\$resolved_commit" = "\$GITHUB_SHA"' <<<"$publish" || fail "publication does not revalidate the remote tag commit"
rg -q 'repos/\$\{GITHUB_REPOSITORY\}/releases/\$\{DRAFT_ID\}' <<<"$publish" || fail "verified draft is not promoted by exact release ID"
rg -q -- '-F draft=false' <<<"$publish" || fail "verified draft is not promoted to the immutable release"
rg -q '^      - publish-release$' <<<"$verify_published" || fail "immutable verification does not depend on final publication"
rg -q 'for attempt in 1 2 3 4 5' <<<"$verify_published" || fail "immutable verification retries are not bounded"
rg -q "state=unknown" <<<"$verify_published" || fail "ambiguous immutable state is not preserved"
rg -q "needs\.verify-published-release\.outputs\.state == 'mutable'" <<<"$mutable_cleanup" || fail "mutable release cleanup is not confirmation-gated"
rg -q 'gh release delete' <<<"$mutable_cleanup" || fail "confirmed mutable release is not deleted"
rg -q '^      - verify-published-release$' <<<"$announce" || fail "announcement does not depend on immutable verification"
rg -q '^      - announce$' <<<"$public_smoke" || fail "public-path smoke does not run after publication"
rg -q "needs\.custom-release-public-installer-smoke\.result == 'failure'" <<<"$public_smoke_cleanup" || fail "failed public smoke cleanup is not failure-gated"
rg -q 'gh release delete' <<<"$public_smoke_cleanup" || fail "failed public smoke does not delete the release"

staged_wrapper="$repo_root/.github/workflows/release-installer-smoke.yml"
draft_wrapper="$repo_root/.github/workflows/release-draft-installer-smoke.yml"
public_wrapper="$repo_root/.github/workflows/release-public-installer-smoke.yml"
candidate_wrapper="$repo_root/.github/workflows/release-candidate-installer-smoke.yml"
installer_verify="$repo_root/.github/workflows/release-installer-verify.yml"
sbom="$repo_root/.github/workflows/release-sbom.yml"
bootstrap="$repo_root/scripts/install-cargo-dist.sh"

for wrapper in "$staged_wrapper" "$draft_wrapper" "$public_wrapper" "$candidate_wrapper"; do
  rg -Uq '^permissions:\n  contents: read$' "$wrapper" || fail "installer wrapper root token is not read-only: $wrapper"
done
rg -Uq 'release-installer-verify\.yml\n    with:\n      mode: staged' "$staged_wrapper" || fail "staged wrapper does not use the shared verifier"
rg -Uq 'release-installer-verify\.yml\n    with:\n      mode: draft' "$draft_wrapper" || fail "draft wrapper does not use the shared verifier"
rg -q 'release_id:.*inputs\.release_id' "$draft_wrapper" || fail "draft wrapper does not forward the numeric release ID"
rg -Uq '^  verify:\n    permissions:\n      contents: write\n    uses:' "$draft_wrapper" || fail "draft wrapper does not preserve push access for unpublished assets"
if rg -q '^permissions:' "$installer_verify"; then
  fail "shared installer verifier overrides least-privilege permissions from its caller"
fi
rg -Uq 'release-installer-verify\.yml\n    with:\n      mode: public' "$public_wrapper" || fail "public wrapper does not use the shared verifier"
rg -Uq 'release-installer-verify\.yml\n    with:\n      mode: candidate' "$candidate_wrapper" || fail "candidate wrapper does not use the shared verifier"
rg -q 'SNO_DOWNLOAD_URL="file://' "$installer_verify" || fail "staged Unix installer does not consume local artifacts"
rg -q 'sh "artifacts/\$\{\{ matrix\.script \}\}"' "$installer_verify" || fail "staged Unix installer is not executed"
rg -q '& "artifacts/\$\{\{ matrix\.script \}\}"' "$installer_verify" || fail "staged PowerShell installer is not executed"
rg -Fq 'export HOME="$(mktemp -d)"' "$installer_verify" || fail "Unix installer verification does not isolate the user home"
rg -Fq 'export CARGO_HOME="$HOME/.cargo"' "$installer_verify" || fail "Unix installer verification does not install inside the isolated home"
rg -q 'releases/\$\{DRAFT_ID\}/assets' "$installer_verify" || fail "draft smoke does not address the draft by numeric release ID"
rg -q 'Accept: application/octet-stream' "$installer_verify" || fail "draft smoke does not download exact asset bytes"
rg -q 'X-GitHub-Api-Version: 2026-03-10' "$installer_verify" || fail "draft asset download does not pin the GitHub API contract"
if rg -q '\bmapfile\b' "$installer_verify"; then
  fail "shared installer verifier uses mapfile, which is unavailable in macOS Bash 3.2"
fi
rg -Fq "tr -d '\\r'" "$installer_verify" || fail "draft asset ID is not normalized for Windows line endings"
rg -q 'releases/download/\$\{TAG\}' "$installer_verify" || fail "public smoke does not use the public release path"
rg -q 'inputs\.mode.*candidate' "$installer_verify" || fail "shared verifier does not support public candidate assets"
rg -q 'fb8dbee9f182173e062a64a387b21a0badc6fab8b2abf9294973f012972bf6d8' "$sbom" || fail "SBOM generator hash is not repository-pinned"
[[ "$(rg -c 'expected="[0-9a-f]{64}"' "$bootstrap")" -eq 5 ]] || fail "cargo-dist host hashes are incomplete"
rg -q 'shasum -a 256' "$bootstrap" || fail "macOS-compatible cargo-dist hash verification is missing"
rg -q 'unzip -q' "$bootstrap" || fail "Windows-compatible cargo-dist ZIP extraction is missing"
if rg -q 'sha256sum --check.*matrix\.archive|shasum -a 256 --check.*matrix\.archive' "$repo_root/.github/workflows/release-archive-smoke.yml"; then
  fail "archive smoke depends on platform-specific checksum-file parsing"
fi
[[ "$(rg -c 'recorded="\$\{recorded#\\\*\}"' "$repo_root/.github/workflows/release-archive-smoke.yml")" -eq 2 ]] || fail "archive smoke does not validate checksum filenames on both Unix paths"
rg -q 'vars\.SNO_RELEASE_AUTHORIZED_SHA' "$preflight" || fail "preflight does not consume the commit-bound administrator authorization receipt"
rg -Fq 'test "${RELEASE_AUTHORIZED_SHA}" = "${GITHUB_SHA}"' "$preflight" || fail "preflight does not bind administrator authorization to the release commit"
if rg -q 'repos/.*immutable-releases' "$preflight"; then
  fail "preflight uses the administration-only immutable-release endpoint with a workflow token"
fi

printf 'release-workflow security invariants verified\n'
