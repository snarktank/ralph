#!/usr/bin/env bash
set -Eeuo pipefail

CONFIG_DIR="${RALPH_CONFIG_DIR:-$HOME/.config/ralph-codex}"
CONFIG_FILE="$CONFIG_DIR/ralph.env"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${RALPH_BIN_DIR:-$HOME/.local/bin}"
APT_UPDATED=0
OPTIONAL_MISSING=0

log() {
  printf '%s\n' "$*"
}

warn() {
  printf 'Warning: %s\n' "$*" >&2
}

die() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

apt_update_once() {
  if [[ "$APT_UPDATED" -eq 0 ]]; then
    sudo apt update
    APT_UPDATED=1
  fi
}

apt_package_available() {
  apt-cache show "$1" >/dev/null 2>&1
}

install_package() {
  local package="$1"
  apt_update_once
  sudo apt install -y "$package"
}

install_apt_if_missing() {
  local command_name="$1"
  local package="$2"

  if have_cmd "$command_name"; then
    log "[skip] $command_name already installed: $(command -v "$command_name")"
    return 0
  fi

  log "[install] $command_name missing; installing apt package: $package"
  install_package "$package"

  if have_cmd "$command_name"; then
    log "[ok] $command_name installed: $(command -v "$command_name")"
  else
    die "$command_name is still missing after installing $package"
  fi
}

install_any_apt_if_missing() {
  local command_name="$1"
  shift

  if have_cmd "$command_name"; then
    log "[skip] $command_name already installed: $(command -v "$command_name")"
    return 0
  fi

  apt_update_once

  local package
  for package in "$@"; do
    if apt_package_available "$package"; then
      log "[install] $command_name missing; installing apt package: $package"
      install_package "$package"
      if have_cmd "$command_name"; then
        log "[ok] $command_name installed: $(command -v "$command_name")"
        return 0
      fi
    fi
  done

  warn "$command_name is missing and none of these apt packages worked: $*"
  OPTIONAL_MISSING=1
  return 1
}

ensure_package_for_capability() {
  local label="$1"
  local package="$2"
  shift 2

  if "$@"; then
    log "[skip] $label already available"
    return 0
  fi

  log "[install] $label missing; installing apt package: $package"
  install_package "$package"

  if "$@"; then
    log "[ok] $label available"
  else
    die "$label is still missing after installing $package"
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

ensure_fd_alias() {
  if have_cmd fd; then
    return 0
  fi

  if ! have_cmd fdfind; then
    install_apt_if_missing fdfind fd-find
  fi

  mkdir -p "$BIN_DIR"
  if [[ ! -e "$BIN_DIR/fd" ]]; then
    ln -s "$(command -v fdfind)" "$BIN_DIR/fd"
    log "[ok] created fd alias: $BIN_DIR/fd -> $(command -v fdfind)"
  elif [[ -L "$BIN_DIR/fd" ]]; then
    log "[skip] fd alias already exists: $BIN_DIR/fd"
  else
    warn "$BIN_DIR/fd exists and is not a symlink; leaving it unchanged."
  fi
}

have_docker_compose() {
  have_cmd docker && docker compose version >/dev/null 2>&1
}

ensure_docker_compose() {
  if have_docker_compose; then
    log "[skip] docker compose already available"
    return 0
  fi

  local package
  for package in docker-compose-plugin docker-compose-v2 docker-compose; do
    if apt_package_available "$package"; then
      log "[install] docker compose missing; installing apt package: $package"
      install_package "$package"
      if have_docker_compose; then
        log "[ok] docker compose available"
        return 0
      fi
      if have_cmd docker-compose; then
        warn "legacy docker-compose is available, but 'docker compose' plugin is still missing."
        OPTIONAL_MISSING=1
        return 0
      fi
    fi
  done

  warn "docker compose is missing and no known compose apt package is available."
  OPTIONAL_MISSING=1
}

ensure_docker_group() {
  if ! have_cmd docker; then
    return 0
  fi

  if ! getent group docker >/dev/null 2>&1; then
    warn "docker group does not exist; docker package may not be fully configured yet."
    return 0
  fi

  if id -nG "$USER" | tr ' ' '\n' | grep -Fxq docker; then
    log "[skip] user $USER is already in docker group"
  else
    log "[configure] adding $USER to docker group"
    sudo usermod -aG docker "$USER"
    warn "Docker group membership requires a new login session, or run: newgrp docker"
  fi
}

check_codex() {
  if have_cmd codex; then
    codex exec --help >/dev/null && log "[ok] Codex CLI is available" || warn "codex exists but 'codex exec --help' failed"
  else
    warn "codex command was not found. Codex CLI is not installed by this bootstrap; install/login Codex before running Ralph."
  fi
}

if [[ -f /etc/os-release ]]; then
  . /etc/os-release
  if [[ "${ID:-}" != "linuxmint" ]]; then
    warn "this bootstrap is tuned for Linux Mint; detected ${PRETTY_NAME:-unknown}."
  fi
fi

have_cmd apt || die "apt was not found. This bootstrap targets Linux Mint/Ubuntu-style systems."

log "Installing missing Linux Mint packages. Already installed commands are skipped."

install_apt_if_missing bash bash
install_apt_if_missing git git
install_apt_if_missing jq jq
install_apt_if_missing curl curl
install_apt_if_missing update-ca-certificates ca-certificates
install_apt_if_missing bwrap bubblewrap
install_apt_if_missing tmux tmux
install_apt_if_missing pgrep procps
install_apt_if_missing killall psmisc
install_apt_if_missing unzip unzip
install_apt_if_missing zip zip

ensure_package_for_capability "build-essential tools (gcc, g++, make)" build-essential have_build_essential
install_apt_if_missing pkg-config pkg-config

install_apt_if_missing python3 python3
install_apt_if_missing pip3 python3-pip
ensure_package_for_capability "python3 venv" python3-venv have_python_venv

install_apt_if_missing node nodejs
install_apt_if_missing npm npm

install_apt_if_missing rg ripgrep
ensure_fd_alias
install_apt_if_missing shellcheck shellcheck

install_any_apt_if_missing gh gh || true
install_any_apt_if_missing docker docker.io || true
ensure_docker_compose
ensure_docker_group

mkdir -p "$CONFIG_DIR"
if [[ ! -f "$CONFIG_FILE" ]]; then
  cp "$SCRIPT_DIR/ralph.env.example" "$CONFIG_FILE"
  log "Created $CONFIG_FILE"
  log "Edit PROJECT_ROOT and RALPH_DIR before starting Ralph."
else
  log "Keeping existing $CONFIG_FILE"
fi

check_codex

echo
log "Next steps:"
log "  1. Edit $CONFIG_FILE"
log "  2. Run: $SCRIPT_DIR/check-env.sh"
log "  3. Run: $SCRIPT_DIR/run-ralph.sh"

if [[ "$OPTIONAL_MISSING" -ne 0 ]]; then
  echo
  warn "Some optional tools could not be installed from the current apt repositories. Review the warnings above."
fi
