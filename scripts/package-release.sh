#!/bin/sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"

version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
[ -n "$version" ] || { printf 'Could not read package version\n' >&2; exit 1; }

requested_arch="${1:-}"
if [ -z "$requested_arch" ]; then
    requested_arch="$(uname -m)"
fi

case "$(uname -s)-${requested_arch}" in
    Linux-x86_64 | Linux-amd64)
        asset="thre-linux-x86_64"
        binary="target/release/thre"
        ;;
    Linux-aarch64 | Linux-arm64)
        asset="thre-linux-aarch64"
        binary="target/aarch64-unknown-linux-musl/release/thre"
        ;;
    *) printf 'Unsupported release platform: %s-%s\n' "$(uname -s)" "$requested_arch" >&2; exit 1 ;;
esac

cargo test --all-targets --all-features --locked
if [ "$asset" = "thre-linux-aarch64" ]; then
    rustup target add aarch64-unknown-linux-musl
    cargo rustc --release --locked --target aarch64-unknown-linux-musl -- -C linker=rust-lld
else
    cargo build --release --locked
fi

rm -rf dist
mkdir -p "dist/${asset}"
install -m 0755 "$binary" "dist/${asset}/thre"
release_notes="RELEASE_NOTES_${version}.md"
[ -f "$release_notes" ] || { printf 'Missing %s\n' "$release_notes" >&2; exit 1; }
cp README.md LICENSE CHANGELOG.md "$release_notes" "dist/${asset}/"
tar -C "dist/${asset}" -czf "dist/${asset}.tar.gz" thre README.md LICENSE CHANGELOG.md "$release_notes"

if command -v sha256sum >/dev/null 2>&1; then
    (cd dist && sha256sum "${asset}.tar.gz" > "${asset}.tar.gz.sha256")
fi

printf 'Packaged thre %s:\n  dist/%s.tar.gz\n' "$version" "$asset"
