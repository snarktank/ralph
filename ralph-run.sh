#!/bin/bash
# ralph-run.sh — launch wrapper for the Ralph control layer.
#
# Wraps (never modifies) ralph.sh. Responsibilities:
#   * Mint a fresh RALPH_RUN_ID if one isn't already set, and export it so it
#     propagates down the process tree to every agent session. The SessionEnd
#     cost hook stamps it on each cost.jsonl line, making loop spend
#     attributable and separable from ad-hoc sessions (Block G.3).
#   * Run the unchanged ralph.sh with whatever args you pass through.
#   * On completion, dump the full audit/rundown to rundown/YYYY-MM-DD.txt so the
#     morning artifact exists without you running anything (Block H.4).
#
# Usage: ./ralph-run.sh [--tool claude|amp] [max_iterations]

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Fresh run id per launch unless the caller already set one.
if [ -z "$RALPH_RUN_ID" ]; then
  export RALPH_RUN_ID="ralph-$(date +%Y%m%d-%H%M%S)"
fi
export RALPH_TOOL="${RALPH_TOOL:-claude}"

echo "Ralph run id: $RALPH_RUN_ID"

# Run the inherited loop, unchanged. Don't let a non-zero loop exit (max
# iterations reached) skip the rundown.
set +e
"$SCRIPT_DIR/ralph.sh" "$@"
LOOP_RC=$?
set -e

# Dump the dated morning rundown (H.4).
mkdir -p "$SCRIPT_DIR/rundown"
RUNDOWN_FILE="$SCRIPT_DIR/rundown/$(date +%Y-%m-%d).txt"
{
  echo "Ralph rundown — run $RALPH_RUN_ID — generated $(date)"
  echo ""
  python3 "$SCRIPT_DIR/project_audit.py"
} > "$RUNDOWN_FILE" 2>&1 || true
echo "Rundown written to: $RUNDOWN_FILE"

exit $LOOP_RC
