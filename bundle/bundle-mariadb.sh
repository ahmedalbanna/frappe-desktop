#!/usr/bin/env bash
# Download and extract portable MariaDB for the target platform
set -euo pipefail

MARIADB_VERSION="11.4.5"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESOURCES_DIR="$(cd "${SCRIPT_DIR}/../src-tauri/resources" && pwd)"

detect_arch() {
    # MariaDB archives use underscore format: x86_64, aarch64
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) echo "unsupported: $(uname -m)" >&2; exit 1 ;;
    esac
}

download_mariadb_linux() {
    local arch="$1"
    local url="https://archive.mariadb.org/mariadb-${MARIADB_VERSION}/bintar-linux-systemd-${arch}/mariadb-${MARIADB_VERSION}-linux-systemd-${arch}.tar.gz"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "Downloading MariaDB ${MARIADB_VERSION} for Linux ${arch} ..."
    curl -L "$url" | tar xz -C "$tmpdir"

    mkdir -p "${RESOURCES_DIR}/mariadb"
    local srcdir="${tmpdir}/mariadb-${MARIADB_VERSION}-linux-systemd-${arch}"
    cp "${srcdir}/bin/mariadbd" "${RESOURCES_DIR}/mariadb/"
    cp "${srcdir}/bin/mariadb" "${RESOURCES_DIR}/mariadb/"
    cp "${srcdir}/bin/mariadb-install-db" "${RESOURCES_DIR}/mariadb/" || true
    cp "${srcdir}/bin/mysqldump" "${RESOURCES_DIR}/mariadb/" || true
    cp "${srcdir}/bin/mysql_install_db" "${RESOURCES_DIR}/mariadb/" || true

    chmod +x "${RESOURCES_DIR}/mariadb/"*
    rm -rf "$tmpdir"
    echo "Done. MariaDB binaries in ${RESOURCES_DIR}/mariadb/"
    ls -lh "${RESOURCES_DIR}/mariadb/"
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
