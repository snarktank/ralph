#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude] [max_iterations]

set -e

# Parse arguments
TOOL="amp"  # Default to amp for backwards compatibility
MODE="cost-efficient"  # Default to cost-efficient mode
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
    --mode)
      MODE="$2"
      shift 2
      ;;
    --mode=*)
      MODE="${1#*=}"
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

# Validate mode choice
if [[ "$MODE" != "max-quality" && "$MODE" != "cost-efficient" ]]; then
  echo "Error: Invalid mode '$MODE'. Must be 'max-quality' or 'cost-efficient'."
  exit 1
fi

# Helper function to get current story info (Claude only)
get_current_story_id() {
  if [ -f "$PRD_FILE" ]; then
    jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$PRD_FILE" 2>/dev/null
  fi
}

get_current_story_model() {
  if [ -f "$PRD_FILE" ]; then
    local model=$(jq -r '[.userStories[] | select(.passes == false)][0].model // empty' "$PRD_FILE" 2>/dev/null)
    if [ -n "$model" ]; then
      echo "$model"
    else
      echo "sonnet"  # fallback if no model specified
    fi
  else
    echo "sonnet"  # fallback default
  fi
}

get_current_story_failures() {
  if [ -f "$PRD_FILE" ]; then
    local failures=$(jq -r '[.userStories[] | select(.passes == false)][0].failures // 0' "$PRD_FILE" 2>/dev/null)
    echo "${failures:-0}"
  else
    echo "0"
  fi
}

# Calculate effective model based on failures (auto-escalation)
get_effective_model() {
  local assigned=$1
  local failures=$2
  
  if [ "$assigned" == "opus" ]; then
    echo "opus"
  elif [ "$assigned" == "sonnet" ]; then
    if [ "$failures" -ge 2 ]; then
      echo "opus"
    else
      echo "sonnet"
    fi
  else  # haiku
    if [ "$failures" -ge 4 ]; then
      echo "opus"
    elif [ "$failures" -ge 2 ]; then
      echo "sonnet"
    else
      echo "haiku"
    fi
  fi
}

# Increment failures for a story by ID
increment_story_failures() {
  local story_id=$1
  if [ -f "$PRD_FILE" ] && [ -n "$story_id" ]; then
    local tmp_file=$(mktemp)
    jq --arg id "$story_id" '
      .userStories = [.userStories[] | if .id == $id then .failures = ((.failures // 0) + 1) else . end]
    ' "$PRD_FILE" > "$tmp_file" && mv "$tmp_file" "$PRD_FILE"
  fi
}
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

echo "Starting Ralph - Tool: $TOOL - Mode: $MODE - Max iterations: $MAX_ITERATIONS"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="

  # Track current story before running (for failure detection)
  STORY_ID_BEFORE=""
  if [[ "$TOOL" == "claude" ]]; then
    STORY_ID_BEFORE=$(get_current_story_id)
  fi

  # Run the selected tool with the ralph prompt
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  else
    # Claude Code: determine model for this iteration
    if [[ "$MODE" == "max-quality" ]]; then
      MODEL="opus"
    else
      ASSIGNED_MODEL=$(get_current_story_model)
      FAILURES=$(get_current_story_failures)
      MODEL=$(get_effective_model "$ASSIGNED_MODEL" "$FAILURES")
      
      if [ "$MODEL" != "$ASSIGNED_MODEL" ]; then
        echo "  Assigned model: $ASSIGNED_MODEL (failures: $FAILURES) → Escalated to: $MODEL"
      else
        echo "  Using model: $MODEL (failures: $FAILURES)"
      fi
    fi
    
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    OUTPUT=$(claude --dangerously-skip-permissions --print --model "$MODEL" < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  fi
  
  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi
  
  # Check if same story is still incomplete (failure detection)
  if [[ "$TOOL" == "claude" ]] && [[ "$MODE" == "cost-efficient" ]]; then
    STORY_ID_AFTER=$(get_current_story_id)
    if [ -n "$STORY_ID_BEFORE" ] && [ "$STORY_ID_BEFORE" == "$STORY_ID_AFTER" ]; then
      echo "  Story $STORY_ID_BEFORE did not complete. Incrementing failure count."
      increment_story_failures "$STORY_ID_BEFORE"
    fi
  fi
  
  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
