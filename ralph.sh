#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [max_iterations]

set -e

MAX_ITERATIONS=${1:-10}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"
ENGINE="${RALPH_ENGINE:-codex}"

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

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "═══════════════════════════════════════════════════════"
  echo "  Ralph Iteration $i of $MAX_ITERATIONS"
  echo "═══════════════════════════════════════════════════════"

  if [ "$ENGINE" = "codex" ]; then
    LAST_MESSAGE_FILE="$SCRIPT_DIR/.last-message"
    rm -f "$LAST_MESSAGE_FILE"

    CODEX_ARGS=()
    if [ "${RALPH_CODEX_FULL_AUTO:-}" = "1" ]; then
      CODEX_ARGS+=(--full-auto)
      if [ -n "${RALPH_CODEX_SANDBOX:-}" ]; then
        CODEX_ARGS+=(--sandbox "$RALPH_CODEX_SANDBOX")
      fi
    else
      CODEX_ARGS+=(--sandbox "${RALPH_CODEX_SANDBOX:-read-only}")
    fi

    if [ -n "${RALPH_CODEX_MODEL:-}" ]; then
      CODEX_ARGS+=(--model "$RALPH_CODEX_MODEL")
    fi

    if [ -n "${RALPH_CODEX_PROFILE:-}" ]; then
      CODEX_ARGS+=(--profile "$RALPH_CODEX_PROFILE")
    fi

    if [ -n "${RALPH_CODEX_ADD_DIR:-}" ]; then
      CODEX_ARGS+=(--add-dir "$RALPH_CODEX_ADD_DIR")
    fi

    if [ -n "${RALPH_CODEX_ARGS:-}" ]; then
      # shellcheck disable=SC2206
      CODEX_ARGS+=(${RALPH_CODEX_ARGS})
    fi

    cat "$SCRIPT_DIR/prompt.md" | codex exec "${CODEX_ARGS[@]}" --output-last-message "$LAST_MESSAGE_FILE" - || true
    OUTPUT=$(cat "$LAST_MESSAGE_FILE" 2>/dev/null || echo "")
  elif [ "$ENGINE" = "amp" ]; then
    AMP_ARGS=${RALPH_AMP_ARGS:---dangerously-allow-all}
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp $AMP_ARGS 2>&1 | tee /dev/stderr) || true
  else
    echo "Unknown RALPH_ENGINE: $ENGINE (expected 'codex' or 'amp')"
    exit 1
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
