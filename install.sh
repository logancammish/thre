#!/bin/sh
set -eu

REPOSITORY="logancammish/thre"
INSTALL_DIR="${THRE_INSTALL_DIR:-${HOME}/.local/bin}"

say() {
    printf '%s\n' "$*"
}

fail() {
    printf 'thre installer: %s\n' "$*" >&2
    exit 1
}

command -v tar >/dev/null 2>&1 || fail "tar is required"

case "$(uname -s)" in
    Linux) platform="linux" ;;
    *) fail "only Linux is currently supported" ;;
esac

case "$(uname -m)" in
    x86_64 | amd64) architecture="x86_64" ;;
    *) fail "unsupported architecture: $(uname -m)" ;;
esac

asset="thre-${platform}-${architecture}.tar.gz"
url="https://github.com/${REPOSITORY}/releases/latest/download/${asset}"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT HUP INT TERM

say "Downloading ${asset}..."
if command -v curl >/dev/null 2>&1; then
    curl --fail --location --proto '=https' --tlsv1.2 --silent --show-error \
        --output "${temporary}/${asset}" "$url" || fail "download failed: $url"
elif command -v wget >/dev/null 2>&1; then
    wget --https-only --quiet --output-document="${temporary}/${asset}" "$url" \
        || fail "download failed: $url"
else
    fail "curl or wget is required"
fi

tar -xzf "${temporary}/${asset}" -C "$temporary"
[ -f "${temporary}/thre" ] || fail "release archive does not contain the thre binary"

mkdir -p "$INSTALL_DIR"
install -m 0755 "${temporary}/thre" "${INSTALL_DIR}/thre"

say "Installed thre to ${INSTALL_DIR}/thre"
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *) say "Add ${INSTALL_DIR} to PATH to run thre from anywhere." ;;
esac
"${INSTALL_DIR}/thre" --version
