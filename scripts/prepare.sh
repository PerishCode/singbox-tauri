#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_ROOT_ENV_NAME="SINGBOX_TAURI_RUNTIME_ROOT_PATH"
DEFAULT_VERSION="${SINGBOX_VERSION:-1.13.3}"

FORCE=0
VERSION="${DEFAULT_VERSION}"
RUNTIME_ROOT="${!RUNTIME_ROOT_ENV_NAME:-}"

usage() {
  cat >&2 <<'EOF'
usage: ./scripts/prepare.sh [--force] [--version <version>] [--runtime-root <path>]
EOF
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --force)
      FORCE=1
      shift
      ;;
    --version)
      [[ $# -ge 2 ]] || usage
      VERSION="$2"
      shift 2
      ;;
    --runtime-root)
      [[ $# -ge 2 ]] || usage
      RUNTIME_ROOT="$2"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

command -v curl >/dev/null
command -v tar >/dev/null
command -v python3 >/dev/null

resolve_runtime_root() {
  if [[ -n "${RUNTIME_ROOT}" ]]; then
    printf '%s\n' "${RUNTIME_ROOT}"
    return
  fi

  printf '%s\n' "${ROOT_DIR}/.runtime/dev"
}

normalize_arch() {
  case "$1" in
    arm64|aarch64) printf 'arm64\n' ;;
    x86_64|amd64) printf 'amd64\n' ;;
    *)
      echo "unsupported arch: $1" >&2
      exit 1
      ;;
  esac
}

normalize_os() {
  case "$1" in
    Darwin) printf 'darwin\n' ;;
    Linux) printf 'linux\n' ;;
    *)
      echo "unsupported os: $1" >&2
      exit 1
      ;;
  esac
}

RUNTIME_ROOT="$(resolve_runtime_root)"
if [[ "${RUNTIME_ROOT}" != /* ]]; then
  RUNTIME_ROOT="${ROOT_DIR}/${RUNTIME_ROOT}"
fi

OS="$(normalize_os "$(uname -s)")"
ARCH="$(normalize_arch "$(uname -m)")"
BIN_DIR="${RUNTIME_ROOT}/bin"
CONFIG_DIR="${RUNTIME_ROOT}/config"
LOGS_DIR="${RUNTIME_ROOT}/logs"
STATE_DIR="${RUNTIME_ROOT}/state"
SECRETS_DIR="${RUNTIME_ROOT}/secrets"
SUBSCRIPTIONS_DIR="${RUNTIME_ROOT}/subscriptions"
METADATA_DIR="${RUNTIME_ROOT}/metadata"
VERSION_FILE="${METADATA_DIR}/sing-box-version"
DEST_BIN="${BIN_DIR}/sing-box"
ARCHIVE_NAME="sing-box-${VERSION}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/SagerNet/sing-box/releases/download/v${VERSION}/${ARCHIVE_NAME}"

mkdir -p \
  "${BIN_DIR}" \
  "${CONFIG_DIR}" \
  "${LOGS_DIR}" \
  "${STATE_DIR}" \
  "${SECRETS_DIR}" \
  "${SUBSCRIPTIONS_DIR}" \
  "${METADATA_DIR}"

if [[ ${FORCE} -eq 0 && -x "${DEST_BIN}" && -f "${VERSION_FILE}" && "$(<"${VERSION_FILE}")" == "${VERSION}" ]]; then
  echo "runtime already prepared: ${RUNTIME_ROOT}"
  exit 0
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/singbox-tauri-prepare.XXXXXX")"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "downloading ${DOWNLOAD_URL}"
curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/${ARCHIVE_NAME}"

tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}"
cp "${TMP_DIR}/sing-box-${VERSION}-${OS}-${ARCH}/sing-box" "${DEST_BIN}"
chmod +x "${DEST_BIN}"
printf '%s\n' "${VERSION}" > "${VERSION_FILE}"

export PREPARE_VERSION="${VERSION}"
export PREPARE_OS="${OS}"
export PREPARE_ARCH="${ARCH}"
export PREPARE_RUNTIME_ROOT="${RUNTIME_ROOT}"
export PREPARE_DEST_BIN="${DEST_BIN}"

python3 - <<'PY' > "${METADATA_DIR}/runtime.json"
import json
import os

print(json.dumps({
    "version": os.environ["PREPARE_VERSION"],
    "os": os.environ["PREPARE_OS"],
    "arch": os.environ["PREPARE_ARCH"],
    "runtime_root": os.environ["PREPARE_RUNTIME_ROOT"],
    "binary_path": os.environ["PREPARE_DEST_BIN"],
}, indent=2))
PY

echo "prepared runtime: ${RUNTIME_ROOT}"
echo "binary: ${DEST_BIN}"
