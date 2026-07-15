#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
scan_root="${1:-$repo_root}"
policy="${2:-$repo_root/policy/test-substitutes.json}"

fail() {
  printf 'test-substitute policy failed: %s\n' "$1" >&2
  exit 1
}

for command in jq rg find; do
  command -v "$command" >/dev/null 2>&1 || fail "missing required command: $command"
done
[[ -d "$scan_root" ]] || fail "scan root does not exist: $scan_root"
jq -e '.schema_version == 1' "$policy" >/dev/null || fail "invalid policy schema"

mapfile -t files < <(
  find "$scan_root" -type f \
    \( -name 'Cargo.toml' -o -name 'package.json' -o -name 'pyproject.toml' -o -name '*.rs' -o -name '*.ts' -o -name '*.tsx' -o -name '*.js' -o -name '*.mjs' -o -name '*.py' \) \
    -not -path '*/.git/*' \
    -not -path '*/node_modules/*' \
    -not -path '*/target/*' \
    -not -path '*/.venv/*' \
    -print | LC_ALL=C sort
)

if [[ "${#files[@]}" -eq 0 ]]; then
  printf 'test-substitute policy verified: no scannable implementation files yet\n'
  exit 0
fi

dependency_pattern="$(jq -r '.banned_dependencies | map(gsub("[][(){}.^$*+?|\\\\-]"; "\\\\&")) | join("|")' "$policy")"
source_pattern="$(jq -r '.banned_source_patterns | join("|")' "$policy")"
server_pattern="$(jq -r '.service_server_patterns | join("|")' "$policy")"

mapfile -t manifests < <(printf '%s\n' "${files[@]}" | rg '/(Cargo\.toml|package\.json|pyproject\.toml)$' || true)
if [[ "${#manifests[@]}" -gt 0 ]] && rg -n -i "(^|[^A-Za-z0-9_-])($dependency_pattern)([^A-Za-z0-9_-]|$)" "${manifests[@]}"; then
  fail "forbidden mocking dependency detected"
fi

if rg -n "$source_pattern" "${files[@]}"; then
  fail "forbidden internal mock or monkey-patch detected"
fi

while IFS= read -r server_file; do
  [[ -n "$server_file" ]] || continue
  relative="${server_file#"$scan_root"/}"
  jq -e --arg path "$relative" 'any(.allowed_service_servers[]; .path == $path)' "$policy" >/dev/null || {
    printf '%s\n' "$relative" >&2
    fail "undeclared service replacement detected"
  }
done < <(rg -l "$server_pattern" "${files[@]}" || true)

while IFS= read -r allowed_path; do
  [[ -n "$allowed_path" ]] || continue
  full_path="$scan_root/$allowed_path"
  [[ -f "$full_path" ]] || continue
  if rg -n '0\.0\.0\.0' "$full_path"; then
    fail "allowlisted service server binds a non-loopback address"
  fi
  while IFS= read -r bind_line; do
    [[ -n "$bind_line" ]] || continue
    printf '%s\n' "$bind_line" | rg -q 'TcpListener::bind\s*\(\s*"(127\.0\.0\.1|\[::1\])(:[0-9]+)?"\s*\)' || \
      fail "allowlisted service server has a bind expression that is not an explicit loopback literal"
  done < <(rg --no-line-number 'TcpListener::bind' "$full_path" || true)
done < <(jq -r '.allowed_service_servers[].path' "$policy")

printf 'test-substitute policy verified: %s files, %s allowed external-service server path\n' \
  "${#files[@]}" \
  "$(jq '.allowed_service_servers | length' "$policy")"
