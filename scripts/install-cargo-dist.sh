#!/usr/bin/env bash

set -euo pipefail

version="0.32.0"
system="$(uname -s)"
machine="$(uname -m)"

case "$system:$machine" in
  Linux:x86_64)
    asset="cargo-dist-x86_64-unknown-linux-gnu.tar.xz"
    expected="eb52f9fae0d0506774e9f1801c1168f87fa2c87a45e2d64d3ae7c89401929946"
    ;;
  Linux:aarch64|Linux:arm64)
    asset="cargo-dist-aarch64-unknown-linux-gnu.tar.xz"
    expected="d29bcffeb3f8b0c517b4ce0dd2470926ed5cb0bb29d78c6bdd5f88d76ee14a6a"
    ;;
  Darwin:x86_64)
    asset="cargo-dist-x86_64-apple-darwin.tar.xz"
    expected="6243464a8389e006b9256ee548bc795638f1a17113c1b6669c0e05ce89fd05c5"
    ;;
  Darwin:arm64|Darwin:aarch64)
    asset="cargo-dist-aarch64-apple-darwin.tar.xz"
    expected="aa343b2ff78ec2981f17a65140250c5ad6062c74072163f68c5c2686d94763a7"
    ;;
  MINGW*:x86_64|MSYS*:x86_64|CYGWIN*:x86_64)
    asset="cargo-dist-x86_64-pc-windows-msvc.zip"
    expected="26e845cabff12a92911ce960af73a86c8f9b2b2d9072b01dfe5b662acf044fa3"
    ;;
  *)
    printf 'unsupported cargo-dist bootstrap host: %s %s\n' "$system" "$machine" >&2
    exit 1
    ;;
esac

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
archive="$tmp_dir/$asset"
url="https://github.com/axodotdev/cargo-dist/releases/download/v${version}/${asset}"

curl --proto '=https' --tlsv1.2 --fail --location --silent --show-error --output "$archive" "$url"
if command -v sha256sum >/dev/null 2>&1; then
  printf '%s  %s\n' "$expected" "$archive" | sha256sum --check --strict
else
  actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
  [[ "$actual" = "$expected" ]] || {
    printf 'cargo-dist checksum mismatch: expected %s, got %s\n' "$expected" "$actual" >&2
    exit 1
  }
fi

case "$asset" in
  *.zip) tar -xf "$archive" -C "$tmp_dir" ;;
  *.tar.xz) tar -xJf "$archive" -C "$tmp_dir" ;;
esac

binary="$(find "$tmp_dir" -type f \( -name dist -o -name dist.exe \) -print -quit)"
[[ -n "$binary" ]] || {
  printf 'verified cargo-dist archive does not contain dist\n' >&2
  exit 1
}

install_dir="$HOME/.cargo/bin"
mkdir -p "$install_dir"
destination="$install_dir/dist"
[[ "$asset" = *.zip ]] && destination="$destination.exe"
cp "$binary" "$destination"
chmod +x "$destination"

if [[ -n "${GITHUB_PATH:-}" ]]; then
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$install_dir" >>"$GITHUB_PATH"
  else
    printf '%s\n' "$install_dir" >>"$GITHUB_PATH"
  fi
fi

"$destination" --version | grep -Fx "cargo-dist $version"
