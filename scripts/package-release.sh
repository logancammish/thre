#!/bin/sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"

version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
[ -n "$version" ] || { printf 'Could not read package version\n' >&2; exit 1; }

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) asset="thre-linux-x86_64" ;;
    *) printf 'Unsupported release host: %s-%s\n' "$(uname -s)" "$(uname -m)" >&2; exit 1 ;;
esac

cargo test --all-targets --all-features --locked
cargo build --release --locked

rm -rf dist
mkdir -p "dist/${asset}"
install -m 0755 target/release/thre "dist/${asset}/thre"
cp README.md LICENSE CHANGELOG.md RELEASE_NOTES_0.1.1.md "dist/${asset}/"
tar -C "dist/${asset}" -czf "dist/${asset}.tar.gz" thre README.md LICENSE CHANGELOG.md RELEASE_NOTES_0.1.1.md

if command -v sha256sum >/dev/null 2>&1; then
    (cd dist && sha256sum "${asset}.tar.gz" > "${asset}.tar.gz.sha256")
fi

printf 'Packaged thre %s:\n  dist/%s.tar.gz\n' "$version" "$asset"
