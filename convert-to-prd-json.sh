#!/bin/bash
# Wrapper to keep the historical root entrypoint.
# The canonical implementation lives in: scripts/ralph/cursor/convert-to-prd-json.sh
# Usage: ./convert-to-prd-json.sh [args...]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/scripts/ralph/cursor/convert-to-prd-json.sh" "$@"

