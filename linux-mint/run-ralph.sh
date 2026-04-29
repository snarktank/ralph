#!/usr/bin/env bash
set -Eeuo pipefail

CONFIG_FILE="${RALPH_ENV_FILE:-$HOME/.config/ralph-codex/ralph.env}"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "Error: missing config file: $CONFIG_FILE" >&2
  echo "Copy linux-mint/ralph.env.example to $CONFIG_FILE and edit it first." >&2
  exit 1
fi

# shellcheck disable=SC1090
source "$CONFIG_FILE"

PROJECT_ROOT="${PROJECT_ROOT:?PROJECT_ROOT is required in $CONFIG_FILE}"
RALPH_DIR="${RALPH_DIR:?RALPH_DIR is required in $CONFIG_FILE}"
RALPH_ITERATIONS="${RALPH_ITERATIONS:-50}"
CODEX_BIN="${CODEX_BIN:-codex}"

RALPH_HTTP_PROXY="${RALPH_HTTP_PROXY:-http://${RALPH_PROXY_HOST:-127.0.0.1}:${RALPH_PROXY_PORT:-7897}}"
RALPH_ALL_PROXY="${RALPH_ALL_PROXY:-socks5h://${RALPH_PROXY_HOST:-127.0.0.1}:${RALPH_PROXY_PORT:-7897}}"
RALPH_NO_PROXY="${RALPH_NO_PROXY:-localhost,127.0.0.1,::1}"

export CODEX_BIN
export RALPH_MODEL="${RALPH_MODEL:-}"
export RALPH_PROFILE="${RALPH_PROFILE:-}"
export RALPH_CODEX_FLAGS="${RALPH_CODEX_FLAGS:-}"
export RALPH_MAX_RETRIES="${RALPH_MAX_RETRIES:-3}"
export RALPH_MAX_STORY_FAILURES="${RALPH_MAX_STORY_FAILURES:-3}"
export RALPH_RETRY_BASE_SECONDS="${RALPH_RETRY_BASE_SECONDS:-10}"
export RALPH_SLEEP_SECONDS="${RALPH_SLEEP_SECONDS:-3}"
export RALPH_REQUIRE_CLEAN="${RALPH_REQUIRE_CLEAN:-0}"

export http_proxy="$RALPH_HTTP_PROXY"
export https_proxy="$RALPH_HTTP_PROXY"
export HTTP_PROXY="$RALPH_HTTP_PROXY"
export HTTPS_PROXY="$RALPH_HTTP_PROXY"
export all_proxy="$RALPH_ALL_PROXY"
export ALL_PROXY="$RALPH_ALL_PROXY"
export no_proxy="$RALPH_NO_PROXY"
export NO_PROXY="$RALPH_NO_PROXY"

if [[ ! -x "$RALPH_DIR/ralph.sh" ]]; then
  echo "Error: ralph.sh is missing or not executable: $RALPH_DIR/ralph.sh" >&2
  exit 1
fi

cd "$PROJECT_ROOT"
exec "$RALPH_DIR/ralph.sh" --tool codex "$RALPH_ITERATIONS"
