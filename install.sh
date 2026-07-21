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

architecture_choice="${THRE_ARCH:-}"
if [ -z "$architecture_choice" ]; then
    if [ -r /dev/tty ]; then
        printf '%s\n' \
            "Select your Linux architecture:" \
            "  1) x86-64 (default)" \
            "  2) ARM64 / AArch64" \
            "" \
            "Press Enter to install x86-64, or type 2 for ARM64: " >/dev/tty
        IFS= read -r architecture_choice </dev/tty || architecture_choice=""
    fi
fi

case "$architecture_choice" in
    "" | 1 | x86_64 | amd64 | x86-64) architecture="x86_64" ;;
    2 | aarch64 | arm64 | ARM64) architecture="aarch64" ;;
    *) fail "unknown architecture selection: $architecture_choice" ;;
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
