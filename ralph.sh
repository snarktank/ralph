#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--webhook URL] [--runner claude|codex]
# Environment: RALPH_WEBHOOK_URL or DISCORD_WEBHOOK_URL - Discord webhook for notifications

set -e

usage() {
  cat <<EOF
Usage: ./ralph.sh [OPTIONS]

Options:
  --webhook URL            Discord webhook URL
  --runner claude|codex    Agent runner to use (default: claude)
  -h, --help               Show this help message
EOF
}

# Parse arguments
WEBHOOK_URL="${RALPH_WEBHOOK_URL:-${DISCORD_WEBHOOK_URL:-}}"
RUNNER="claude"

while [[ $# -gt 0 ]]; do
  case $1 in
    --webhook)
      if [[ -z "${2:-}" || "${2:-}" == -* ]]; then
        echo "Error: --webhook requires a non-empty URL value."
        usage
        exit 1
      fi
      WEBHOOK_URL="$2"
      shift 2
      ;;
    --webhook=*)
      WEBHOOK_URL="${1#*=}"
      if [[ -z "$WEBHOOK_URL" ]]; then
        echo "Error: --webhook requires a non-empty URL value."
        usage
        exit 1
      fi
      shift
      ;;
    --runner)
      if [[ -z "${2:-}" || "${2:-}" == -* ]]; then
        echo "Error: --runner requires a value: claude or codex."
        usage
        exit 1
      fi
      RUNNER="$2"
      shift 2
      ;;
    --runner=*)
      RUNNER="${1#*=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Error: Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ "$RUNNER" != "claude" && "$RUNNER" != "codex" ]]; then
  echo "Error: Invalid --runner value '$RUNNER'. Must be 'claude' or 'codex'."
  usage
  exit 1
fi

# Preflight: verify runner binary exists
if ! command -v "$RUNNER" &>/dev/null; then
  echo "Error: Runner '$RUNNER' not found in PATH."
  exit 1
fi

# Run selected agent runner and capture output/exit status
run_runner() {
  local output_file
  output_file=$(mktemp)
  local exit_code=0

  if [[ "$RUNNER" == "claude" ]]; then
    claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" > "$output_file" 2>&1 || exit_code=$?
  else
    codex exec --dangerously-bypass-approvals-and-sandbox - < "$SCRIPT_DIR/CLAUDE.md" > "$output_file" 2>&1 || exit_code=$?
  fi

  cat "$output_file" >&2
  RUNNER_OUTPUT=$(cat "$output_file")
  rm -f "$output_file"
  RUNNER_EXIT_CODE=$exit_code
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"

CONSECUTIVE_FAILURES=0

# Function to send Discord notification
send_discord_notification() {
  local status="$1"
  local message="$2"
  local color="$3"  # Discord embed color (decimal)

  if [[ -z "$WEBHOOK_URL" ]]; then
    return 0
  fi

  local project_name=""
  local branch_name=""
  if [[ -f "$PRD_FILE" ]]; then
    project_name=$(jq -r '.project // empty' "$PRD_FILE" 2>/dev/null || echo "")
    branch_name=$(jq -r '.branchName // ""' "$PRD_FILE" 2>/dev/null || echo "")
  fi
  # Fallback: use git repo directory name if project is not set
  if [[ -z "$project_name" ]]; then
    project_name=$(basename "$(git rev-parse --show-toplevel 2>/dev/null)" 2>/dev/null || echo "Unknown")
  fi
  # Fallback: use current git branch if branchName is not set
  if [[ -z "$branch_name" ]]; then
    branch_name=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
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
    --arg ts "$timestamp" \
    '{embeds: [{title: $title, description: $desc, color: $color, fields: [{name: "Branch", value: $branch, inline: true}], timestamp: $ts}]}')

  curl -s --max-time 10 -H "Content-Type: application/json" -d "$payload" "$WEBHOOK_URL" > /dev/null 2>&1 || true
}

# Archive previous run if branch changed
if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")

  if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
    # Archive the previous run
    DATE=$(date +%Y-%m-%dT%H%M%S)
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

# Calculate max iterations from prd.json
# = remaining stories + review checkpoints + pending review + buffer
if [ ! -f "$PRD_FILE" ]; then
  echo "Error: No prd.json found at $PRD_FILE"
  exit 1
fi

REMAINING_STORIES=$(jq '[.userStories[] | select(.passes != true)] | length' "$PRD_FILE")
REVIEW_CHECKPOINTS=$(jq '[.userStories[] | select(.passes != true and .reviewAfter == true)] | length' "$PRD_FILE")
REVIEW_PENDING=$(jq 'if .reviewPending == true then 1 else 0 end' "$PRD_FILE")
# +2 buffer for retries
MAX_ITERATIONS=$(( REMAINING_STORIES + REVIEW_CHECKPOINTS + REVIEW_PENDING + 2 ))

if [ "$REMAINING_STORIES" -eq 0 ] && [ "$REVIEW_PENDING" -eq 0 ]; then
  echo "All stories already complete. Nothing to do."
  exit 0
fi

echo "Starting Ralph - $REMAINING_STORIES stories remaining, $REVIEW_CHECKPOINTS review checkpoints, max $MAX_ITERATIONS iterations"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS"
  echo "==============================================================="

  # Run selected agent with the Ralph prompt
  run_runner
  OUTPUT="$RUNNER_OUTPUT"

  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    send_discord_notification "Complete" "All tasks finished successfully at iteration $i of $MAX_ITERATIONS" "5763719"
    exit 0
  fi

  ITERATION_FAILED=0
  if [ "$RUNNER_EXIT_CODE" -ne 0 ]; then
    ITERATION_FAILED=1
    echo "Runner '$RUNNER' failed with exit code $RUNNER_EXIT_CODE."
  fi
  if [[ -z "${OUTPUT//[[:space:]]/}" ]]; then
    ITERATION_FAILED=1
    echo "Runner '$RUNNER' produced no meaningful output."
  fi

  if [ "$ITERATION_FAILED" -eq 1 ]; then
    CONSECUTIVE_FAILURES=$((CONSECUTIVE_FAILURES + 1))
    echo "Consecutive failures: $CONSECUTIVE_FAILURES/3"
    if [ "$CONSECUTIVE_FAILURES" -ge 3 ]; then
      echo ""
      echo "Ralph stopped after 3 consecutive runner failures."
      send_discord_notification "Runner Failure" "Stopped after 3 consecutive runner failures on '$RUNNER'." "15158332"
      exit 1
    fi
  else
    CONSECUTIVE_FAILURES=0
  fi

  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
send_discord_notification "Max Iterations" "Reached $MAX_ITERATIONS iterations without completing all tasks. Manual review needed." "15158332"
exit 1
