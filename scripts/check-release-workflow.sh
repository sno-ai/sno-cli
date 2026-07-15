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
rg -q '^# SECURITY HARDENED:' "$workflow" || fail "security-hardening marker is missing"
rg -Uq '^permissions:\n  "contents": "read"$' "$workflow" || fail "root token is not read-only"

while IFS=: read -r line_number line; do
  reference="$(sed -E 's/^.*uses:[[:space:]]*//; s/[[:space:]]+#.*$//' <<<"$line")"
  [[ "$reference" = ./* ]] && continue
  [[ "$reference" =~ @([0-9a-f]{40})$ ]] || fail "mutable action reference at line $line_number: $reference"
done < <(rg -n --no-heading '^[[:space:]]+(-[[:space:]]+)?uses:' "$workflow")

permission_error="$(awk '
  /^  [A-Za-z0-9_-]+:$/ { job = $1; sub(/:$/, "", job); in_permissions = 0 }
  /^    permissions:[[:space:]]*write-all/ { print job ":write-all" }
  /^    permissions:$/ { in_permissions = 1; next }
  in_permissions && /^      [A-Za-z0-9_-]+:[[:space:]]*write/ {
    permission = $1; sub(/:$/, "", permission)
    allowed = (job == "host" && (permission == "attestations" || permission == "contents" || permission == "id-token")) ||
              (job == "publish-release" && permission == "contents") ||
              (job == "cleanup-failed-draft" && permission == "contents")
    if (!allowed) print job ":" permission
  }
  in_permissions && $0 !~ /^      / && $0 !~ /^    permissions:/ { in_permissions = 0 }
' "$workflow")"
[[ -z "$permission_error" ]] || fail "unapproved write permission: $permission_error"

plan="$(job_block plan)"
local_build="$(job_block build-local-artifacts)"
host="$(job_block host)"
draft_smoke="$(job_block custom-release-draft-installer-smoke)"
publish="$(job_block publish-release)"
announce="$(job_block announce)"
public_smoke="$(job_block custom-release-public-installer-smoke)"

[[ "$(rg -c 'run: scripts/install-cargo-dist\.sh' <<<"$plan")" -eq 1 ]] || fail "plan does not use the verified cargo-dist bootstrap exactly once"
[[ "$(rg -c 'run: scripts/install-cargo-dist\.sh' <<<"$local_build")" -eq 1 ]] || fail "local builds do not use the verified cargo-dist bootstrap exactly once"
if rg -q 'cargo-dist-installer\.(sh|ps1)|\|\s*(sh|iex)' "$workflow"; then
  fail "unverified remote bootstrap remains in release workflow"
fi

rg -q '^      - custom-release-installer-smoke$' <<<"$host" || fail "host does not depend on staged installer smoke"
rg -q 'gh release create .* --draft ' <<<"$host" || fail "host does not create a draft release"
rg -q '^      - host$' <<<"$draft_smoke" || fail "draft download smoke does not depend on uploaded draft assets"
rg -q '^      - custom-release-draft-installer-smoke$' <<<"$publish" || fail "release publication does not depend on draft download smoke"
rg -q 'gh release edit .* --draft=false' <<<"$publish" || fail "verified draft is not promoted to the immutable release"
rg -q '^      - publish-release$' <<<"$announce" || fail "announcement does not depend on final publication"
rg -q '^      - announce$' <<<"$public_smoke" || fail "public-path smoke does not run after publication"

staged_wrapper="$repo_root/.github/workflows/release-installer-smoke.yml"
draft_wrapper="$repo_root/.github/workflows/release-draft-installer-smoke.yml"
public_wrapper="$repo_root/.github/workflows/release-public-installer-smoke.yml"
installer_verify="$repo_root/.github/workflows/release-installer-verify.yml"
sbom="$repo_root/.github/workflows/release-sbom.yml"
bootstrap="$repo_root/scripts/install-cargo-dist.sh"

rg -Uq 'release-installer-verify\.yml\n    with:\n      mode: staged' "$staged_wrapper" || fail "staged wrapper does not use the shared verifier"
rg -Uq 'release-installer-verify\.yml\n    with:\n      mode: draft' "$draft_wrapper" || fail "draft wrapper does not use the shared verifier"
rg -Uq 'release-installer-verify\.yml\n    with:\n      mode: public' "$public_wrapper" || fail "public wrapper does not use the shared verifier"
rg -q 'SNO_DOWNLOAD_URL="file://' "$installer_verify" || fail "staged Unix installer does not consume local artifacts"
rg -q 'sh "artifacts/\$\{\{ matrix\.script \}\}"' "$installer_verify" || fail "staged Unix installer is not executed"
rg -q '& "artifacts/\$\{\{ matrix\.script \}\}"' "$installer_verify" || fail "staged PowerShell installer is not executed"
rg -q 'gh release download' "$installer_verify" || fail "draft smoke does not download GitHub release assets"
rg -q 'releases/download/\$\{TAG\}' "$installer_verify" || fail "public smoke does not use the public release path"
rg -q 'fb8dbee9f182173e062a64a387b21a0badc6fab8b2abf9294973f012972bf6d8' "$sbom" || fail "SBOM generator hash is not repository-pinned"
[[ "$(rg -c 'expected="[0-9a-f]{64}"' "$bootstrap")" -eq 5 ]] || fail "cargo-dist host hashes are incomplete"
rg -q 'shasum -a 256' "$bootstrap" || fail "macOS-compatible cargo-dist hash verification is missing"

printf 'release-workflow security invariants verified\n'
