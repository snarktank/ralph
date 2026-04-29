#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="${RALPH_CONFIG_DIR:-$HOME/.config/ralph-codex}"
CONFIG_FILE="$CONFIG_DIR/ralph.env"
BIN_DIR="${RALPH_BIN_DIR:-$HOME/.local/bin}"
SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_NAME="${RALPH_SERVICE_NAME:-ralph-codex.service}"
SERVICE_FILE="$SERVICE_DIR/$SERVICE_NAME"
RUNNER="$BIN_DIR/ralph-codex-run"

mkdir -p "$CONFIG_DIR" "$BIN_DIR" "$SERVICE_DIR"

if [[ ! -f "$CONFIG_FILE" ]]; then
  cp "$SCRIPT_DIR/ralph.env.example" "$CONFIG_FILE"
  echo "Created $CONFIG_FILE"
  echo "Edit PROJECT_ROOT and RALPH_DIR before starting the service."
fi

cat > "$RUNNER" <<EOF
#!/usr/bin/env bash
exec "$SCRIPT_DIR/run-ralph.sh"
EOF
chmod +x "$RUNNER"

cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=Ralph Codex long-running task loop
After=network-online.target

[Service]
Type=simple
ExecStart=$RUNNER
Restart=no
KillSignal=SIGINT
TimeoutStopSec=30

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload

echo "Installed user service: $SERVICE_FILE"
echo
echo "Before starting, edit:"
echo "  $CONFIG_FILE"
echo
echo "Start:"
echo "  systemctl --user start $SERVICE_NAME"
echo
echo "Logs:"
echo "  journalctl --user -u $SERVICE_NAME -f"
echo
echo "Optional, keep user services alive after logout:"
echo "  loginctl enable-linger \"$USER\""
