#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude] [--debug] [max_sessions]

set -e

# Parse arguments
TOOL="amp"
MAX_SESSIONS=10
DEBUG=false

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
    --debug)
      DEBUG=true
      shift
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_SESSIONS="$1"
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
PROGRESS_FILE="$SCRIPT_DIR/progress.md"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
STATE_FILE="$SCRIPT_DIR/state.json"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"

# Validate required files exist
if [ ! -f "$PRD_FILE" ]; then
  echo "Error: PRD file not found at $PRD_FILE"
  echo "Create a prd.json first (see prd.json.example)."
  exit 1
fi

# Validate PRD is valid JSON
if ! jq empty "$PRD_FILE" 2>/dev/null; then
  echo "Error: $PRD_FILE is not valid JSON."
  exit 1
fi

# Validate required PRD fields
if ! jq -e '.project and .branchName and .userStories and (.userStories | length > 0)' "$PRD_FILE" > /dev/null 2>&1; then
  echo "Error: $PRD_FILE is missing required fields (project, branchName, userStories)."
  exit 1
fi

# Archive previous run if branch changed
if [ -f "$LAST_BRANCH_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")

  if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
    DATE=$(date +%Y-%m-%d)
    FOLDER_NAME=$(echo "$LAST_BRANCH" | sed 's|^ralph/||')
    ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"

    echo "Archiving previous run: $LAST_BRANCH"
    mkdir -p "$ARCHIVE_FOLDER"
    [ -f "$PRD_FILE" ] && cp "$PRD_FILE" "$ARCHIVE_FOLDER/"
    [ -f "$PROGRESS_FILE" ] && cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/"
    [ -f "$STATE_FILE" ] && cp "$STATE_FILE" "$ARCHIVE_FOLDER/"
    echo "   Archived to: $ARCHIVE_FOLDER"

    # Reset progress file for new run
    echo "# Ralph Progress Log" > "$PROGRESS_FILE"
    echo "Started: $(date)" >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"
    echo "" >> "$PROGRESS_FILE"
    echo "## Codebase Patterns" >> "$PROGRESS_FILE"
    echo "> Reusable patterns discovered during implementation. Read this FIRST every session." >> "$PROGRESS_FILE"
    # Reset state.json for new run (will be re-bootstrapped)
    rm -f "$STATE_FILE"
  fi
fi

# Track current branch
CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
if [ -n "$CURRENT_BRANCH" ]; then
  echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"
fi

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
  echo "" >> "$PROGRESS_FILE"
  echo "## Codebase Patterns" >> "$PROGRESS_FILE"
  echo "> Reusable patterns discovered during implementation. Read this FIRST every session." >> "$PROGRESS_FILE"
fi

# Bootstrap state.json if it doesn't exist
if [ ! -f "$STATE_FILE" ]; then
  echo "Bootstrapping state.json from prd.json..."
  FIRST_STORY_ID=$(jq -r '[.userStories[] | select(.passes == false)] | sort_by(.priority) | first | .id // empty' "$PRD_FILE")
  FIRST_STORY_TITLE=$(jq -r '[.userStories[] | select(.passes == false)] | sort_by(.priority) | first | .title // empty' "$PRD_FILE")
  PRD_BRANCH=$(jq -r '.branchName // ""' "$PRD_FILE")

  if [ -n "$FIRST_STORY_ID" ]; then
    ITER_MODE=$(jq -r --arg id "$FIRST_STORY_ID" '[.userStories[] | select(.id == $id)] | first | .iterationMode // empty' "$PRD_FILE")
    MAX_ITER=$(jq -r --arg id "$FIRST_STORY_ID" '[.userStories[] | select(.id == $id)] | first | .maxIterations // 0' "$PRD_FILE")

    if [ -n "$ITER_MODE" ] && [ "$ITER_MODE" != "null" ]; then
      NEXT_ACTION="iteration 1/$MAX_ITER"
    else
      NEXT_ACTION="implement"
    fi

    jq -n \
      --arg branch "$PRD_BRANCH" \
      --arg story "$FIRST_STORY_ID" \
      --arg title "$FIRST_STORY_TITLE" \
      --arg action "$NEXT_ACTION" \
      --arg date "$(date +%Y-%m-%d)" \
      '{
        branch: $branch,
        currentStory: $story,
        storyTitle: $title,
        nextAction: $action,
        progressTokens: 0,
        compactionNeeded: false,
        compactionThreshold: 10000,
        lastUpdated: $date
      }' > "$STATE_FILE"
    echo "   Created state.json: $FIRST_STORY_ID - $FIRST_STORY_TITLE"
  fi
fi

echo "Starting Ralph - Tool: $TOOL - Max sessions: $MAX_SESSIONS - Debug: $DEBUG"

for i in $(seq 1 $MAX_SESSIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Session $i of $MAX_SESSIONS ($TOOL)"
  echo "==============================================================="

  # Run the selected tool with the ralph prompt
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  else
    # Claude Code flags:
    #   Normal: --print only (minimal, proven stable)
    #   Debug:  full SDK stack (stream-json requires --verbose,
    #           --include-partial-messages requires stream-json)
    CLAUDE_ARGS=(
      --dangerously-skip-permissions
      --print
    )
    if [[ "$DEBUG" == "true" ]]; then
      CLAUDE_ARGS+=(--output-format stream-json --verbose --include-partial-messages)
    fi

    OUTPUT=$(
      claude "${CLAUDE_ARGS[@]}" < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr
    ) || true
  fi

  # Check for completion by reading prd.json directly (ground truth)
  # Immune to false positives from agent text output
  ALL_PASS=$(jq '[.userStories[].passes] | all' "$PRD_FILE" 2>/dev/null || echo "false")
  if [ "$ALL_PASS" = "true" ]; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at session $i of $MAX_SESSIONS"
    exit 0
  fi

  # Validate state.json integrity after session
  if [ -f "$STATE_FILE" ]; then
    if ! jq -e '.currentStory and .lastUpdated and .branch' "$STATE_FILE" > /dev/null 2>&1; then
      echo "Warning: state.json may be malformed after session $i. Check manually."
    fi
  fi

  echo "Session $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max sessions ($MAX_SESSIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1