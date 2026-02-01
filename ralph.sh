#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude] [--webhook URL] [max_iterations]
# Environment: RALPH_WEBHOOK_URL or DISCORD_WEBHOOK_URL - Discord webhook for notifications

set -e

# Parse arguments
TOOL="claude"  # Default to claude
MAX_ITERATIONS=10
WEBHOOK_URL="${RALPH_WEBHOOK_URL:-${DISCORD_WEBHOOK_URL:-}}"

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
    --webhook)
      WEBHOOK_URL="$2"
      shift 2
      ;;
    --webhook=*)
      WEBHOOK_URL="${1#*=}"
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

# Function to send Discord notification
send_discord_notification() {
  local status="$1"
  local message="$2"
  local color="$3"  # Discord embed color (decimal)

  if [[ -z "$WEBHOOK_URL" ]]; then
    return 0
  fi

  local project_name="Unknown"
  local branch_name=""
  if [[ -f "$PRD_FILE" ]]; then
    project_name=$(jq -r '.project // "Unknown"' "$PRD_FILE" 2>/dev/null || echo "Unknown")
    branch_name=$(jq -r '.branchName // ""' "$PRD_FILE" 2>/dev/null || echo "")
  fi

  local timestamp
  timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

  # Build JSON safely with jq to handle special characters
  local payload
  payload=$(jq -n \
    --arg title "[$project_name] Ralph: $status" \
    --arg desc "$message" \
    --argjson color "$color" \
    --arg branch "$branch_name" \
    --arg tool "$TOOL" \
    --arg ts "$timestamp" \
    '{embeds: [{title: $title, description: $desc, color: $color, fields: [{name: "Branch", value: $branch, inline: true}, {name: "Tool", value: $tool, inline: true}], timestamp: $ts}]}')

  curl -s --max-time 10 -H "Content-Type: application/json" -d "$payload" "$WEBHOOK_URL" > /dev/null 2>&1 || true
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

echo "Starting Ralph - Tool: $TOOL - Max iterations: $MAX_ITERATIONS"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="

  # Run the selected tool with the ralph prompt
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  else
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  fi
  
  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    send_discord_notification "Complete" "All tasks finished successfully at iteration $i of $MAX_ITERATIONS" "5763719"
    exit 0
  fi
  
  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
send_discord_notification "Max Iterations" "Reached $MAX_ITERATIONS iterations without completing all tasks. Manual review needed." "15158332"
exit 1
