#!/usr/bin/env bash

set -euo pipefail

repo_root="${1:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}"
manifest="${2:-$repo_root/policy/release-surfaces.tsv}"

fail() {
  printf 'release-surface policy failed: %s\n' "$1" >&2
  exit 1
}

command -v git >/dev/null 2>&1 || fail "missing required command: git"
command -v rg >/dev/null 2>&1 || fail "missing required command: rg"
[[ -d "$repo_root/.git" ]] || fail "not a Git repository: $repo_root"
[[ -f "$manifest" ]] || fail "manifest does not exist: $manifest"

is_candidate() {
  case "$1" in
    .github/workflows/*.yml|.github/workflows/*.yaml|Cargo.toml|Cargo.lock|dist-workspace.toml|README.md|CONTRIBUTING.md|policy/release-surfaces.tsv|scripts/*release*.sh|scripts/install-cargo-dist.sh|ai-doc/ACTIVE/PRD/*release*.md|ai-doc/ACTIVE/PRD/*RELEASE*.md|openspec/changes/rust-binary-distribution/*|openspec/changes/rust-binary-distribution/*/*|openspec/changes/rust-binary-distribution/*/*/*|package.json|pyproject.toml|setup.py|setup.cfg|*/package.json|*/pyproject.toml|*/setup.py|*/setup.cfg)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

declare -A disposition=()
declare -A reason=()
while IFS=$'\t' read -r state path explanation extra; do
  [[ -n "${state:-}" ]] || continue
  [[ "$state" != \#* ]] || continue
  [[ -z "${extra:-}" ]] || fail "too many fields for $path"
  [[ "$state" = "include" || "$state" = "exclude" ]] || fail "invalid disposition for $path: $state"
  [[ -n "${path:-}" ]] || fail "manifest entry is missing a path"
  [[ -z "${disposition[$path]+present}" ]] || fail "duplicate manifest entry: $path"
  if [[ "$state" = "exclude" && -z "${explanation:-}" ]]; then
    fail "excluded path requires a reason: $path"
  fi
  disposition["$path"]="$state"
  reason["$path"]="${explanation:-}"
done <"$manifest"

mapfile -t candidates < <(
  git -C "$repo_root" ls-files --cached --others --exclude-standard |
    while IFS= read -r path; do
      [[ -e "$repo_root/$path" ]] && is_candidate "$path" && printf '%s\n' "$path"
    done | LC_ALL=C sort -u
)

for path in "${candidates[@]}"; do
  [[ -n "${disposition[$path]+present}" ]] || fail "candidate is absent from manifest: $path"
done

for path in "${!disposition[@]}"; do
  is_candidate "$path" || fail "manifest path is not a release-surface candidate: $path"
  printf '%s\n' "${candidates[@]}" | rg -Fxq -- "$path" || fail "manifest path is not tracked or present: $path"
  if [[ "$path" = .github/workflows/*.yml || "$path" = .github/workflows/*.yaml ]]; then
    if rg -n -i --pcre2 \
      '(npm\s+publish|twine\s+upload|maturin\s+publish|python\s+-m\s+build|pypa/gh-action-pypi-publish|installers\s*=\s*\[[^]]*"npm"|"npm:[^"]+")' \
      "$repo_root/$path"; then
      fail "prohibited npm or Python release definition detected in $path"
    fi
  fi

  [[ "${disposition[$path]}" = "exclude" ]] && continue
  [[ -f "$repo_root/$path" ]] || fail "included path does not exist: $path"

  case "$path" in
    package.json|pyproject.toml|setup.py|setup.cfg|*/package.json|*/pyproject.toml|*/setup.py|*/setup.cfg)
      fail "non-Rust package surface is included: $path"
      ;;
  esac

  if [[ "$path" != .github/workflows/*.yml && "$path" != .github/workflows/*.yaml ]] && rg -n -i --pcre2 \
    '(npm\s+publish|twine\s+upload|maturin\s+publish|python\s+-m\s+build|pypa/gh-action-pypi-publish|installers\s*=\s*\[[^]]*"npm"|"npm:[^"]+")' \
    "$repo_root/$path"; then
    fail "prohibited npm or Python release definition detected in $path"
  fi
done

printf 'release-surface policy verified: %s candidates, %s governed entries\n' \
  "${#candidates[@]}" "${#disposition[@]}"
