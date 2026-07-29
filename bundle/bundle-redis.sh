#!/usr/bin/env bash
set -euo pipefail

REDIS_VERSION="7.4.2"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESOURCES_DIR="$(cd "${SCRIPT_DIR}/../src-tauri/resources" && pwd)"

download_redis_linux() {
    local url="https://github.com/redis/redis/archive/refs/tags/${REDIS_VERSION}.tar.gz"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "Downloading Redis ${REDIS_VERSION} source for Linux ..."
    curl -L "$url" | tar xz -C "$tmpdir"

    (
        cd "${tmpdir}/redis-${REDIS_VERSION}"
        make -j"$(nproc)" MALLOC=libc
    )

    mkdir -p "${RESOURCES_DIR}/redis"
    cp "${tmpdir}/redis-${REDIS_VERSION}/src/redis-server" "${RESOURCES_DIR}/redis/"
    cp "${tmpdir}/redis-${REDIS_VERSION}/src/redis-cli" "${RESOURCES_DIR}/redis/"

    rm -rf "$tmpdir"
    echo "Done. Redis binaries in ${RESOURCES_DIR}/redis/"
    ls -lh "${RESOURCES_DIR}/redis/"
}

download_redis_windows() {
    local url="https://github.com/redis-windows/redis-windows/releases/download/${REDIS_VERSION}/Redis-${REDIS_VERSION}-Windows-x64.zip"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "Downloading Redis ${REDIS_VERSION} for Windows x64 ..."
    curl -L "$url" -o "${tmpdir}/redis.zip"
    unzip -q "${tmpdir}/redis.zip" -d "$tmpdir"

    mkdir -p "${RESOURCES_DIR}/redis"
    cp "${tmpdir}/redis-server.exe" "${RESOURCES_DIR}/redis/"
    cp "${tmpdir}/redis-cli.exe" "${RESOURCES_DIR}/redis/"
    rm -rf "$tmpdir"
    echo "Done. Redis binaries in ${RESOURCES_DIR}/redis/"
}

case "$(uname -s)" in
    Linux) download_redis_linux ;;
    MINGW*|MSYS*|CYGWIN*) download_redis_windows ;;
    Darwin) echo "macOS support coming soon" >&2; exit 1 ;;
    *) echo "Unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac
