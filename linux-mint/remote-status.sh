#!/usr/bin/env bash
set -Eeuo pipefail

CONFIG_FILE="${RALPH_ENV_FILE:-$HOME/.config/ralph-codex/ralph.env}"

if [[ -f "$CONFIG_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$CONFIG_FILE"
fi

PROJECT_ROOT="${PROJECT_ROOT:-$PWD}"
RALPH_DIR="${RALPH_DIR:-$PROJECT_ROOT/scripts/ralph}"
SERVICE_NAME="${RALPH_SERVICE_NAME:-ralph-codex.service}"
WEB_PORTS="${RALPH_REMOTE_WEB_PORTS:-5173 3000 8000 8080 80 443}"

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

section() {
  printf '\n== %s ==\n' "$*"
}

first_lan_ip() {
  hostname -I 2>/dev/null | awk '{print $1}'
}

section "Host"
hostnamectl 2>/dev/null || hostname
echo "LAN IPs: $(hostname -I 2>/dev/null || true)"
uptime || true

section "Resources"
free -h 2>/dev/null || true
df -h "$PROJECT_ROOT" 2>/dev/null || true

section "Ralph Service"
if have_cmd systemctl; then
  systemctl --user is-active "$SERVICE_NAME" 2>/dev/null || true
  systemctl --user status "$SERVICE_NAME" --no-pager -l 2>/dev/null || true
else
  echo "systemctl not found"
fi

section "Ralph Processes"
pgrep -af '[r]alph.sh' || echo "No ralph.sh process found."
pgrep -af '[c]odex' || echo "No codex process found."

section "Ralph Files"
echo "PROJECT_ROOT=$PROJECT_ROOT"
echo "RALPH_DIR=$RALPH_DIR"

if [[ -d "$PROJECT_ROOT" ]]; then
  git -C "$PROJECT_ROOT" status --short --branch 2>/dev/null || true
fi

if [[ -f "$RALPH_DIR/.ralph-state.json" ]]; then
  echo
  echo ".ralph-state.json:"
  if have_cmd jq; then
    jq . "$RALPH_DIR/.ralph-state.json" || true
  else
    cat "$RALPH_DIR/.ralph-state.json"
  fi
fi

if [[ -f "$RALPH_DIR/prd.json" ]]; then
  echo
  echo "Stories:"
  if have_cmd jq; then
    jq '.userStories[]? | {id, title, priority, passes}' "$RALPH_DIR/prd.json" || true
  else
    echo "jq not found; cannot summarize prd.json"
  fi
fi

if [[ -f "$RALPH_DIR/progress.txt" ]]; then
  echo
  echo "progress.txt tail:"
  tail -n 80 "$RALPH_DIR/progress.txt" || true
fi

if [[ -d "$RALPH_DIR/runs" ]]; then
  latest_run="$(find "$RALPH_DIR/runs" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1)"
  if [[ -n "${latest_run:-}" ]]; then
    echo
    echo "Latest run: $latest_run"
    find "$latest_run" -mindepth 2 -maxdepth 2 -name status.json | sort | tail -n 3 | while read -r status_file; do
      echo
      echo "$status_file:"
      if have_cmd jq; then
        jq . "$status_file" || true
      else
        cat "$status_file"
      fi
    done
  fi
fi

section "Web Ports"
LAN_IP="$(first_lan_ip)"
if have_cmd ss; then
  for port in $WEB_PORTS; do
    if ss -ltn "( sport = :$port )" 2>/dev/null | awk 'NR > 1 {found=1} END {exit !found}'; then
      echo "[LISTEN] tcp/$port"
      if [[ -n "$LAN_IP" ]]; then
        echo "         LAN URL: http://$LAN_IP:$port"
        echo "         SSH tunnel: ssh -L $port:127.0.0.1:$port $USER@$LAN_IP"
      fi
    else
      echo "[closed] tcp/$port"
    fi
  done
else
  echo "ss not found; cannot inspect listening ports."
fi
