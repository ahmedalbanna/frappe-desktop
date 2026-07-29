#!/usr/bin/env bash
# Download and extract portable MariaDB for the target platform
set -euo pipefail

MARIADB_VERSION="11.4.5"
RESOURCES_DIR="$(dirname "$0")/../src-tauri/resources"

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86-64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) echo "unsupported: $(uname -m)" >&2; exit 1 ;;
    esac
}

download_mariadb_linux() {
    local arch="$1"
    local url="https://archive.mariadb.org/mariadb-${MARIADB_VERSION}/bintar-linux-systemd/mariadb-${MARIADB_VERSION}-linux-systemd-${arch}.tar.gz"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "Downloading MariaDB ${MARIADB_VERSION} for Linux ${arch} ..."
    curl -L "$url" | tar xz -C "$tmpdir"

    mkdir -p "${RESOURCES_DIR}/mariadb"
    cp "${tmpdir}/mariadb-${MARIADB_VERSION}-linux-systemd-${arch}/bin/mariadbd" "${RESOURCES_DIR}/mariadb/"
    cp "${tmpdir}/mariadb-${MARIADB_VERSION}-linux-systemd-${arch}/bin/mariadb" "${RESOURCES_DIR}/mariadb/"
    cp "${tmpdir}/mariadb-${MARIADB_VERSION}-linux-systemd-${arch}/bin/mysqldump" "${RESOURCES_DIR}/mariadb/"

    rm -rf "$tmpdir"
    echo "Done. MariaDB binaries in ${RESOURCES_DIR}/mariadb/"
}

download_mariadb_windows() {
    local url="https://archive.mariadb.org/mariadb-${MARIADB_VERSION}/winx64-packages/mariadb-${MARIADB_VERSION}-winx64.zip"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "Downloading MariaDB ${MARIADB_VERSION} for Windows x64 ..."
    curl -L "$url" -o "${tmpdir}/mariadb.zip"
    unzip -q "${tmpdir}/mariadb.zip" -d "$tmpdir"

    mkdir -p "${RESOURCES_DIR}/mariadb"
    cp "${tmpdir}/mariadb-${MARIADB_VERSION}-winx64/bin/mariadbd.exe" "${RESOURCES_DIR}/mariadb/"
    cp "${tmpdir}/mariadb-${MARIADB_VERSION}-winx64/bin/mariadb.exe" "${RESOURCES_DIR}/mariadb/"
    cp "${tmpdir}/mariadb-${MARIADB_VERSION}-winx64/bin/mysqldump.exe" "${RESOURCES_DIR}/mariadb/"

    rm -rf "$tmpdir"
    echo "Done. MariaDB binaries in ${RESOURCES_DIR}/mariadb/"
}

case "$(uname -s)" in
    Linux) download_mariadb_linux "$(detect_arch)" ;;
    MINGW*|MSYS*|CYGWIN*) download_mariadb_windows ;;
    Darwin) echo "macOS support coming soon" >&2; exit 1 ;;
    *) echo "Unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac
