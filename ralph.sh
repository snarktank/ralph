#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude|codex] [max_iterations]

set -e

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
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" && "$TOOL" != "codex" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp', 'claude', or 'codex'."
  exit 1
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"
CODEX_PROMPT_FILE="${CODEX_PROMPT_FILE:-$SCRIPT_DIR/CODEX.md}"
CODEX_CMD="${CODEX_CMD:-codex exec --full-auto}"
CODEX_INPUT="${CODEX_INPUT:-stdin}"
CODEX_LAST_MESSAGE_FILE="${CODEX_LAST_MESSAGE_FILE:-$SCRIPT_DIR/.codex-last-message}"

if [[ "$TOOL" == "codex" && "$CODEX_INPUT" != "stdin" && "$CODEX_INPUT" != "file" ]]; then
  echo "Error: Invalid CODEX_INPUT '$CODEX_INPUT'. Must be 'stdin' or 'file'."
  exit 1
fi

read -r -a CODEX_CMD_ARR <<< "$CODEX_CMD"

if [[ "$TOOL" == "codex" && "$CODEX_INPUT" == "file" && "${CODEX_CMD_ARR[0]}" == "codex" && "${CODEX_CMD_ARR[1]}" == "exec" ]]; then
  echo "Error: CODEX_INPUT=file is not supported with 'codex exec'. Use stdin (default) or wrap Codex to read files."
  exit 1
fi

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
  elif [[ "$TOOL" == "claude" ]]; then
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  else
    # Codex: use non-interactive `codex exec` with stdin by default; override CODEX_CMD for custom flags.
    CODEX_CMD_RUN=("${CODEX_CMD_ARR[@]}")
    if [[ "${CODEX_CMD_ARR[0]}" == "codex" && "${CODEX_CMD_ARR[1]}" == "exec" ]]; then
      CODEX_CMD_RUN+=("--output-last-message" "$CODEX_LAST_MESSAGE_FILE")
    fi

    : > "$CODEX_LAST_MESSAGE_FILE"
    if [[ "$CODEX_INPUT" == "file" ]]; then
      OUTPUT=$("${CODEX_CMD_RUN[@]}" "$CODEX_PROMPT_FILE" 2>&1 | tee /dev/stderr) || true
    else
      OUTPUT=$(cat "$CODEX_PROMPT_FILE" | "${CODEX_CMD_RUN[@]}" 2>&1 | tee /dev/stderr) || true
    fi
  fi
  
  # Check for completion signal
  if [[ "$TOOL" == "codex" && -s "$CODEX_LAST_MESSAGE_FILE" ]]; then
    COMPLETE_SIGNAL=$(grep -q "<promise>COMPLETE</promise>" "$CODEX_LAST_MESSAGE_FILE" && echo "yes" || echo "no")
  else
    COMPLETE_SIGNAL=$(echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>" && echo "yes" || echo "no")
  fi

  if [[ "$COMPLETE_SIGNAL" == "yes" ]]; then
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
