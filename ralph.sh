#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Wrapper to keep the historical root entrypoint.
# The canonical implementation lives in: scripts/ralph/ralph.sh
# Usage: ./ralph.sh [args...]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/scripts/ralph/ralph.sh" "$@"
