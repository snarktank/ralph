#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [max_iterations]

set -e

MAX_ITERATIONS=${1:-10}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check for required commands
if ! command -v agent &> /dev/null; then
  echo "Error: 'agent' command not found. Please install Cursor CLI: https://cursor.com/docs/cli"
  exit 1
fi

if ! command -v jq &> /dev/null; then
  echo "Error: 'jq' command not found. Please install jq: brew install jq"
  exit 1
fi

# Find project root (where prd.json should be located)
# Check script directory first, then parent directories up to 3 levels
PROJECT_ROOT="$SCRIPT_DIR"
for i in {0..3}; do
  if [ -f "$PROJECT_ROOT/prd.json" ]; then
    break
  fi
  if [ "$PROJECT_ROOT" = "/" ]; then
    # Fallback to script directory if not found
    PROJECT_ROOT="$SCRIPT_DIR"
    break
  fi
  PROJECT_ROOT="$(dirname "$PROJECT_ROOT")"
done

PRD_FILE="$PROJECT_ROOT/prd.json"
PROGRESS_FILE="$PROJECT_ROOT/progress.txt"
ARCHIVE_DIR="$PROJECT_ROOT/archive"
LAST_BRANCH_FILE="$PROJECT_ROOT/.last-branch"

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

CONSECUTIVE_ERRORS=0
MAX_RETRIES=3
RETRY_DELAY=10
ITERATION=1

while [ $ITERATION -le $MAX_ITERATIONS ]; do
  echo ""
  echo "═══════════════════════════════════════════════════════"
  echo "  Ralph Iteration $ITERATION of $MAX_ITERATIONS"
  echo "═══════════════════════════════════════════════════════"
  
  # Run Cursor CLI agent with the ralph prompt
  # --print flag is required for non-interactive mode and enables shell execution (bash access)
  # --force flag forces allow commands unless explicitly denied
  # --workspace sets the working directory (where prd.json is located)
  OUTPUT=$(agent --print --force --workspace "$PROJECT_ROOT" --output-format text "$(cat "$SCRIPT_DIR/prompt.md")" 2>&1 | tee /dev/stderr) || true
  
  # Check for connection errors - these mean the iteration didn't actually run
  if echo "$OUTPUT" | grep -qE "ConnectError|ETIMEDOUT|ECONNRESET|ENOTFOUND"; then
    CONSECUTIVE_ERRORS=$((CONSECUTIVE_ERRORS + 1))
    echo ""
    echo "⚠️  Connection error detected ($CONSECUTIVE_ERRORS consecutive)"
    
    if [ $CONSECUTIVE_ERRORS -ge $MAX_RETRIES ]; then
      echo "❌ Too many consecutive connection errors. Stopping."
      echo "   Check your network connection and Cursor CLI status."
      exit 1
    fi
    
    # Exponential backoff: 10s, 20s, 40s...
    WAIT_TIME=$((RETRY_DELAY * CONSECUTIVE_ERRORS))
    echo "   Waiting ${WAIT_TIME}s before retry..."
    sleep $WAIT_TIME
    
    # Don't increment iteration - retry this one
    continue
  fi
  
  # Reset error counter on successful connection
  CONSECUTIVE_ERRORS=0
  
  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "✅ Ralph completed all tasks!"
    echo "Completed at iteration $ITERATION of $MAX_ITERATIONS"
    exit 0
  fi
  
  echo "Iteration $ITERATION complete. Continuing..."
  ITERATION=$((ITERATION + 1))
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
