#!/usr/bin/env bash
set -Eeuo pipefail

readonly REPOSITORY="sno-ai/sno-cli"
readonly -a ARCHIVE_ASSETS=(
  "sno-aarch64-apple-darwin.tar.gz"
  "sno-aarch64-unknown-linux-gnu.tar.gz"
  "sno-x86_64-apple-darwin.tar.gz"
  "sno-x86_64-unknown-linux-gnu.tar.gz"
)
readonly -a RELEASE_ASSETS=("${ARCHIVE_ASSETS[@]}" "sno-installer.sh")
TEMP_DIR=""
trap 'if [[ -n "$TEMP_DIR" ]]; then rm -rf -- "$TEMP_DIR"; fi' EXIT

usage() {
  cat >&2 <<'HELP'
Usage:
  scripts/release.sh check VERSION STAGING_DIR
  scripts/release.sh stage VERSION SOURCE_COMMIT STAGING_DIR FILL_SCRIPT
  scripts/release.sh publish VERSION

check accepts four compiled archives or the final five release assets.
stage uploads four archives, then uses the private FILL_SCRIPT to populate the
canonical installer from actual uploaded asset URLs and native GitHub digests.
publish downloads the draft's five assets, verifies native digests and publishes.
HELP
}

fail() { printf 'release failed: %s\n' "$1" >&2; exit 1; }
require_command() { command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"; }
validate_version() {
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]] || fail 'VERSION must be a semantic CLI version such as 1.0.0'
}
sorted_lines() { LC_ALL=C sort; }
validate_asset_names() {
  local actual=$1 expected=$2
  [[ "$actual" == "$expected" ]] || fail "release assets do not match the public distribution contract; expected [$expected], found [$actual]"
}
archive_variable() {
  case "$1" in
    sno-x86_64-unknown-linux-gnu.tar.gz) printf 'SNO_X86_64_LINUX' ;;
    sno-aarch64-unknown-linux-gnu.tar.gz) printf 'SNO_AARCH64_LINUX' ;;
    sno-x86_64-apple-darwin.tar.gz) printf 'SNO_X86_64_MACOS' ;;
    sno-aarch64-apple-darwin.tar.gz) printf 'SNO_AARCH64_MACOS' ;;
  esac
}
archive_digest() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  else shasum -a 256 "$1" | awk '{print $1}'
  fi
}
validate_staging_directory() {
  local version=$1 staging_dir=$2 phase=$3
  local actual expected archive root entries digest variable
  [[ "$staging_dir" = /* && -d "$staging_dir" ]] || fail "STAGING_DIR must be an existing absolute directory: $staging_dir"
  if find "$staging_dir" -mindepth 1 -maxdepth 1 ! -type f -print -quit | grep -q .; then
    fail 'staging directory contains a directory, link, or other non-file entry'
  fi
  actual=$(find "$staging_dir" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; | sorted_lines)
  if [[ "$phase" == final ]]; then expected=$(printf '%s\n' "${RELEASE_ASSETS[@]}" | sorted_lines)
  else expected=$(printf '%s\n' "${ARCHIVE_ASSETS[@]}" | sorted_lines)
  fi
  validate_asset_names "$actual" "$expected"
  for archive in "${ARCHIVE_ASSETS[@]}"; do
    root=${archive%.tar.gz}
    entries=$(tar -tzf "$staging_dir/$archive" | sorted_lines) || fail "cannot list archive: $archive"
    expected=$(printf '%s\n' "$root/" "$root/LICENSE" "$root/README.md" "$root/sno" | sorted_lines)
    [[ "$entries" == "$expected" ]] || fail "$archive contains files outside README.md, LICENSE, and sno"
    if [[ "$phase" == final ]]; then
      variable=$(archive_variable "$archive")
      digest=$(archive_digest "$staging_dir/$archive")
      grep -Fxq "${variable}_URL='https://github.com/$REPOSITORY/releases/download/v$version/$archive'" "$staging_dir/sno-installer.sh" || fail "installer does not select $archive from release v$version"
      grep -Fxq "${variable}_DIGEST='sha256:$digest'" "$staging_dir/sno-installer.sh" || fail "installer digest does not match archive $archive"
    fi
  done
  if [[ "$phase" == final ]]; then
    sh -n "$staging_dir/sno-installer.sh" || fail 'installer is not valid shell'
    grep -Fxq "SNO_RELEASE_VERSION='$version'" "$staging_dir/sno-installer.sh" || fail 'installer does not report the selected release version'
    if grep -Fq 'sno-cli-core' "$staging_dir/sno-installer.sh"; then fail 'installer contains the private repository name'; fi
  fi
}
verify_native_digests() {
  local staging_dir=$1 metadata=$2 archive expected actual
  for archive in "${ARCHIVE_ASSETS[@]}"; do
    expected=$(jq -er --arg name "$archive" '.assets[] | select(.name == $name) | .digest' "$metadata") || fail "GitHub did not provide native digest for $archive"
    actual=$(archive_digest "$staging_dir/$archive")
    [[ "$expected" == "sha256:$actual" ]] || fail "downloaded $archive does not match GitHub native digest"
  done
}
check_command() {
  local version=$1 staging_dir=$2 phase=archives
  validate_version "$version"
  [[ ! -f "$staging_dir/sno-installer.sh" ]] || phase=final
  validate_staging_directory "$version" "$staging_dir" "$phase"
  printf 'release assets verified: version=%s stage=%s\n' "$version" "$phase"
}
stage_command() {
  local version=$1 source_commit=$2 staging_dir=$3 fill_script=$4 tag="v$1" asset names
  local -a upload_paths=() release_flags=()
  require_command gh
  require_command jq
  validate_version "$version"
  [[ "$source_commit" =~ ^[0-9a-f]{40}$ ]] || fail 'SOURCE_COMMIT must be a 40-character lowercase Git commit'
  [[ -f "$fill_script" ]] || fail "private FILL_SCRIPT does not exist: $fill_script"
  validate_staging_directory "$version" "$staging_dir" archives
  if gh release view "$tag" --repo "$REPOSITORY" >/dev/null 2>&1; then fail "release $tag already exists; existing releases are never replaced"; fi
  TEMP_DIR=$(mktemp -d)
  cat > "$TEMP_DIR/notes.md" <<NOTES
# Sno CLI $version

Compiled Sno CLI executables and the canonical installer were built privately for version \`$version\`. The private source baseline reference is \`$source_commit\`.

Install Linux x86-64/ARM64 GNU (including WSL2 x86-64) or macOS Intel/Apple silicon:

\`\`\`sh
sh -c 'sno_installer_body=\$(curl -fsSL https://github.com/$REPOSITORY/releases/download/$tag/sno-installer.sh) && printf "%s\n" "\$sno_installer_body" | sh'
\`\`\`

The product source remains private. GitHub-generated source archives contain only the public distribution repository.
NOTES
  for asset in "${ARCHIVE_ASSETS[@]}"; do upload_paths+=("$staging_dir/$asset"); done
  [[ "$version" != *-* ]] || release_flags+=(--prerelease)
  printf 'creating draft release %s in %s\n' "$tag" "$REPOSITORY" >&2
  gh release create "$tag" --repo "$REPOSITORY" --target main --draft --title "Sno CLI $version" --notes-file "$TEMP_DIR/notes.md" "${release_flags[@]}" "${upload_paths[@]}" >/dev/null || fail "create draft $tag failed; inspect GitHub before retrying"
  gh api "repos/$REPOSITORY/releases" --paginate --jq ".[] | select(.tag_name == \"$tag\")" > "$TEMP_DIR/release.json" || fail "read uploaded asset metadata failed; draft $tag remains unpublished"
  verify_native_digests "$staging_dir" "$TEMP_DIR/release.json"
  bash "$fill_script" "$version" "$TEMP_DIR/release.json" "$TEMP_DIR/sno-installer.sh" || fail "populate canonical installer failed; draft $tag remains unpublished"
  gh release upload "$tag" "$TEMP_DIR/sno-installer.sh" --repo "$REPOSITORY" >/dev/null || fail "upload canonical installer failed; draft $tag remains unpublished"
  names=$(gh api "repos/$REPOSITORY/releases" --paginate --jq ".[] | select(.tag_name == \"$tag\") | .assets[].name" | sorted_lines)
  validate_asset_names "$names" "$(printf '%s\n' "${RELEASE_ASSETS[@]}" | sorted_lines)"
  gh release view "$tag" --repo "$REPOSITORY" --json url --jq '.url'
}
publish_command() {
  local version=$1 tag="v$1" names is_draft
  local -a publish_flags=(--latest)
  require_command gh
  require_command jq
  validate_version "$version"
  TEMP_DIR=$(mktemp -d)
  gh api "repos/$REPOSITORY/releases" --paginate --jq ".[] | select(.tag_name == \"$tag\")" > "$TEMP_DIR/release.json" || fail "draft $tag does not exist"
  is_draft=$(jq -r '.draft' "$TEMP_DIR/release.json")
  [[ "$is_draft" == true ]] || fail "release $tag is not a draft; published releases are never replaced"
  names=$(jq -r '.assets[].name' "$TEMP_DIR/release.json" | sorted_lines)
  validate_asset_names "$names" "$(printf '%s\n' "${RELEASE_ASSETS[@]}" | sorted_lines)"
  mkdir "$TEMP_DIR/assets"
  gh release download "$tag" --repo "$REPOSITORY" --dir "$TEMP_DIR/assets" || fail "download draft $tag failed; release remains unpublished"
  validate_staging_directory "$version" "$TEMP_DIR/assets" final
  verify_native_digests "$TEMP_DIR/assets" "$TEMP_DIR/release.json"
  [[ "$version" != *-* ]] || publish_flags=(--prerelease --latest=false)
  printf 'publishing verified release %s\n' "$tag" >&2
  gh release edit "$tag" --repo "$REPOSITORY" --draft=false "${publish_flags[@]}" >/dev/null || fail "publish $tag failed; inspect actual GitHub status"
  gh release view "$tag" --repo "$REPOSITORY" --json url --jq '.url'
}
case "${1:-}" in
  check) [[ $# -eq 3 ]] || { usage; exit 2; }; check_command "$2" "$3" ;;
  stage) [[ $# -eq 5 ]] || { usage; exit 2; }; stage_command "$2" "$3" "$4" "$5" ;;
  publish) [[ $# -eq 2 ]] || { usage; exit 2; }; publish_command "$2" ;;
  -h|--help) usage ;;
  *) usage; exit 2 ;;
esac
