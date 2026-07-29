#!/usr/bin/env bash
# Download and extract portable Python for the target platform
set -euo pipefail

PYTHON_VERSION="3.12.9"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESOURCES_DIR="$(cd "${SCRIPT_DIR}/../src-tauri/resources" && pwd)"

download_python_linux() {
    local arch="$1"
    local url="https://github.com/actions/python-versions/releases/download/${PYTHON_VERSION}-${arch}/python-${PYTHON_VERSION}-linux-${arch}.tar.gz"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "Downloading Python ${PYTHON_VERSION} for Linux ${arch} ..."
    curl -L "$url" | tar xz -C "$tmpdir"

    mkdir -p "${RESOURCES_DIR}/python"
    cp -r "${tmpdir}"/* "${RESOURCES_DIR}/python/"
    rm -rf "$tmpdir"
    echo "Done. Python in ${RESOURCES_DIR}/python/"
}

download_python_windows() {
    local url="https://www.python.org/ftp/python/${PYTHON_VERSION}/python-${PYTHON_VERSION}-embed-amd64.zip"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "Downloading Python ${PYTHON_VERSION} for Windows x64 ..."
    curl -L "$url" -o "${tmpdir}/python.zip"
    unzip -q "${tmpdir}/python.zip" -d "${RESOURCES_DIR}/python/"
    rm -rf "$tmpdir"
    echo "Done. Python in ${RESOURCES_DIR}/python/"
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x64" ;;
        aarch64|arm64) echo "arm64" ;;
        *) echo "unsupported: $(uname -m)" >&2; exit 1 ;;
    esac
}

case "$(uname -s)" in
    Linux) download_python_linux "$(detect_arch)" ;;
    MINGW*|MSYS*|CYGWIN*) download_python_windows ;;
    *) echo "Unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac
