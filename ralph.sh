#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude|codex|copilot] [max_iterations]

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
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" && "$TOOL" != "codex" && "$TOOL" != "copilot" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp', 'claude', 'codex', or 'copilot'."
  exit 1
fi

# Refuse shell metacharacters in override strings; we execute commands without eval.
validate_command_string() {
  local command_string="$1"
  local forbidden_chars=(";" "|" "&" "<" ">" "(" ")" "{" "}" '$')
  local char

  for char in "${forbidden_chars[@]}"; do
    if [[ "$command_string" == *"$char"* ]]; then
      echo "Error: Command contains unsafe shell metacharacters: $command_string"
      return 1
    fi
  done
}


# Parse command string into argv array without eval and verify expected binary.
run_with_prompt() {
  local prompt_file="$1"
  local command_string="$2"
  local expected_binary="$3"
  local -a command_parts

  if [[ ! -f "$prompt_file" ]]; then
    echo "Error: Prompt file not found: $prompt_file"
    return 1
  fi

  validate_command_string "$command_string" || return 1

  # shellcheck disable=SC2206
  command_parts=($command_string)

  if [[ ${#command_parts[@]} -eq 0 ]]; then
    echo "Error: Empty command provided for $expected_binary"
    return 1
  fi

  local binary_name
  binary_name="$(basename "${command_parts[0]}")"
  if [[ "$binary_name" != "$expected_binary" ]]; then
    echo "Error: $expected_binary runner must start with '$expected_binary'. Got: ${command_parts[0]}"
    return 1
  fi

  if ! command -v "${command_parts[0]}" >/dev/null 2>&1; then
    echo "Error: Command not found: ${command_parts[0]}"
    return 1
  fi

  "${command_parts[@]}" < "$prompt_file"
}

execute_runner() {
  local prompt_file="$1"
  local command_string="$2"
  local expected_binary="$3"

  RUN_EXIT_CODE=0
  OUTPUT="$(run_with_prompt "$prompt_file" "$command_string" "$expected_binary" 2>&1)" || RUN_EXIT_CODE=$?
  printf '%s\n' "$OUTPUT" >&2
}

has_completion_signal() {
  local output="$1"
  # Only accept an exact completion tag on its own line.
  grep -Eq '^<promise>COMPLETE</promise>$' <<< "$output"
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

  # Run the selected tool with the corresponding Ralph prompt.
  if [[ "$TOOL" == "amp" ]]; then
    AMP_CMD="${RALPH_AMP_CMD:-amp --dangerously-allow-all}"
    execute_runner "$SCRIPT_DIR/prompt.md" "$AMP_CMD" "amp"
  elif [[ "$TOOL" == "claude" ]]; then
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output.
    CLAUDE_CMD="${RALPH_CLAUDE_CMD:-claude --dangerously-skip-permissions --print}"
    execute_runner "$SCRIPT_DIR/CLAUDE.md" "$CLAUDE_CMD" "claude"
  elif [[ "$TOOL" == "codex" ]]; then
    # Override with RALPH_CODEX_CMD if your local Codex command differs.
    CODEX_CMD="${RALPH_CODEX_CMD:-codex exec --full-auto}"
    execute_runner "$SCRIPT_DIR/CODEX.md" "$CODEX_CMD" "codex"
  else
    # GitHub Copilot runner command is environment-specific; override with RALPH_COPILOT_CMD as needed.
    COPILOT_CMD="${RALPH_COPILOT_CMD:-gh copilot agent run}"
    execute_runner "$SCRIPT_DIR/COPILOT.md" "$COPILOT_CMD" "gh"
  fi

  # Check for completion signal from a successful run.
  if [[ $RUN_EXIT_CODE -eq 0 ]] && has_completion_signal "$OUTPUT"; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi

  if [[ $RUN_EXIT_CODE -ne 0 ]]; then
    echo "Runner command failed with exit code $RUN_EXIT_CODE. Continuing..."
  else
    echo "Iteration $i complete. Continuing..."
  fi
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
