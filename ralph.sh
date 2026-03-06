#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude] [max_iterations]

set -e

# Parse arguments
TOOL="amp"  # Default to amp for backwards compatibility
MAX_ITERATIONS=10
LIMIT_WAIT=3600  # Default: wait 1 hour when rate limited

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
    --limit-wait)
      LIMIT_WAIT="$2"
      shift 2
      ;;
    --limit-wait=*)
      LIMIT_WAIT="${1#*=}"
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

echo "Starting Ralph - Tool: $TOOL - Max iterations: $MAX_ITERATIONS - Limit wait: ${LIMIT_WAIT}s"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="

  while true; do
    # Run the selected tool with the ralph prompt
    if [[ "$TOOL" == "amp" ]]; then
      OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
    else
      # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
      OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
    fi

    # Check for rate/usage limit signals in the output
    if echo "$OUTPUT" | grep -qiE "rate.?limit|429|quota|too many requests|usage limit|limit exceeded|try again later|hit.*limit|you.*hit.*limit"; then
      echo ""

      # Try to parse reset time from message, e.g. "resets 8pm (Asia/Saigon)"
      RESET_TIME_STR=$(echo "$OUTPUT" | grep -oiE "resets [0-9]+(:[0-9]+)?(am|pm)" | grep -oiE "[0-9]+(:[0-9]+)?(am|pm)" | head -1)
      RESET_TZ_STR=$(echo "$OUTPUT" | grep -oiE "\([A-Za-z]+/[A-Za-z_]+\)" | tr -d '()' | head -1)

      WAIT_SECS=$LIMIT_WAIT
      if [[ -n "$RESET_TIME_STR" && -n "$RESET_TZ_STR" ]]; then
        NOW=$(date +%s)
        RESET_EPOCH=$(TZ="$RESET_TZ_STR" date -d "today $RESET_TIME_STR" +%s 2>/dev/null || true)
        if [[ -n "$RESET_EPOCH" && $RESET_EPOCH -le $NOW ]]; then
          RESET_EPOCH=$(TZ="$RESET_TZ_STR" date -d "tomorrow $RESET_TIME_STR" +%s 2>/dev/null || true)
        fi
        if [[ -n "$RESET_EPOCH" && $RESET_EPOCH -gt $NOW ]]; then
          WAIT_SECS=$((RESET_EPOCH - NOW))
          RESET_LOCAL=$(TZ="$RESET_TZ_STR" date -d "@$RESET_EPOCH" "+%H:%M %Z")
          echo "*** Rate limit hit. Waiting until $RESET_LOCAL (${WAIT_SECS}s) before retrying iteration $i... ***"
        else
          echo "*** Rate limit hit. Could not parse reset time. Waiting ${WAIT_SECS}s before retrying iteration $i... ***"
        fi
      else
        echo "*** Rate limit hit. Waiting ${WAIT_SECS}s before retrying iteration $i... ***"
      fi

      REMAINING=$WAIT_SECS
      while [[ $REMAINING -gt 0 ]]; do
        HOURS=$((REMAINING / 3600))
        MINS=$(( (REMAINING % 3600) / 60 ))
        SECS=$((REMAINING % 60))
        printf "\r  Retrying in %02d:%02d:%02d..." "$HOURS" "$MINS" "$SECS"
        sleep 1
        REMAINING=$((REMAINING - 1))
      done
      printf "\r  Retrying now...                    \n"
      continue
    fi

    # No rate limit — exit inner retry loop
    break
  done

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
