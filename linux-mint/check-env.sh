#!/usr/bin/env bash
set -Eeuo pipefail

CONFIG_FILE="${RALPH_ENV_FILE:-$HOME/.config/ralph-codex/ralph.env}"

if [[ -f "$CONFIG_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$CONFIG_FILE"
fi

PROJECT_ROOT="${PROJECT_ROOT:-$PWD}"
RALPH_DIR="${RALPH_DIR:-$PROJECT_ROOT/scripts/ralph}"
CODEX_BIN="${CODEX_BIN:-codex}"
RALPH_PROXY_HOST="${RALPH_PROXY_HOST:-127.0.0.1}"
RALPH_PROXY_PORT="${RALPH_PROXY_PORT:-7897}"

FAILED=0
OPTIONAL_MISSING=0

ok() {
  printf '[OK] %s\n' "$*"
}

warn() {
  printf '[WARN] %s\n' "$*" >&2
}

fail() {
  printf '[FAIL] %s\n' "$*" >&2
  FAILED=1
}

optional_missing() {
  printf '[OPTIONAL] %s\n' "$*" >&2
  OPTIONAL_MISSING=1
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

check_command() {
  local name="$1"
  if have_cmd "$name"; then
    ok "$name: $(command -v "$name")"
  else
    fail "missing command: $name"
  fi
}

check_optional_command() {
  local name="$1"
  if have_cmd "$name"; then
    ok "$name: $(command -v "$name")"
  else
    optional_missing "missing optional command: $name"
  fi
}

check_capability() {
  local label="$1"
  shift
  if "$@"; then
    ok "$label"
  else
    fail "missing capability: $label"
  fi
}

check_optional_capability() {
  local label="$1"
  shift
  if "$@"; then
    ok "$label"
  else
    optional_missing "missing optional capability: $label"
  fi
}

check_tcp_port() {
  local host="$1"
  local port="$2"
  if timeout 2 bash -c ":</dev/tcp/$host/$port" 2>/dev/null; then
    ok "Clash mixed port reachable at $host:$port"
  else
    warn "Clash mixed port is not reachable at $host:$port"
  fi
}

have_build_essential() {
  have_cmd gcc && have_cmd g++ && have_cmd make
}

have_python_venv() {
  python3 -m venv --help >/dev/null 2>&1
}

have_fd() {
  have_cmd fd || have_cmd fdfind
}

have_docker_compose() {
  have_cmd docker && docker compose version >/dev/null 2>&1
}

have_sshd() {
  have_cmd sshd || [[ -x /usr/sbin/sshd ]]
}

check_docker_group() {
  if ! have_cmd docker; then
    optional_missing "docker is missing; skipping docker group check"
    return 0
  fi

  if ! getent group docker >/dev/null 2>&1; then
    optional_missing "docker group is missing"
    return 0
  fi

  if id -nG "$USER" | tr ' ' '\n' | grep -Fxq docker; then
    ok "user $USER is in docker group"
  else
    warn "user $USER is not in docker group yet. Run 'newgrp docker' or log out/in after bootstrap."
  fi
}

echo "== System =="
if [[ -f /etc/os-release ]]; then
  . /etc/os-release
  echo "OS: ${PRETTY_NAME:-unknown}"
  if [[ "${ID:-}" != "linuxmint" ]]; then
    warn "This helper is tuned for Linux Mint; current ID is ${ID:-unknown}."
  fi
else
  warn "/etc/os-release not found"
fi

if [[ -r /proc/meminfo ]]; then
  mem_kb="$(awk '/^MemTotal:/ {print $2}' /proc/meminfo)"
  swap_kb="$(awk '/^SwapTotal:/ {print $2}' /proc/meminfo)"
  mem_gb="$((mem_kb / 1024 / 1024))"
  swap_gb="$((swap_kb / 1024 / 1024))"
  echo "Memory: ${mem_gb}GB"
  echo "Swap: ${swap_gb}GB"
  if (( mem_gb < 12 )); then
    warn "Memory is below 12GB. Long Codex runs may be unstable."
  else
    ok "Memory is suitable for long Codex runs."
  fi
  if (( swap_gb < 2 )); then
    warn "Swap is below 2GB. On a 16GB machine, 4-8GB swap is recommended for unattended runs."
  else
    ok "Swap is available."
  fi
fi

available_kb="$(df -Pk "$PROJECT_ROOT" 2>/dev/null | awk 'NR==2 {print $4}')"
if [[ -n "${available_kb:-}" ]]; then
  available_gb="$((available_kb / 1024 / 1024))"
  echo "Free disk at PROJECT_ROOT: ${available_gb}GB"
  if (( available_gb < 5 )); then
    warn "Free disk space is below 5GB. runs/ logs and project builds may fill the disk."
  else
    ok "Disk space is suitable."
  fi
fi

echo
echo "== Required Commands =="
check_command bash
check_command git
check_command jq
check_command curl
check_command bwrap
check_command tmux
check_command unzip
check_command zip
check_command pgrep
check_command killall
check_command update-ca-certificates

echo
echo "== Development Toolchain =="
check_capability "build-essential tools (gcc, g++, make)" have_build_essential
check_command pkg-config
check_command python3
check_command pip3
check_capability "python3 venv" have_python_venv
check_command node
check_command npm
check_command rg
check_capability "fd or fdfind" have_fd
check_command shellcheck

echo
echo "== Optional GitHub/Docker Tools =="
check_optional_command gh
check_optional_command docker
check_optional_capability "docker compose" have_docker_compose
check_docker_group

echo
echo "== Optional Remote Access Tools =="
check_optional_capability "OpenSSH server (sshd)" have_sshd
check_optional_command ufw
check_optional_command ss

echo
echo "== Codex CLI =="
if have_cmd "$CODEX_BIN"; then
  "$CODEX_BIN" --version || warn "codex --version failed"
  "$CODEX_BIN" exec --help >/dev/null && ok "codex exec is available" || fail "codex exec --help failed"
else
  fail "Codex CLI command not found: $CODEX_BIN"
fi

echo
echo "== Clash Proxy =="
check_tcp_port "$RALPH_PROXY_HOST" "$RALPH_PROXY_PORT"

echo
echo "== Ralph Files =="
if [[ -d "$PROJECT_ROOT/.git" ]] || git -C "$PROJECT_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  ok "git repository: $PROJECT_ROOT"
else
  fail "PROJECT_ROOT is not a git repository: $PROJECT_ROOT"
fi

[[ -x "$RALPH_DIR/ralph.sh" ]] && ok "executable: $RALPH_DIR/ralph.sh" || fail "ralph.sh is not executable: $RALPH_DIR/ralph.sh"
[[ -f "$RALPH_DIR/CODEX.md" ]] && ok "found: $RALPH_DIR/CODEX.md" || fail "missing: $RALPH_DIR/CODEX.md"
[[ -f "$RALPH_DIR/prd.json" ]] && ok "found: $RALPH_DIR/prd.json" || warn "missing: $RALPH_DIR/prd.json"

if [[ -f "$RALPH_DIR/prd.json" ]]; then
  jq -e '.userStories and (.userStories | type == "array")' "$RALPH_DIR/prd.json" >/dev/null \
    && ok "prd.json shape is valid" \
    || fail "prd.json does not contain userStories array"
fi

echo
if [[ "$FAILED" -eq 0 ]]; then
  ok "Linux Mint Ralph environment has all required pieces."
  if [[ "$OPTIONAL_MISSING" -ne 0 ]]; then
    warn "Some optional GitHub/Docker/remote-access tools are missing. Ralph can still run, but publishing/container/remote checks may be limited."
  fi
else
  fail "Environment checks failed. Fix the required items above before running Ralph unattended."
  exit 1
fi
