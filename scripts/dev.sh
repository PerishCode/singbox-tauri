#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SESSION_NAME="${SESSION_NAME:-singbox-tauri-dev}"
LOG_DIR="${ROOT_DIR}/.local"
LOG_FILE="${LOG_DIR}/dev.log"

command -v tmux >/dev/null
command -v pnpm >/dev/null

start() {
  if tmux has-session -t "${SESSION_NAME}" 2>/dev/null; then
    echo "session already running: ${SESSION_NAME}" >&2
    exit 1
  fi

  mkdir -p "${LOG_DIR}"
  : > "${LOG_FILE}"

  "${ROOT_DIR}/scripts/prepare.sh"

  tmux new-session -d -s "${SESSION_NAME}" \
    "bash -lc 'cd "${ROOT_DIR}" && pnpm build && pnpm tauri dev --no-watch --no-dev-server 2>&1 | tee -a "${LOG_FILE}"'"

  echo "started: ${SESSION_NAME}"
}

stop() {
  if ! tmux has-session -t "${SESSION_NAME}" 2>/dev/null; then
    echo "session not running: ${SESSION_NAME}" >&2
    exit 1
  fi

  tmux kill-session -t "${SESSION_NAME}"
  echo "stopped: ${SESSION_NAME}"
}

restart() {
  if tmux has-session -t "${SESSION_NAME}" 2>/dev/null; then
    tmux kill-session -t "${SESSION_NAME}"
  fi
  start
}

attach() {
  exec tmux attach -t "${SESSION_NAME}"
}

logs() {
  touch "${LOG_FILE}"
  exec tail -n 200 -f "${LOG_FILE}"
}

status() {
  if tmux has-session -t "${SESSION_NAME}" 2>/dev/null; then
    echo "running: ${SESSION_NAME}"
  else
    echo "stopped: ${SESSION_NAME}"
  fi
}

case "${1:-}" in
  start) start ;;
  stop) stop ;;
  restart) restart ;;
  attach) attach ;;
  logs) logs ;;
  status) status ;;
  *)
    echo "usage: $0 {start|stop|restart|attach|logs|status}" >&2
    exit 1
    ;;
esac
