#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [max_iterations] [--worker amp|cursor]
#        or set RALPH_WORKER environment variable (amp|cursor)
#        Default worker is 'amp' if not specified

set -e

# Parse arguments
MAX_ITERATIONS=10
WORKER="${RALPH_WORKER:-amp}"
CURSOR_TIMEOUT="${RALPH_CURSOR_TIMEOUT:-1800}"  # Default: 30 minutes (in seconds)

while [[ $# -gt 0 ]]; do
  case $1 in
    --worker)
      WORKER="$2"
      shift 2
      ;;
    --cursor-timeout)
      CURSOR_TIMEOUT="$2"
      shift 2
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      fi
      shift
      ;;
  esac
done

# Validate worker
if [[ "$WORKER" != "amp" && "$WORKER" != "cursor" ]]; then
  echo "Error: Worker must be 'amp' or 'cursor' (got: $WORKER)" >&2
  exit 1
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"

# Archive previous run if branch changed
if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")
  
  if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
    # Archive the previous run
    DATE=$(date +%Y-%m-%d)
    # Strip "ralph/" prefix from branch name for folder
    FOLDER_NAME=$(echo "$LAST_BRANCH" | sed 's|^ralph/||')
    ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"
    
    echo "Archiving previous run: $LAST_BRANCH"
    mkdir -p "$ARCHIVE_FOLDER"
    [ -f "$PRD_FILE" ] && cp "$PRD_FILE" "$ARCHIVE_FOLDER/"
    [ -f "$PROGRESS_FILE" ] && cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/"
    echo "   Archived to: $ARCHIVE_FOLDER"
    
    # Reset progress file for new run
    echo "# Ralph Progress Log" > "$PROGRESS_FILE"
    echo "Started: $(date)" >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"
  fi
fi

# Track current branch
if [ -f "$PRD_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  if [ -n "$CURRENT_BRANCH" ]; then
    echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"
  fi
fi

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

echo "Starting Ralph - Max iterations: $MAX_ITERATIONS"
echo "Worker: $WORKER"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "═══════════════════════════════════════════════════════"
  echo "  Ralph Iteration $i of $MAX_ITERATIONS (Worker: $WORKER)"
  echo "═══════════════════════════════════════════════════════"
  
  # Select prompt and execute based on worker
  if [[ "$WORKER" == "amp" ]]; then
    # Amp worker: use prompt.md and execute amp
    PROMPT_FILE="$SCRIPT_DIR/prompt.md"
    OUTPUT=$(cat "$PROMPT_FILE" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  elif [[ "$WORKER" == "cursor" ]]; then
    # Cursor worker: use cursor/prompt.cursor.md and execute cursor CLI
    # Uses non-interactive headless mode with file edits enabled
    # Always uses normal spawn (never PTY), stdin is closed (no interactive prompts)
    PROMPT_FILE="$SCRIPT_DIR/cursor/prompt.cursor.md"
    PROMPT_TEXT=$(cat "$PROMPT_FILE")
    # Execute cursor with: --model auto --print --force --approve-mcps
    # stdin is automatically closed when using command substitution in bash
    # Per-iteration hard timeout (wall-clock) - kills process if exceeded
    # Note: MCP cleanup is handled by Cursor CLI itself when processes exit normally
    # If MCP processes are orphaned, they may need manual cleanup (outside scope of this script)
    if command -v timeout >/dev/null 2>&1; then
      OUTPUT=$(timeout "$CURSOR_TIMEOUT" cursor --model auto --print --force --approve-mcps "$PROMPT_TEXT" </dev/null 2>&1 | tee /dev/stderr) || true
      TIMEOUT_EXIT=$?
      if [[ $TIMEOUT_EXIT -eq 124 ]]; then
        echo "Warning: Cursor iteration timed out after ${CURSOR_TIMEOUT} seconds" >&2
      fi
    else
      # Fallback if timeout command is not available
      OUTPUT=$(cursor --model auto --print --force --approve-mcps "$PROMPT_TEXT" </dev/null 2>&1 | tee /dev/stderr) || true
    fi
  fi
  
  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi
  
  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
