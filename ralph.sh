#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude] [max_iterations]

set -e
set -o pipefail

# Parse arguments
TOOL="amp"  # Default to amp for backwards compatibility
MAX_ITERATIONS=10

while [[ $# -gt 0 ]]; do
  case $1 in
    --tool)
      TOOL="$2"
      shift 2
      ;;
    --tool=*)
      TOOL="${1#*=}"
      shift
      ;;
    *)
      # Assume it's max_iterations if it's a number
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      fi
      shift
      ;;
  esac
done

# Validate tool choice
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp' or 'claude'."
  exit 1
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" || { echo "Error: Cannot determine script directory"; exit 1; }
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"
HINTS_FILE="$SCRIPT_DIR/.ralph-hints.txt"

# Archive previous run if branch changed
if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")
  
  if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
    # Archive the previous run
    DATE=$(date +%Y-%m-%d)
    # Strip "ralph/" prefix from branch name for folder
    FOLDER_NAME="${LAST_BRANCH#ralph/}"
    ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"
    
    echo "Archiving previous run: $LAST_BRANCH"
    if ! mkdir -p "$ARCHIVE_FOLDER"; then
      echo "Error: Failed to create archive folder: $ARCHIVE_FOLDER"
      exit 1
    fi
    [ -f "$PRD_FILE" ] && { cp "$PRD_FILE" "$ARCHIVE_FOLDER/" || echo "Warning: Failed to archive prd.json"; }
    [ -f "$PROGRESS_FILE" ] && { cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/" || echo "Warning: Failed to archive progress.txt"; }
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
    if ! echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"; then
      echo "Warning: Failed to write branch tracking file"
    fi
  fi
fi

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
  if ! { echo "# Ralph Progress Log" && echo "Started: $(date)" && echo "---"; } > "$PROGRESS_FILE"; then
    echo "Error: Failed to initialize progress file"
    exit 1
  fi
fi

echo "Starting Ralph - Tool: $TOOL - Max iterations: $MAX_ITERATIONS"

# Validate required tools are available
if ! command -v jq &>/dev/null; then
  echo "Error: jq is required but not installed."
  exit 1
fi

if [[ "$TOOL" == "amp" ]] && ! command -v amp &>/dev/null; then
  echo "Error: amp is required but not installed."
  exit 1
fi

if [[ "$TOOL" == "claude" ]] && ! command -v claude &>/dev/null; then
  echo "Error: claude is required but not installed."
  exit 1
fi

# Track consecutive errors
ERROR_COUNT=0
MAX_CONSECUTIVE_ERRORS=3

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="

  # Check for user hints file and prepend to prompt if present
  HINTS=""
  if [ -f "$HINTS_FILE" ]; then
    # Atomic read-and-delete: rename first, then read
    HINTS_CONSUMED="${HINTS_FILE}.consumed"
    if mv "$HINTS_FILE" "$HINTS_CONSUMED" 2>/dev/null; then
      HINTS=$(cat "$HINTS_CONSUMED")
      rm -f "$HINTS_CONSUMED"
      echo "📌 Applying user hints to this iteration"
    fi
  fi

  # Run the selected tool with the ralph prompt
  TOOL_EXIT_CODE=0
  if [[ "$TOOL" == "amp" ]]; then
    if [ -n "$HINTS" ]; then
      # Prepend hints to prompt for amp
      OUTPUT=$( (printf '%s\n' "$HINTS"; echo ""; cat "$SCRIPT_DIR/prompt.md") | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || TOOL_EXIT_CODE=$?
    else
      OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || TOOL_EXIT_CODE=$?
    fi
  else
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    if [ -n "$HINTS" ]; then
      # Prepend hints to CLAUDE.md for this iteration
      OUTPUT=$( (printf '%s\n' "$HINTS"; echo ""; echo "---"; echo ""; cat "$SCRIPT_DIR/CLAUDE.md") | claude --dangerously-skip-permissions --print 2>&1 | tee /dev/stderr) || TOOL_EXIT_CODE=$?
    else
      OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || TOOL_EXIT_CODE=$?
    fi
  fi

  # Check for tool errors
  if [ "$TOOL_EXIT_CODE" -ne 0 ]; then
    ERROR_COUNT=$((ERROR_COUNT + 1))
    echo "⚠️  Warning: $TOOL exited with code $TOOL_EXIT_CODE (error $ERROR_COUNT of $MAX_CONSECUTIVE_ERRORS)"
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Iteration $i: $TOOL error (exit code $TOOL_EXIT_CODE)" >> "$PROGRESS_FILE"

    if [ "$ERROR_COUNT" -ge "$MAX_CONSECUTIVE_ERRORS" ]; then
      echo "❌ Error: $MAX_CONSECUTIVE_ERRORS consecutive tool failures. Stopping."
      echo "$(date '+%Y-%m-%d %H:%M:%S') - STOPPED: $MAX_CONSECUTIVE_ERRORS consecutive failures" >> "$PROGRESS_FILE"
      exit 1
    fi
  elif [ -z "$OUTPUT" ] || [ "${#OUTPUT}" -lt 50 ]; then
    # Tool succeeded but output suspiciously short
    ERROR_COUNT=$((ERROR_COUNT + 1))
    echo "⚠️  Warning: $TOOL returned minimal output (error $ERROR_COUNT of $MAX_CONSECUTIVE_ERRORS)"
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Iteration $i: minimal output warning" >> "$PROGRESS_FILE"

    if [ "$ERROR_COUNT" -ge "$MAX_CONSECUTIVE_ERRORS" ]; then
      echo "❌ Error: $MAX_CONSECUTIVE_ERRORS consecutive minimal outputs. Stopping."
      echo "$(date '+%Y-%m-%d %H:%M:%S') - STOPPED: $MAX_CONSECUTIVE_ERRORS minimal outputs" >> "$PROGRESS_FILE"
      exit 1
    fi
  else
    # Success - reset error count
    ERROR_COUNT=0
  fi

  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "✅ Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    echo "$(date '+%Y-%m-%d %H:%M:%S') - COMPLETED at iteration $i" >> "$PROGRESS_FILE"
    exit 0
  fi

  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
