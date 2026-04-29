#!/usr/bin/env bash
set -Eeuo pipefail

CONFIG_FILE="${RALPH_ENV_FILE:-$HOME/.config/ralph-codex/ralph.env}"

if [[ -f "$CONFIG_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$CONFIG_FILE"
fi

SSH_PORT="${RALPH_REMOTE_SSH_PORT:-22}"
WEB_PORTS="${RALPH_REMOTE_WEB_PORTS:-}"
ALLOW_FROM="${RALPH_REMOTE_ALLOW_FROM:-${RALPH_REMOTE_LAN_CIDR:-}}"
ENABLE_UFW="${RALPH_REMOTE_ENABLE_UFW:-0}"
SKIP_UFW=0
APT_UPDATED=0

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

usage() {
  cat <<'USAGE'
Usage:
  ./linux-mint/setup-remote-access.sh [options]

Options:
  --ssh-port <port>       SSH port to allow and optionally configure. Default: 22.
  --web-port <port>       Add one LAN web port to allow. Can be repeated.
  --web-ports "<ports>"   Add a space-separated list of LAN web ports.
  --allow-from <source>    Restrict ufw rules to one client IP or CIDR, for example 192.168.1.10.
  --lan-cidr <cidr>       Backward-compatible alias for --allow-from.
  --enable-ufw            Enable ufw after adding rules.
  --no-ufw                Install/start SSH only; skip ufw package and rules.
  -h, --help              Show this help.

Examples:
  ./linux-mint/setup-remote-access.sh --enable-ufw --allow-from 192.168.1.10
  ./linux-mint/setup-remote-access.sh --web-port 5173 --enable-ufw --allow-from 192.168.1.10
USAGE
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

have_sshd() {
  have_cmd sshd || [[ -x /usr/sbin/sshd ]]
}

apt_update_once() {
  if [[ "$APT_UPDATED" -eq 0 ]]; then
    sudo apt update
    APT_UPDATED=1
  fi
}

install_package() {
  local package="$1"
  apt_update_once
  sudo apt install -y "$package"
}

ensure_command_package() {
  local command_name="$1"
  local package="$2"

  if have_cmd "$command_name"; then
    log "[skip] $command_name already installed: $(command -v "$command_name")"
    return 0
  fi

  log "[install] $command_name missing; installing apt package: $package"
  install_package "$package"

  have_cmd "$command_name" || die "$command_name is still missing after installing $package"
}

ensure_sshd_package() {
  if have_sshd; then
    log "[skip] OpenSSH server already installed"
    return 0
  fi

  log "[install] OpenSSH server missing; installing apt package: openssh-server"
  install_package openssh-server
  have_sshd || die "sshd is still missing after installing openssh-server"
}

validate_port() {
  local port="$1"
  [[ "$port" =~ ^[0-9]+$ ]] || die "invalid port: $port"
  (( port >= 1 && port <= 65535 )) || die "port out of range: $port"
}

normalize_ports() {
  local ports=()
  local port

  for port in $WEB_PORTS; do
    validate_port "$port"
    ports+=("$port")
  done

  if [[ "${#ports[@]}" -eq 0 ]]; then
    return 0
  fi

  printf '%s\n' "${ports[@]}" | awk '!seen[$0]++' | xargs
}

configure_ssh_port() {
  validate_port "$SSH_PORT"

  if [[ "$SSH_PORT" == "22" ]]; then
    return 0
  fi

  log "[configure] setting sshd listen port to $SSH_PORT"
  sudo mkdir -p /etc/ssh/sshd_config.d
  printf 'Port %s\n' "$SSH_PORT" | sudo tee /etc/ssh/sshd_config.d/99-ralph-codex-port.conf >/dev/null

  if [[ -x /usr/sbin/sshd ]]; then
    sudo /usr/sbin/sshd -t
  else
    sudo sshd -t
  fi
}

enable_ssh_service() {
  log "[configure] enabling and starting ssh service"
  sudo systemctl enable --now ssh
  sudo systemctl is-active --quiet ssh || die "ssh service is not active"
}

ufw_allow_tcp() {
  local port="$1"
  local label="$2"

  validate_port "$port"

  if [[ -n "$ALLOW_FROM" ]]; then
    log "[configure] allowing $label from $ALLOW_FROM to tcp/$port"
    sudo ufw allow from "$ALLOW_FROM" to any port "$port" proto tcp
  else
    warn "RALPH_REMOTE_ALLOW_FROM is empty; allowing tcp/$port from any source visible to this machine."
    sudo ufw allow "$port/tcp"
  fi
}

configure_ufw() {
  if [[ "$SKIP_UFW" -eq 1 ]]; then
    warn "Skipping ufw setup because --no-ufw was provided."
    return 0
  fi

  ensure_command_package ufw ufw

  ufw_allow_tcp "$SSH_PORT" "SSH"

  local port
  for port in $NORMALIZED_WEB_PORTS; do
    ufw_allow_tcp "$port" "web"
  done

  if [[ "$ENABLE_UFW" == "1" ]]; then
    log "[configure] enabling ufw"
    sudo ufw --force enable
  else
    warn "ufw rules were added, but ufw was not enabled. Re-run with --enable-ufw when ready."
  fi

  sudo ufw status verbose
}

first_lan_ip() {
  hostname -I 2>/dev/null | awk '{print $1}'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ssh-port)
      [[ $# -ge 2 ]] || die "--ssh-port requires a value"
      SSH_PORT="$2"
      shift 2
      ;;
    --web-port)
      [[ $# -ge 2 ]] || die "--web-port requires a value"
      WEB_PORTS="${WEB_PORTS:+$WEB_PORTS }$2"
      shift 2
      ;;
    --web-ports)
      [[ $# -ge 2 ]] || die "--web-ports requires a value"
      WEB_PORTS="${WEB_PORTS:+$WEB_PORTS }$2"
      shift 2
      ;;
    --allow-from)
      [[ $# -ge 2 ]] || die "--allow-from requires a value"
      ALLOW_FROM="$2"
      shift 2
      ;;
    --lan-cidr)
      [[ $# -ge 2 ]] || die "--lan-cidr requires a value"
      ALLOW_FROM="$2"
      shift 2
      ;;
    --enable-ufw)
      ENABLE_UFW=1
      shift
      ;;
    --no-ufw)
      SKIP_UFW=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

have_cmd apt || die "apt was not found. This helper targets Linux Mint/Ubuntu-style systems."

NORMALIZED_WEB_PORTS="$(normalize_ports)"

ensure_sshd_package
configure_ssh_port
enable_ssh_service
configure_ufw

LAN_IP="$(first_lan_ip)"

echo
log "Remote access is ready."
log "Linux user: $USER"
if [[ -n "$LAN_IP" ]]; then
  if [[ "$SSH_PORT" == "22" ]]; then
    log "Windows SSH:"
    log "  ssh $USER@$LAN_IP"
  else
    log "Windows SSH:"
    log "  ssh -p $SSH_PORT $USER@$LAN_IP"
  fi

  if [[ -n "$NORMALIZED_WEB_PORTS" ]]; then
    echo
    log "LAN web URLs:"
    local_port=""
    for local_port in $NORMALIZED_WEB_PORTS; do
      log "  http://$LAN_IP:$local_port"
    done
  fi
else
  warn "Could not detect LAN IP. Run: hostname -I"
fi

echo
log "Status helper:"
log "  ~/codex-ralph/linux-mint/remote-status.sh"
