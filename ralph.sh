#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--help] [--doctor] [--status] [--tool claude|codex] [max_iterations]

set -e

SCRIPT_NAME="$(basename "$0")"
DEFAULT_TOOL="codex"
DEFAULT_MAX_ITERATIONS=10
DOCTOR_MODE=0
STATUS_MODE=0
REPO_ENTRYPOINT="./ralph.sh"

print_help() {
  cat <<EOF
Usage:
  ./$SCRIPT_NAME [--help] [--doctor] [--status] [--tool codex|claude] [max_iterations]

Runs Ralph's one-story-per-iteration loop for the current repository.

Options:
  --help, -h             Print this help and exit
  --doctor               Run non-destructive preflight checks and exit
  --status               Summarize Ralph progress and exit
  --tool TOOL            Agent runtime to use: codex (default) or claude
  --tool=TOOL            Same as above
  max_iterations         Positive integer iteration cap (default: $DEFAULT_MAX_ITERATIONS)

Defaults:
  tool: $DEFAULT_TOOL
  max_iterations: $DEFAULT_MAX_ITERATIONS

Codex-first happy path:
  1. Open this repo in Codex.
  2. Optionally run /skills to confirm the repo-local skills are available.
  3. Use \$prd to create tasks/prd-<feature>.md.
  4. Use \$ralph to convert that PRD into ./prd.json.
  5. Run ./$SCRIPT_NAME.

Ralph state files in this repo:
  prd.json
  progress.txt
  archive/

Examples:
  ./$SCRIPT_NAME --doctor
  ./$SCRIPT_NAME --status
  ./$SCRIPT_NAME
  ./$SCRIPT_NAME 20
  ./$SCRIPT_NAME --tool claude 5

Global command:
  Install bin/ralph onto your PATH once, then run:
  ralph --status
  ralph --tool claude 5
  See README.md for the install and uninstall steps.
EOF
}

print_usage_error() {
  echo "Error: $1" >&2
  echo "Run $REPO_ENTRYPOINT --help for usage." >&2
  exit 1
}

# Parse arguments
TOOL="$DEFAULT_TOOL"
MAX_ITERATIONS="$DEFAULT_MAX_ITERATIONS"
POSITIONAL_MAX_ITERATIONS_SEEN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h)
      print_help
      exit 0
      ;;
    --doctor)
      DOCTOR_MODE=1
      shift
      ;;
    --status)
      STATUS_MODE=1
      shift
      ;;
    --tool)
      if [[ $# -lt 2 ]] || [[ "$2" == -* ]]; then
        print_usage_error "Missing value for --tool."
      fi
      TOOL="$2"
      shift 2
      ;;
    --tool=*)
      TOOL="${1#*=}"
      if [[ -z "$TOOL" ]]; then
        print_usage_error "Missing value for --tool."
      fi
      shift
      ;;
    -*)
      print_usage_error "Unknown flag '$1'."
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        if [[ "$POSITIONAL_MAX_ITERATIONS_SEEN" -eq 1 ]]; then
          print_usage_error "Only one max_iterations value is supported."
        fi
        POSITIONAL_MAX_ITERATIONS_SEEN=1
        MAX_ITERATIONS="$1"
      else
        print_usage_error "Unexpected argument '$1'. max_iterations must be a positive integer."
      fi
      shift
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || echo "$SCRIPT_DIR")"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"
CODEX_PROMPT_FILE="$SCRIPT_DIR/CODEX.md"
CLAUDE_PROMPT_FILE="$SCRIPT_DIR/CLAUDE.md"
PRD_SKILL_FILE="$SCRIPT_DIR/.agents/skills/prd/SKILL.md"
RALPH_SKILL_FILE="$SCRIPT_DIR/.agents/skills/ralph/SKILL.md"

case "$TOOL" in
  claude)
    TOOL_NAME="Claude Code"
    TOOL_BIN="claude"
    PROMPT_FILE="$CLAUDE_PROMPT_FILE"
    TOOL_CMD=(claude --dangerously-skip-permissions --print)
    ;;
  codex)
    TOOL_NAME="Codex"
    TOOL_BIN="codex"
    PROMPT_FILE="$CODEX_PROMPT_FILE"
    TOOL_CMD=(codex exec --full-auto --color never)
    ;;
  *)
    print_usage_error "Invalid tool '$TOOL'. Use 'codex' or 'claude'."
    ;;
esac

print_repo_asset_expectations() {
  cat <<EOF
Ralph expects these repo files under $SCRIPT_DIR:
- $CODEX_PROMPT_FILE
- $CLAUDE_PROMPT_FILE
- $PRD_SKILL_FILE
- $RALPH_SKILL_FILE
EOF
}

print_codex_first_recovery_path() {
  cat <<EOF
Codex-first recovery path:
1. Optionally run \`/skills\` to confirm the repo-local skills are available.
2. Use \`\$prd\` to draft a PRD into \`tasks/prd-<feature>.md\`.
3. Use \`\$ralph\` to convert that PRD into \`$PRD_FILE\`.
4. Run \`$REPO_ENTRYPOINT\`.
EOF
}

print_missing_runtime_binary_help() {
  echo "Error: Required runtime binary '$TOOL_BIN' is not installed or not on PATH."

  if [ "$TOOL" = "codex" ]; then
    echo "Next: Install 'codex', then rerun $REPO_ENTRYPOINT or $REPO_ENTRYPOINT --doctor."
    echo "Alternative: If you want Claude Code instead, install 'claude' and run $REPO_ENTRYPOINT --tool claude."
    return
  fi

  echo "Next: Install 'claude', then rerun $REPO_ENTRYPOINT --tool claude or $REPO_ENTRYPOINT --doctor --tool claude."
  echo "Alternative: If you want Codex instead, install 'codex' and run $REPO_ENTRYPOINT."
}

print_missing_jq_help() {
  echo "Error: Required binary 'jq' is not installed or not on PATH."
  echo "Next: Install 'jq', then rerun $REPO_ENTRYPOINT --doctor so Ralph can validate $PRD_FILE."
}

print_missing_repo_asset_help() {
  local asset_label="$1"
  local asset_path="$2"
  local next_command="$3"

  echo "Error: Missing $asset_label: $asset_path"
  echo ""
  print_repo_asset_expectations
  echo ""
  echo "Next: Restore $asset_path, then rerun:"
  echo "  $next_command"
}

print_invalid_prd_help() {
  local reason="$1"
  local next_action="$2"

  echo "Error: $reason"
  echo "Next: $next_action"
  echo ""
  print_codex_first_recovery_path
}

print_missing_prd_help() {
  cat <<EOF
Error: Missing Ralph state file: $PRD_FILE

$(print_codex_first_recovery_path)

If you copied Ralph into another repository, also restore these repo assets:
$(print_repo_asset_expectations)
EOF
}

print_status_header() {
  echo "Ralph status"
  echo "Project root: $PROJECT_ROOT"
  echo "Ralph directory: $SCRIPT_DIR"
  echo "PRD file: $PRD_FILE"
  echo "Progress log: $PROGRESS_FILE"
  echo "Archive directory: $ARCHIVE_DIR"
  echo ""
}

status_unavailable() {
  local reason="$1"
  local next_action="$2"

  print_status_header
  echo "Status: unavailable"
  echo "Reason: $reason"
  echo "Next: $next_action"
  return 1
}

run_status() {
  local summary=""
  local branch_name=""
  local total_stories=""
  local passed_stories=""
  local pending_stories=""
  local next_story_id=""
  local next_story_title=""

  if ! command -v jq >/dev/null 2>&1; then
    status_unavailable "'jq' is required to read $PRD_FILE." "Install jq, then rerun $REPO_ENTRYPOINT --status."
    return 1
  fi

  if [ ! -f "$PRD_FILE" ]; then
    status_unavailable "Missing Ralph state file: $PRD_FILE" "In Codex, run /skills, then \$prd to draft a PRD, then \$ralph to create $PRD_FILE, then rerun $REPO_ENTRYPOINT --status."
    return 1
  fi

  if ! jq -e '.' "$PRD_FILE" >/dev/null 2>&1; then
    status_unavailable "$PRD_FILE is not valid JSON." "Fix the JSON in $PRD_FILE or rerun \$ralph to regenerate it from your PRD markdown, then rerun $REPO_ENTRYPOINT --status."
    return 1
  fi

  if ! jq -e '.branchName | type == "string" and length > 0' "$PRD_FILE" >/dev/null 2>&1; then
    status_unavailable "$PRD_FILE is missing a non-empty 'branchName' field." "Edit $PRD_FILE so 'branchName' is a non-empty string, or rerun \$ralph to regenerate it, then rerun $REPO_ENTRYPOINT --status."
    return 1
  fi

  if ! jq -e '.userStories | type == "array" and length > 0' "$PRD_FILE" >/dev/null 2>&1; then
    status_unavailable "$PRD_FILE must contain a non-empty 'userStories' array." "Update $PRD_FILE with at least one story, or rerun \$ralph to regenerate it, then rerun $REPO_ENTRYPOINT --status."
    return 1
  fi

  summary="$(
    jq -r '
      {
        branchName: .branchName,
        total: (.userStories | length),
        passed: ([.userStories[] | select(.passes == true)] | length),
        pending: ([.userStories[] | select(.passes != true)] | length),
        next: (
          [
            .userStories[]
            | select(.passes != true)
            | {
                id: ((.id // "unknown") | tostring),
                title: ((.title // "Untitled") | tostring),
                priority: (.priority // 999999)
              }
          ]
          | sort_by(.priority, .id)
          | .[0] // {}
        )
      }
      | [
          .branchName,
          (.total | tostring),
          (.passed | tostring),
          (.pending | tostring),
          (.next.id // ""),
          (.next.title // "")
        ]
      | @tsv
    ' "$PRD_FILE"
  )"

  IFS=$'\t' read -r branch_name total_stories passed_stories pending_stories next_story_id next_story_title <<< "$summary"

  print_status_header
  echo "Branch name: $branch_name"
  echo "Stories total: $total_stories"
  echo "Stories passed: $passed_stories"
  echo "Stories pending: $pending_stories"

  if [ "$pending_stories" -eq 0 ]; then
    echo "Next pending story: none - all stories pass"
  else
    echo "Next pending story: $next_story_id - $next_story_title"
  fi
}

DOCTOR_FAILURES=0

doctor_pass() {
  echo "[PASS] $1 - $2"
}

doctor_fail() {
  echo "[FAIL] $1 - $2"
  echo "       Next: $3"
  DOCTOR_FAILURES=$((DOCTOR_FAILURES + 1))
}

run_doctor_binary_check() {
  local label="$1"
  local binary_name="$2"
  local next_action="$3"
  local binary_path=""

  if binary_path="$(command -v "$binary_name" 2>/dev/null)"; then
    doctor_pass "$label" "found '$binary_name' at $binary_path"
  else
    doctor_fail "$label" "'$binary_name' is not installed or not on PATH." "$next_action"
  fi
}

run_doctor_file_check() {
  local label="$1"
  local file_path="$2"
  local next_action="$3"

  if [ -f "$file_path" ]; then
    doctor_pass "$label" "found $file_path"
  else
    doctor_fail "$label" "missing $file_path" "$next_action"
  fi
}

run_doctor_progress_check() {
  if [ -f "$PROGRESS_FILE" ]; then
    if [ -w "$PROGRESS_FILE" ]; then
      doctor_pass "State: progress.txt" "found writable $PROGRESS_FILE"
      return
    fi

    doctor_fail "State: progress.txt" "$PROGRESS_FILE exists but is not writable." "Restore write access to $PROGRESS_FILE before running $REPO_ENTRYPOINT."
    return
  fi

  if [ -w "$SCRIPT_DIR" ]; then
    doctor_pass "State: progress.txt" "$PROGRESS_FILE will be created on the first Ralph run."
  else
    doctor_fail "State: progress.txt" "missing $PROGRESS_FILE and $SCRIPT_DIR is not writable." "Create $PROGRESS_FILE manually or restore write access to $SCRIPT_DIR before running $REPO_ENTRYPOINT."
  fi
}

run_doctor_archive_check() {
  if [ -d "$ARCHIVE_DIR" ]; then
    if [ -w "$ARCHIVE_DIR" ]; then
      doctor_pass "State: archive/" "found writable $ARCHIVE_DIR"
      return
    fi

    doctor_fail "State: archive/" "$ARCHIVE_DIR exists but is not writable." "Restore write access to $ARCHIVE_DIR before running $REPO_ENTRYPOINT."
    return
  fi

  if [ -w "$SCRIPT_DIR" ]; then
    doctor_pass "State: archive/" "$ARCHIVE_DIR will be created automatically when Ralph archives a previous run."
  else
    doctor_fail "State: archive/" "missing $ARCHIVE_DIR and $SCRIPT_DIR is not writable." "Create $ARCHIVE_DIR manually or restore write access to $SCRIPT_DIR before running $REPO_ENTRYPOINT."
  fi
}

run_doctor_prd_check() {
  if [ ! -f "$PRD_FILE" ]; then
    doctor_fail "State: prd.json" "missing $PRD_FILE" "Run /skills, then \$prd, then \$ralph to create $PRD_FILE before running $REPO_ENTRYPOINT."
    return
  fi

  if ! command -v jq >/dev/null 2>&1; then
    doctor_fail "State: prd.json" "found $PRD_FILE, but Ralph cannot validate it without 'jq'." "Install jq, then rerun $REPO_ENTRYPOINT --doctor so Ralph can validate $PRD_FILE."
    return
  fi

  if ! jq -e '.' "$PRD_FILE" >/dev/null 2>&1; then
    doctor_fail "State: prd.json" "$PRD_FILE is not valid JSON." "Fix the JSON in $PRD_FILE or rerun \$ralph to regenerate it from your PRD markdown, then rerun $REPO_ENTRYPOINT --doctor."
    return
  fi

  if ! jq -e '.branchName | type == "string" and length > 0' "$PRD_FILE" >/dev/null 2>&1; then
    doctor_fail "State: prd.json" "$PRD_FILE is missing a non-empty 'branchName' field." "Update $PRD_FILE so 'branchName' is a non-empty string, then rerun $REPO_ENTRYPOINT --doctor."
    return
  fi

  if ! jq -e '.userStories | type == "array" and length > 0' "$PRD_FILE" >/dev/null 2>&1; then
    doctor_fail "State: prd.json" "$PRD_FILE must contain a non-empty 'userStories' array." "Update $PRD_FILE with at least one story, or rerun \$ralph to regenerate it from your PRD markdown, then rerun $REPO_ENTRYPOINT --doctor."
    return
  fi

  if [ ! -w "$PRD_FILE" ]; then
    doctor_fail "State: prd.json" "$PRD_FILE is valid, but it is not writable." "Restore write access to $PRD_FILE before running $REPO_ENTRYPOINT so Ralph can update story status."
    return
  fi

  doctor_pass "State: prd.json" "validated writable $PRD_FILE"
}

run_doctor() {
  echo "Ralph doctor"
  echo "Project root: $PROJECT_ROOT"
  echo "Ralph directory: $SCRIPT_DIR"
  echo "Selected tool: $TOOL_NAME ($TOOL)"
  echo ""

  run_doctor_binary_check "Binary: jq" "jq" "Install jq and rerun $REPO_ENTRYPOINT --doctor."
  run_doctor_binary_check "Binary: $TOOL_BIN" "$TOOL_BIN" "Install '$TOOL_BIN', or rerun $REPO_ENTRYPOINT --doctor with --tool codex or --tool claude for the runtime you intend to use."
  run_doctor_file_check "Prompt: CODEX.md" "$CODEX_PROMPT_FILE" "Restore $CODEX_PROMPT_FILE under $SCRIPT_DIR so Codex runs have their prompt file."
  run_doctor_file_check "Prompt: CLAUDE.md" "$CLAUDE_PROMPT_FILE" "Restore $CLAUDE_PROMPT_FILE under $SCRIPT_DIR so Claude Code runs have their prompt file."
  run_doctor_file_check "Skill: .agents/skills/prd/SKILL.md" "$PRD_SKILL_FILE" "Restore $PRD_SKILL_FILE under $SCRIPT_DIR so the \$prd workflow is available in this repo."
  run_doctor_file_check "Skill: .agents/skills/ralph/SKILL.md" "$RALPH_SKILL_FILE" "Restore $RALPH_SKILL_FILE under $SCRIPT_DIR so the \$ralph workflow is available in this repo."
  run_doctor_prd_check
  run_doctor_progress_check
  run_doctor_archive_check

  echo ""

  if [ "$DOCTOR_FAILURES" -eq 0 ]; then
    echo "Doctor status: PASS"
    return 0
  fi

  echo "Doctor status: FAIL ($DOCTOR_FAILURES checks failed)"
  return 1
}

validate_prd_file() {
  if [ ! -f "$PRD_FILE" ]; then
    print_missing_prd_help
    exit 1
  fi

  if ! jq -e '.' "$PRD_FILE" >/dev/null 2>&1; then
    print_invalid_prd_help "Invalid JSON in $PRD_FILE." "Fix the JSON in $PRD_FILE or rerun \$ralph to regenerate it from your PRD markdown."
    exit 1
  fi

  if ! jq -e '.branchName | type == "string" and length > 0' "$PRD_FILE" >/dev/null 2>&1; then
    print_invalid_prd_help "$PRD_FILE must contain a non-empty string field 'branchName'." "Edit $PRD_FILE so 'branchName' is a non-empty string, or rerun \$ralph to regenerate it."
    exit 1
  fi

  if ! jq -e '.userStories | type == "array" and length > 0' "$PRD_FILE" >/dev/null 2>&1; then
    print_invalid_prd_help "$PRD_FILE must contain a non-empty 'userStories' array." "Update $PRD_FILE with at least one story, or rerun \$ralph to regenerate it from your PRD markdown."
    exit 1
  fi
}

get_pending_story_count() {
  jq -r '
    if (.userStories | type) != "array" then
      error("userStories must be an array")
    else
      [.userStories[] | select(.passes != true)] | length
    end
  ' "$PRD_FILE"
}

build_runtime_context() {
  cat <<EOF
# Ralph Runtime Context

Use these exact paths when reading and writing Ralph state:
- Project root: $PROJECT_ROOT
- Ralph directory: $SCRIPT_DIR
- PRD file: $PRD_FILE
- Progress log: $PROGRESS_FILE
- Archive directory: $ARCHIVE_DIR

EOF
}

if [ "$DOCTOR_MODE" -eq 1 ]; then
  run_doctor
  exit $?
fi

if [ "$STATUS_MODE" -eq 1 ]; then
  run_status
  exit $?
fi

if ! command -v "$TOOL_BIN" >/dev/null 2>&1; then
  print_missing_runtime_binary_help
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  print_missing_jq_help
  exit 1
fi

if [ ! -f "$PROMPT_FILE" ]; then
  print_missing_repo_asset_help "prompt file" "$PROMPT_FILE" "$REPO_ENTRYPOINT --doctor"
  exit 1
fi

validate_prd_file

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

cd "$PROJECT_ROOT"

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

echo "Starting Ralph - Tool: $TOOL_NAME ($TOOL) - Max iterations: $MAX_ITERATIONS"
echo "Project root: $PROJECT_ROOT"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL_NAME)"
  echo "==============================================================="

  # Run the selected tool with the ralph prompt
  OUTPUT=$(
    {
      build_runtime_context
      cat "$PROMPT_FILE"
    } | "${TOOL_CMD[@]}" 2>&1 | tee /dev/stderr
  ) || true

  PENDING_STORIES="$(get_pending_story_count 2>/dev/null)" || {
    echo "Error: Could not determine remaining story count from $PRD_FILE after iteration $i."
    echo "Check $PRD_FILE for invalid JSON or a malformed userStories array."
    exit 1
  }

  if [ "$PENDING_STORIES" -eq 0 ]; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi

  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo "Warning: Agent reported completion, but $PENDING_STORIES stories are still pending in $PRD_FILE."
    echo "Continuing because Ralph state is not fully complete yet."
  fi
  
  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
