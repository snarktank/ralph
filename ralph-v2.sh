#!/bin/bash

# Ralph Wiggum v2.0 — Autonomous AI Agent Loop
# Phases 0-3: Bugfixes, Memory, Error Recovery, Harness Engineering
#
# Usage: ./ralph.sh [--llm-provider nvidia] [--max-iterations N] [--debug]

set -e

# ═══════════════════════════════════════════════════════════════
# SECTION 1: Dependency Checks
# ═══════════════════════════════════════════════════════════════

if ! command -v jq &> /dev/null; then
  echo "Error: jq is required. Install: sudo apt install jq"
  exit 1
fi
if ! command -v curl &> /dev/null; then
  echo "Error: curl is required. Install: sudo apt install curl"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 2: Argument Parsing
# ═══════════════════════════════════════════════════════════════

TOOL="amp"
MAX_ITERATIONS=10
LLM_PROVIDER=""
LLM_API_KEY=""
LLM_MODEL=""
LLM_BASE_URL=""
LLM_TEMPERATURE="0.7"

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
    --llm-provider)
      LLM_PROVIDER="$2"
      shift 2
      ;;
    --llm-api-key)
      LLM_API_KEY="$2"
      shift 2
      ;;
    --llm-model)
      LLM_MODEL="$2"
      shift 2
      ;;
    --llm-base-url)
      LLM_BASE_URL="$2"
      shift 2
      ;;
    --llm-temperature)
      LLM_TEMPERATURE="$2"
      shift 2
      ;;
    --max-iterations)
      MAX_ITERATIONS="$2"
      shift 2
      ;;
    --max-iterations=*)
      MAX_ITERATIONS="${1#*=}"
      shift
      ;;
    --debug)
      export RALPH_DEBUG=1
      shift
      ;;
    --version|-v)
      echo "Ralph v2.0"
      exit 0
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      fi
      shift
      ;;
  esac
done

# ═══════════════════════════════════════════════════════════════
# SECTION 3: Environment Loading
# ═══════════════════════════════════════════════════════════════

if [ -f ".env" ]; then
  # shellcheck disable=SC1091
  . .env
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 4: NVIDIA Provider Setup + System Prompt (Phase 3a)
# ═══════════════════════════════════════════════════════════════

if [[ -n "$LLM_PROVIDER" ]]; then
  if [[ "$LLM_PROVIDER" != "nvidia" ]]; then
    echo "Error: Only '--llm-provider nvidia' is currently supported."
    exit 1
  fi

  if [[ -z "$LLM_API_KEY" ]]; then
    LLM_API_KEY="${NVIDIA_API_KEY:-${LLM_API_KEY}}"
  fi
  if [[ -z "$LLM_API_KEY" ]]; then
    echo "Error: --llm-api-key or NVIDIA_API_KEY required for nvidia provider"
    exit 1
  fi

  LLM_BASE_URL="${LLM_BASE_URL:-https://integrate.api.nvidia.com/v1}"
  LLM_MODEL="${LLM_MODEL:-moonshotai/kimi-k2-instruct}"

  # Phase 3a: Enhanced system prompt with planning, few-shot, guardrails
  LLM_SYSTEM_PROMPT=$(cat << 'SYSPROMPT'
You are an autonomous coding agent implementing user stories from a PRD.
You MUST respond with ONLY a valid JSON object — no text before or after.

RESPONSE FORMAT:
{
  "thoughts": "your reasoning about what to do next",
  "actions": [action1, action2, ...]
}

ACTION TYPES:

1. plan (ALWAYS use this first before implementing):
   { "type": "plan", "todos": "1. Step one\n2. Step two\n3. Step three" }
   This records your plan. Always plan before coding.

2. read_file:
   { "type": "read_file", "path": "file.ext" }
   Optional: "line_start": N, "line_end": M for specific ranges.
   Files >200 lines are auto-chunked (first 50 + last 50).

3. list_files:
   { "type": "list_files", "path": "directory/" }

4. edit_file:
   { "type": "edit_file", "path": "file.ext", "changes": [
     { "oldString": "exact text to find", "newString": "replacement text" }
   ]}
   Make SMALL changes. One logical change per edit. Verify after editing.

5. create_file:
   { "type": "create_file", "path": "file.ext", "content": "full file content" }

6. run_command:
   { "type": "run_command", "command": "shell command here" }

7. finish:
   { "type": "finish" }
   Use when the current story is fully implemented and verified.
   This triggers automatic verification if verificationCommand exists.

EXAMPLE (good response):
{
  "thoughts": "I need to create a stats parser. Let me plan first, then read existing code.",
  "actions": [
    {"type": "plan", "todos": "1. Read prd.json for requirements\n2. Create parse_stats.sh\n3. Test output\n4. Finish"},
    {"type": "read_file", "path": "prd.json"}
  ]
}

WORKFLOW:
1. Read prd.json to find first story where passes is false
2. Read progress.txt, learnings.md, AGENTS.md for prior context
3. Use plan action to outline your approach BEFORE implementing
4. Implement with minimal, focused changes
5. Verify your changes (read files back, run tests)
6. Use finish action — this triggers verificationCommand if defined
7. Append summary to progress.txt
8. Update AGENTS.md with discovered patterns or gotchas

MEMORY FILES:
- progress.txt: Progress log. Append a summary after each story.
- learnings.md: Append-only insights log. Record what worked and what failed.
- AGENTS.md: Knowledge base. Record patterns, gotchas, conventions for future iterations.

PROTECTED FILES (do NOT directly edit):
- .env, .git/*, .gitignore — never modify these
- prd.json — only modified automatically by finish action (passes field)

SELF-MODIFICATION (ralph.sh):
- Your edits go to a staging copy, validated with bash -n before applying
- ALWAYS commit before editing: run_command "git add -A && git commit -m checkpoint"
- Make SMALL incremental changes — one edit_file per logical change
- After editing, read_file to verify your staged changes

GIT RULES (MANDATORY):
- Before .sh edits: run_command "git add -A && git commit -m checkpoint-before-edit"
- BEFORE calling finish: you MUST run_command "git add -A && git commit -m feat-STORYID-description"
- NEVER call finish without committing first. The commit proves your work is saved.
- Use run_command for all git operations
SYSPROMPT
  )

  TOOL="llm_direct_nvidia"
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 5: Tool Validation
# ═══════════════════════════════════════════════════════════════

if [[ "$TOOL" != "amp" && "$TOOL" != "claude" && "$TOOL" != "llm_direct_nvidia" ]]; then
  echo "Error: Invalid tool '$TOOL'. Use 'amp', 'claude', or --llm-provider nvidia."
  exit 1
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 6: Path Definitions + Constants (Phase 1, 2)
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/ralph-debug.log"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"

# Phase 1: Memory file paths
LEARNINGS_FILE="$SCRIPT_DIR/learnings.md"
AGENTS_FILE="$SCRIPT_DIR/AGENTS.md"
STEERING_FILE="$SCRIPT_DIR/STEERING.md"

# Global timer variables
SCRIPT_START_SECONDS=$SECONDS
TOTAL_API_CALLS=0
TOTAL_API_TIME=0
ITERATION_START_SECONDS=0
CURRENT_ITERATION=0

# Phase 1: Context monitoring
MAX_CONTEXT_TOKENS=${MAX_CONTEXT_TOKENS:-200000}
CONTEXT_WARN_THRESHOLD=70
CONTEXT_SHRINK_THRESHOLD=80
CONSECUTIVE_API_ERRORS=0

# Phase 2: Error recovery constants
STUCK_THRESHOLD=3       # self-reflection after N same errors
SKIP_THRESHOLD=7        # skip story after N same errors
CONSECUTIVE_SAME_ERRORS=0
LAST_ERROR_HASH=""

# Phase 2: Fallback models
LLM_FALLBACK_MODELS=(
  "abacusai/dracarys-llama-3.1-70b-instruct"
  "microsoft/phi-3-medium-128k-instruct"
  "microsoft/phi-3-mini-128k-instruct"
)
LLM_ORIGINAL_MODEL="$LLM_MODEL"

# Phase 3b: Protected file patterns (hard block)
PROTECTED_FILES_BLOCK=(".env" ".git" ".gitignore")

# ═══════════════════════════════════════════════════════════════
# SECTION 7: Helper Functions
# ═══════════════════════════════════════════════════════════════

# Estimate token count (~4 chars/token heuristic)
estimate_tokens() {
  local text="$1"
  echo $(( ${#text} / 4 ))
}

# Check context usage and warn/shrink
check_context_usage() {
  local history_json="$1"
  local history_text
  history_text=$(echo "$history_json" | jq -r '.[].content' 2>/dev/null)
  local sys_tokens hist_tokens total_tokens usage_pct
  sys_tokens=$(estimate_tokens "$LLM_SYSTEM_PROMPT")
  hist_tokens=$(estimate_tokens "$history_text")
  total_tokens=$(( sys_tokens + hist_tokens ))
  usage_pct=$(( total_tokens * 100 / MAX_CONTEXT_TOKENS ))

  if (( usage_pct >= CONTEXT_SHRINK_THRESHOLD )); then
    echo "CONTEXT: ⚠️ ${usage_pct}% used (~${total_tokens}/${MAX_CONTEXT_TOKENS} tokens) — shrinking" >&2
    echo "shrink"
  elif (( usage_pct >= CONTEXT_WARN_THRESHOLD )); then
    echo "CONTEXT: ⚡ ${usage_pct}% used (~${total_tokens}/${MAX_CONTEXT_TOKENS} tokens)" >&2
    echo "warn"
  else
    echo "ok"
  fi
}

# Log action to ralph-debug.log
log_action() {
  local action_type="$1" action_details="$2" success="$3"
  local timestamp
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "$timestamp - $action_type - $action_details - $success" >> "$LOG_FILE"
}

# Debug log — only prints when RALPH_DEBUG=1
debug_log() {
  [[ "${RALPH_DEBUG:-0}" == "1" ]] && echo "DEBUG: $*" >&2
}

# Clean status line with emoji
status_line() {
  local icon="$1" action="$2" detail="$3" result="$4"
  local iter_prefix=""
  if [[ -n "$CURRENT_ITERATION" && -n "$MAX_ITERATIONS" ]]; then
    iter_prefix="[iter $CURRENT_ITERATION/$MAX_ITERATIONS] "
  fi
  # Output to stderr so it's visible even inside $() captures
  echo "${iter_prefix}${icon} ${action}: ${detail} ... ${result}" >&2
}

# Print end-of-run summary
print_summary() {
  local total_seconds=$(( SECONDS - SCRIPT_START_SECONDS ))
  local minutes=$(( total_seconds / 60 ))
  local seconds=$(( total_seconds % 60 ))
  local stories_total=0 stories_passed=0

  if [ -f "$PRD_FILE" ]; then
    stories_total=$(jq '.userStories | length' "$PRD_FILE" 2>/dev/null || echo 0)
    stories_passed=$(jq '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE" 2>/dev/null || echo 0)
  fi

  # Bulletproof: count LLM calls from debug log (file I/O always works)
  local api_calls=0
  if [ -f "$LOG_FILE"]; then
    api_calls=$(grep -c "execute_nvidia_actions - true" "$LOG_FILE" 2>/dev/null || echo 0)
  fi

  echo ""
  echo "═══════════════════════════════════════"
  echo "  Ralph v2.0 Summary"
  echo "───────────────────────────────────────"
  echo "  Stories:    ${stories_passed}/${stories_total} completed"
  echo "  Iterations: ${CURRENT_ITERATION:-?}/${MAX_ITERATIONS} used"
  echo "  LLM rounds: ${api_calls}"
  echo "  Total time: ${minutes}m ${seconds}s"
  echo "  Model:      ${LLM_MODEL:-N/A}"
  echo "═══════════════════════════════════════"
}

# --- Phase 1: Memory helpers ---

# Append a learning to learnings.md
append_learning() {
  local message="$1"
  local timestamp
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "- [$timestamp] $message" >> "$LEARNINGS_FILE"
}

# Initialize memory files if they don't exist
init_memory_files() {
  if [ ! -f "$LEARNINGS_FILE" ]; then
    cat > "$LEARNINGS_FILE" << 'LEARNEOF'
# Learnings Log
# Append-only log of insights from each Ralph iteration
# Format: - [timestamp] message
---
LEARNEOF
    debug_log "Created $LEARNINGS_FILE"
  fi

  if [ ! -f "$AGENTS_FILE" ]; then
    cat > "$AGENTS_FILE" << 'AGENTSEOF'
# AGENTS.md — Ralph Agent Knowledge Base

This file is read by Ralph at the start of each iteration.
Ralph updates it with discovered patterns, gotchas, and conventions.
Human developers can also add rules here.

## Discovered Patterns
<!-- Ralph will add patterns here as it learns -->

## Known Gotchas
<!-- Ralph will add gotchas here -->

## Project Conventions
<!-- Ralph will add conventions here -->
AGENTSEOF
    debug_log "Created $AGENTS_FILE"
  fi
  # STEERING.md is NOT auto-created — humans create it when needed
}

# --- Phase 2: Error recovery helpers ---

# Compute hash of error message for stuck detection
compute_error_hash() {
  local text="$1"
  if command -v md5sum &>/dev/null; then
    echo "$text" | md5sum | cut -d' ' -f1
  else
    # Fallback: use character count + first 80 chars
    echo "${#text}:${text:0:80}"
  fi
}

# Check if agent is stuck (same error repeating)
# Sets CONSECUTIVE_SAME_ERRORS and returns "stuck" or "ok"
check_stuck() {
  local error_msg="$1"
  local error_hash
  error_hash=$(compute_error_hash "$error_msg")

  if [[ "$error_hash" == "$LAST_ERROR_HASH" ]]; then
    (( CONSECUTIVE_SAME_ERRORS++ )) || true
  else
    CONSECUTIVE_SAME_ERRORS=1
    LAST_ERROR_HASH="$error_hash"
  fi

  if (( CONSECUTIVE_SAME_ERRORS >= STUCK_THRESHOLD )); then
    echo "stuck"
  else
    echo "ok"
  fi
}

# --- Phase 3b: Guardrails ---

# Check if a file is protected from direct editing
# Returns: 0 = allowed, 1 = blocked
# Prints warning/error message
check_protected_file() {
  local filepath="$1"
  local action_type="$2"  # edit_file or create_file

  # Hard block
  case "$filepath" in
    .env|.env.*|.git|.git/*)
      echo "🛡️ BLOCKED: Cannot $action_type protected file: $filepath"
      return 1
      ;;
  esac

  # Soft warning
  case "$filepath" in
    prd.json|./prd.json)
      echo "⚠️ WARNING: Direct edit of prd.json. Use 'finish' action to mark stories as passed."
      return 0
      ;;
  esac

  return 0
}

# ═══════════════════════════════════════════════════════════════
# SECTION 8: Archive + Branch Tracking
# ═══════════════════════════════════════════════════════════════

if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
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
    echo "  Archived to: $ARCHIVE_FOLDER"

    echo "# Ralph Progress Log" > "$PROGRESS_FILE"
    echo "Started: $(date)" >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"
  fi
fi

if [ -f "$PRD_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  if [ -n "$CURRENT_BRANCH" ]; then
    echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"
  fi
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 9: Initialize Memory + Progress + Banner (Phase 1)
# ═══════════════════════════════════════════════════════════════

init_memory_files

# Reset API stats for this run
  # rm -f "$SCRIPT_DIR/.api_stats"  # disabled: using debug log for stats

  if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

echo ""
echo "═══════════════════════════════════════"
echo "  Ralph v2.0 — Tool: $TOOL"
echo "  Max iterations: $MAX_ITERATIONS"
[[ "$TOOL" == "llm_direct_nvidia" ]] && echo "  Model: $LLM_MODEL"
[[ "${RALPH_DEBUG:-0}" == "1" ]] && echo "  Debug: ON"
echo "═══════════════════════════════════════"
echo ""

# ═══════════════════════════════════════════════════════════════
# SECTION 10: NVIDIA API Call Function
# ═══════════════════════════════════════════════════════════════

llm_call_nvidia() {
  local prompt="$1"
  local history="$2"

  local messages
  messages=$(jq -n --arg sys "$LLM_SYSTEM_PROMPT" \
    --argjson hist "$history" \
    --arg prompt "$prompt" \
    '[{role: "system", content: $sys}] + $hist + [{role: "user", content: $prompt}]')

  local _api_body
  _api_body=$(jq -n \
    --arg model "$LLM_MODEL" \
    --argjson msgs "$messages" \
    --argjson temp "${LLM_TEMPERATURE}" \
    --argjson max_tokens 8192 \
    '{model: $model, messages: $msgs, temperature: $temp, max_tokens: $max_tokens, stream: false}')

  local _retry _api_result _backoff_delay=5
  for _retry in 1 2 3; do
    local _call_start=$SECONDS
    _api_result=$(curl -s --max-time 120 -X POST "${LLM_BASE_URL}/chat/completions" \
      -H "Authorization: Bearer $LLM_API_KEY" \
      -H "Content-Type: application/json" \
      -d "$_api_body")
    local _call_duration=$(( SECONDS - _call_start ))
    # echo "1 $_call_duration" >> "$SCRIPT_DIR/.api_stats"  # disabled: using debug log for stats
    # (( TOTAL_API_CALLS++ )) || true
    # (( TOTAL_API_TIME += _call_duration )) || true

    # Check for API-level error
    local _api_error
    _api_error=$(echo "$_api_result" | jq -r '.error.message // empty' 2>/dev/null)
    if [[ -n "$_api_error" ]]; then
      debug_log "API error: $_api_error"
      (( CONSECUTIVE_API_ERRORS++ )) || true

      # Phase 2 Level 4: Fallback model after 3 consecutive errors
      if (( CONSECUTIVE_API_ERRORS >= 3 )); then
        for _fb_model in "${LLM_FALLBACK_MODELS[@]}"; do
          if [[ "$_fb_model" != "$LLM_MODEL" ]]; then
            status_line "🔄" "fallback" "$LLM_MODEL → $_fb_model" "after $CONSECUTIVE_API_ERRORS errors"
            append_learning "Model fallback: $LLM_MODEL → $_fb_model after $CONSECUTIVE_API_ERRORS API errors"
            LLM_MODEL="$_fb_model"
            CONSECUTIVE_API_ERRORS=0
            _api_body=$(jq -n \
              --arg model "$LLM_MODEL" \
              --argjson msgs "$messages" \
              --argjson temp "${LLM_TEMPERATURE}" \
              --argjson max_tokens 8192 \
              '{model: $model, messages: $msgs, temperature: $temp, max_tokens: $max_tokens, stream: false}')
            break
          fi
        done
      fi
    fi

    # Check for valid response with content
    if echo "$_api_result" | jq -e '.choices[0].message.content' > /dev/null 2>&1; then
      CONSECUTIVE_API_ERRORS=0
      echo "$_api_result"
      return 0
    fi

    debug_log "API attempt $_retry failed, retrying in ${_backoff_delay}s"
    sleep "$_backoff_delay"
    _backoff_delay=$(( _backoff_delay * 3 ))
  done

  echo "$_api_result"
}

# ═══════════════════════════════════════════════════════════════
# SECTION 11: Execute Actions (Phase 0 bugfixes, Phase 3b guardrails)
# ═══════════════════════════════════════════════════════════════

execute_nvidia_actions() {
  local actions_start_seconds=$SECONDS
  local actions_json="$1"
  local action_count=0
  log_action "start" "execute_nvidia_actions" "true"

  while read -r action; do
    local type
    type=$(echo "$action" | jq -r '.type // empty')
    (( action_count++ )) || true
    debug_log "Action #$action_count: $type"

    case "$type" in

      # --- Phase 3a: Plan/write_todos (no-op context engineering) ---
      "plan"|"write_todos")
        local todos
        todos=$(echo "$action" | jq -r '.todos // .plan // empty')
        if [[ -n "$todos" ]]; then
          echo "📋 PLAN:"
          echo "$todos"
          status_line "📋" "plan" "recorded" "OK"
          log_action "plan" "$todos" "true"
        else
          echo "📋 PLAN: (empty plan)"
          log_action "plan" "(empty)" "true"
        fi
        ;;

      "list_files")
        log_action "start" "$action" "false"
        local path
        path=$(echo "$action" | jq -r '.path // "."')
        if [[ -d "$path" ]]; then
          ls -la "$path"
          status_line "📂" "list_files" "$path" "OK"
          log_action "end" "$action" "true"
        else
          echo "ERROR [list_files]: Directory not found: $path"
          status_line "📂" "list_files" "$path" "FAIL (not found)"
          log_action "end" "$action" "false"
        fi
        ;;

      "read_file")
        log_action "start" "$action" "false"
        local path
        path=$(echo "$action" | jq -r '.path // empty')
        local line_start line_end
        line_start=$(echo "$action" | jq -r '.line_start // empty')
        line_end=$(echo "$action" | jq -r '.line_end // empty')

        # SANDBOX: staged ralph.sh
        if [[ ("$path" == "ralph.sh" || "$path" == "./ralph.sh") && -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
          path="$SCRIPT_DIR/.staging/ralph.sh"
          echo "SANDBOX: Reading staged version of ralph.sh"
        fi

        if [[ -n "$path" && -f "$path" ]]; then
          local total_lines
          total_lines=$(wc -l < "$path")
          if [[ -n "$line_start" && -n "$line_end" ]]; then
            sed -n "${line_start},${line_end}p" "$path"
            echo "[Read lines $line_start-$line_end of $total_lines total]"
          elif (( total_lines > 200 )); then
            head -n 50 "$path"
            local omitted=$(( total_lines - 100 ))
            echo ""
            echo "[... $omitted lines omitted (lines 51-$(( total_lines - 50 ))) ...]"
            echo "[Use read_file with line_start/line_end to read specific sections]"
            echo ""
            tail -n 50 "$path"
            echo "[File chunked: $total_lines lines total, showing first 50 + last 50]"
          else
            cat "$path"
          fi
          status_line "📖" "read_file" "$path (${total_lines} lines)" "OK"
          log_action "end" "$action" "true"
        else
          echo "ERROR [read_file]: File not found: $path"
          status_line "📖" "read_file" "$path" "FAIL (not found)"
          log_action "end" "$action" "false"
        fi
        ;;

      "edit_file")
        log_action "start" "$action" "false"
        local path
        path=$(echo "$action" | jq -r '.path // empty')
        local changes
        changes=$(echo "$action" | jq -c '.changes // empty')

        if [[ -z "$path" || -z "$changes" || "$changes" == "null" ]]; then
          echo "ERROR [edit_file]: Missing path or changes"
          log_action "end" "$action" "false"
          continue
        fi

        # Phase 3b: Protected file check
        local _guard_msg
        if ! _guard_msg=$(check_protected_file "$path" "edit_file"); then
          echo "$_guard_msg"
          log_action "end" "BLOCKED edit_file $path" "false"
          continue
        fi
        [[ -n "$_guard_msg" ]] && echo "$_guard_msg"

        # SANDBOX: redirect ralph.sh edits to staging
        if [[ "$path" == "ralph.sh" || "$path" == "./ralph.sh" ]]; then
          mkdir -p "$SCRIPT_DIR/.staging"
          if [[ ! -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
            cp "$SCRIPT_DIR/ralph.sh" "$SCRIPT_DIR/.staging/ralph.sh"
            echo "SANDBOX: Created staging copy of ralph.sh"
          fi
          path="$SCRIPT_DIR/.staging/ralph.sh"
          echo "SANDBOX: Editing staging copy (not live code)"
        fi

        echo "$changes" | jq -c '.[]' | while read -r change; do
          local old_str new_str
          old_str=$(echo "$change" | jq -r '.oldString // empty')
          new_str=$(echo "$change" | jq -r '.newString // empty')
          if [[ -n "$old_str" && -f "$path" ]]; then
            _EP="$path" _EO="$old_str" _EN="$new_str" python3 -c \
              "import os;p=os.environ['_EP'];c=open(p).read().replace(os.environ['_EO'],os.environ['_EN'],1);open(p,'w').write(c)"
            status_line "✏️" "edit_file" "$path" "OK"
            log_action "end" "$action" "true"
          else
            echo "WARNING [edit_file]: Skipping invalid change in $path"
            log_action "end" "$action" "false"
          fi
        done
        ;;

      "create_file")
        log_action "start" "$action" "false"
        local path
        path=$(echo "$action" | jq -r '.path // .filename // empty')
        local content
        content=$(echo "$action" | jq -r '.content // empty')

        if [[ -z "$path" ]]; then
          echo "ERROR [create_file]: Missing path"
          log_action "end" "$action" "false"
          continue
        fi
        if [[ -z "$content" ]]; then
          echo "ERROR [create_file]: Missing content"
          log_action "end" "$action" "false"
          continue
        fi

        # Phase 3b: Protected file check
        local _guard_msg
        if ! _guard_msg=$(check_protected_file "$path" "create_file"); then
          echo "$_guard_msg"
          log_action "end" "BLOCKED create_file $path" "false"
          continue
        fi
        [[ -n "$_guard_msg" ]] && echo "$_guard_msg"

        mkdir -p "$(dirname "$path")"
        printf '%s\n' "$content" > "$path"
        status_line "📝" "create_file" "$path" "OK"
        log_action "end" "$action" "true"
        ;;

      "run_command")
        log_action "start" "$action" "false"
        local command
        command=$(echo "$action" | jq -r '.command // empty')

        if [[ -z "$command" ]]; then
          echo "ERROR [run_command]: Empty command"
          log_action "end" "$action" "false"
          continue
        fi

        debug_log "Running: $command"
        local _cmd_start=$SECONDS
        local output exit_code

        # Phase 0: Isolated execution via bash -c (subprocess-style)
        output=$(cd "$SCRIPT_DIR" && bash -c "$command" 2>&1) || true
        exit_code=$?
        local _cmd_dur=$(( SECONDS - _cmd_start ))

        echo "$output"
        if [[ $exit_code -ne 0 ]]; then
          status_line "🏃" "run_command" "$command" "FAIL (exit $exit_code, ${_cmd_dur}s)"
          log_action "end" "$action" "false"
        else
          status_line "🏃" "run_command" "$command" "OK (${_cmd_dur}s)"
          log_action "end" "$action" "true"
        fi
        ;;

      "finish")
        log_action "start" "finish" "false"

        # SANDBOX: validate staged ralph.sh
        if [[ -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
          echo "SANDBOX: Validating staged ralph.sh with bash -n..."
          local _sandbox_err
          _sandbox_err=$(bash -n "$SCRIPT_DIR/.staging/ralph.sh" 2>&1)
          if [[ $? -eq 0 ]]; then
            echo "SANDBOX: ✅ Syntax OK — applying changes"
            echo "SANDBOX: Diff:"
            diff "$SCRIPT_DIR/ralph.sh" "$SCRIPT_DIR/.staging/ralph.sh" || true
            cp "$SCRIPT_DIR/.staging/ralph.sh" "$SCRIPT_DIR/ralph.sh"
            rm -rf "$SCRIPT_DIR/.staging"
          else
            echo "SANDBOX: ❌ SYNTAX ERROR — changes REJECTED"
            echo "SANDBOX: Error: $_sandbox_err"
            rm -rf "$SCRIPT_DIR/.staging"
            echo "Fix syntax errors and try finish again."
            log_action "end" "finish" "false"
            continue
          fi
        fi

        # Get current failing story
        local _sid
        _sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$SCRIPT_DIR/prd.json" 2>/dev/null)

        # Phase 0 fix: single code path for marking passes=true
        local _story_verified=false

        if [[ -n "$_sid" && "$_sid" != "null" ]]; then
          local _verification_cmd
          _verification_cmd=$(jq -r --arg id "$_sid" \
            '.userStories[] | select(.id == $id) | .verificationCommand // empty' \
            "$SCRIPT_DIR/prd.json" 2>/dev/null)

          if [[ -n "$_verification_cmd" && "$_verification_cmd" != "null" ]]; then
            # Has verificationCommand — run TDD check
            echo "VERIFICATION: Running for story $_sid:"
            echo "VERIFICATION: $ $_verification_cmd"

            # Rejection counter
            if [[ ! -f "$SCRIPT_DIR/.rejection_count" ]]; then
              echo 0 > "$SCRIPT_DIR/.rejection_count"
            fi
            local _current_rejections
            _current_rejections=$(cat "$SCRIPT_DIR/.rejection_count")

            # Phase 0: isolated execution + Phase 2: capture output for feedback
            local _verify_output _verify_exit
            _verify_output=$(cd "$SCRIPT_DIR" && bash -c "$_verification_cmd" 2>&1) || true
            _verify_exit=$?

            if [[ $_verify_exit -eq 0 ]]; then
              echo "VERIFICATION: ✅ PASSED — Story $_sid verified"
              echo "$_verify_output"
              echo 0 > "$SCRIPT_DIR/.rejection_count"
              _story_verified=true
              append_learning "Story $_sid passed verification: $_verification_cmd"
            else
              # Phase 2 Level 2: detailed failure feedback
              local _new_rejections=$(( _current_rejections + 1 ))
              echo "$_new_rejections" > "$SCRIPT_DIR/.rejection_count"

              echo "VERIFICATION: ❌ FAILED (exit $_verify_exit, rejection $_new_rejections)"
              echo "VERIFICATION OUTPUT:"
              echo "$_verify_output"
              echo "---"

              if (( _new_rejections >= 5 )); then
                echo "VERIFICATION: 🚨 CRITICAL — $_new_rejections consecutive failures."
                echo "VERIFICATION: Consider a COMPLETELY DIFFERENT approach."
                echo "VERIFICATION: Re-read the acceptance criteria and test output above."
              else
                echo "VERIFICATION: Analyze the output above. Fix the issues and try finish again."
              fi

              append_learning "Story $_sid verification FAILED (attempt $_new_rejections): exit $_verify_exit"
              log_action "end" "finish" "false"
              continue
            fi
          else
            # No verificationCommand — trust the agent
            echo "VERIFICATION: No verificationCommand for $_sid — trusting agent"
            _story_verified=true
          fi

          # Phase 0 fix: single conditional mark
          if [[ "$_story_verified" == "true" ]]; then
            local _tmp
            _tmp=$(mktemp)
            jq --arg id "$_sid" \
              '(.userStories[] | select(.id == $id)).passes = true' \
              "$SCRIPT_DIR/prd.json" > "$_tmp" && mv "$_tmp" "$SCRIPT_DIR/prd.json"
            echo "✅ Story $_sid marked as PASSED"
            echo 0 > "$SCRIPT_DIR/.rejection_count" 2>/dev/null || true
          fi
        else
          echo "FINISH: No failing stories found — all done"
        fi

        # Check remaining stories
        local _remaining
        _remaining=$(jq '[.userStories[] | select(.passes == false)] | length' "$SCRIPT_DIR/prd.json" 2>/dev/null || echo 0)

        if (( _remaining == 0 )); then
          echo "<promise>COMPLETE</promise>"
        else
          echo "STORY_COMPLETED: $_sid ($_remaining stories remaining)"
          echo "<promise>STORY_COMPLETED</promise>"
        fi
        log_action "end" "finish" "true"
        return 0
        ;;

      *)
        echo "WARNING: Unknown action type: $type"
        log_action "unknown_action" "$type" "false"
        ;;
    esac

  done < <(echo "$actions_json" | jq -c '.actions[]')

  log_action "end" "execute_nvidia_actions" "true"
  local actions_duration=$(( SECONDS - actions_start_seconds ))
  debug_log "Executed $action_count actions in ${actions_duration}s"
}

# ═══════════════════════════════════════════════════════════════
# SECTION 12: Construct Prompt (Phase 3c — Better Context)
# ═══════════════════════════════════════════════════════════════

construct_nvidia_prompt() {
  local context_json="$1"

  # Phase 1c: STEERING.md — human live override (highest priority)
  if [ -f "$STEERING_FILE" ] && [ -s "$STEERING_FILE" ]; then
    echo "═══ STEERING INSTRUCTIONS (PRIORITY — follow these first) ═══"
    cat "$STEERING_FILE"
    echo "═══ END STEERING ═══"
    echo ""
  fi

  # Story context
  echo "═══ CURRENT STORY ═══"
  echo "$context_json"
  echo ""

  # Phase 1b: AGENTS.md — accumulated knowledge
  if [ -f "$AGENTS_FILE" ]; then
    local _agents_size
    _agents_size=$(wc -c < "$AGENTS_FILE")
    if (( _agents_size < 3000 )); then
      echo "═══ AGENT KNOWLEDGE BASE (AGENTS.md) ═══"
      cat "$AGENTS_FILE"
      echo "═══ END AGENTS.md ═══"
      echo ""
    else
      echo "═══ AGENT KNOWLEDGE BASE (last 30 lines) ═══"
      tail -n 30 "$AGENTS_FILE"
      echo "═══ END AGENTS.md ═══"
      echo ""
    fi
  fi

  # Phase 1a: Recent learnings
  if [ -f "$LEARNINGS_FILE" ]; then
    local _learn_lines
    _learn_lines=$(wc -l < "$LEARNINGS_FILE")
    if (( _learn_lines > 5 )); then
      echo "═══ RECENT LEARNINGS (last 10) ═══"
      tail -n 10 "$LEARNINGS_FILE"
      echo "═══ END LEARNINGS ═══"
      echo ""
    fi
  fi

  # Progress context
  echo "═══ RECENT PROGRESS (last 8 lines) ═══"
  tail -n 8 "$SCRIPT_DIR/progress.txt" 2>/dev/null || echo "(no progress yet)"
  echo ""

  # Project structure
  echo "═══ PROJECT FILES ═══"
  find "$SCRIPT_DIR" -maxdepth 2 -type f \
    \( -name "*.sh" -o -name "*.json" -o -name "*.md" -o -name "*.txt" -o -name "*.py" -o -name "*.js" \) \
    ! -path "*/.git/*" ! -path "*/node_modules/*" ! -path "*/archive/*" \
    2>/dev/null | sort | head -30
  echo ""

  # Git status
  echo "═══ GIT STATUS ═══"
  cd "$SCRIPT_DIR" && git status --short 2>/dev/null | head -10
  echo ""
  echo "═══ LAST 3 COMMITS ═══"
  cd "$SCRIPT_DIR" && git log --oneline -3 2>/dev/null || echo "(no git history)"
  echo ""
}

# ═══════════════════════════════════════════════════════════════
# SECTION 13: Get Story Context
# ═══════════════════════════════════════════════════════════════

get_story_context() {
  local prd_file="$SCRIPT_DIR/prd.json"

  if [ ! -f "$prd_file" ]; then
    echo '{"error": "PRD file not found"}'
    return
  fi

  local story
  story=$(jq -c '[.userStories[] | select(.passes == false)][0]' "$prd_file" 2>/dev/null)

  if [[ -z "$story" || "$story" == "null" ]]; then
    echo "{}"
    return
  fi

  echo "$story" | jq -c '{id, title, description, acceptanceCriteria, branchName, verificationCommand}'
}

# ═══════════════════════════════════════════════════════════════
# SECTION 14: Main Agent Loop (Phase 1 memory, Phase 2 recovery)
# ═══════════════════════════════════════════════════════════════

run_nvidia_agent_loop() {
  local conversation_history='[]'
  local iteration_count=0
  local consecutive_json_fails=0

  # Phase 2: per-loop error tracking
  local loop_same_errors=0
  local loop_last_error_hash=""

  debug_log "Starting agent loop"

  while true; do
    debug_log "Agent loop iteration $iteration_count"

    # Get current story context
    local context
    context=$(get_story_context)

    # Early exit: no more stories
    if [[ "$context" == "{}" || -z "$context" ]]; then
      echo "All stories completed!"
      echo "<promise>COMPLETE</promise>"
      return 0
    fi

    # Display current story info
    local _story_id _story_title
    _story_id=$(echo "$context" | jq -r '.id // "?"')
    _story_title=$(echo "$context" | jq -r '.title // "unknown"')
    if (( iteration_count % 5 == 0 )); then
      echo "📌 Working on: [$_story_id] $_story_title (inner iteration $iteration_count)"
    fi

    # Build prompt with full context (Phase 3c)
    local full_prompt
    full_prompt=$(construct_nvidia_prompt "$context")

    # Phase 1c: Inject STEERING.md as priority context
    # (Already handled inside construct_nvidia_prompt)

    # Call NVIDIA API
    local llm_response
    llm_response=$(llm_call_nvidia "$full_prompt" "$conversation_history")

    # Extract content
    local llm_content
    if ! llm_content=$(echo "$llm_response" | jq -r \
      '(.choices[0].message.content // .choices[0].message.reasoning_content // .choices[0].message.reasoning) // empty' \
      2>/dev/null); then
      echo "ERROR: Failed to parse LLM response" >&2
      echo "$llm_response" > /tmp/llm_response_debug.txt
      return 1
    fi

    debug_log "LLM content length: ${#llm_content}"

    if [[ -z "$llm_content" || "$llm_content" == "null" ]]; then
      echo "ERROR: Empty response from NVIDIA API"
      debug_log "Full response: $llm_response"
      # Phase 2 Level 4: don't immediately fail — retry
      if (( iteration_count < 3 )); then
        echo "Retrying after empty response..."
        (( iteration_count++ )) || true
        sleep 5
        continue
      fi
      return 1
    fi

    # Strip markdown fences
    llm_content=$(echo "$llm_content" | sed '/^```[a-z]*$/d' | sed '/^```$/d')

    # Parse JSON
    local actions_json
    actions_json=$(echo "$llm_content" | jq -e '.' 2>/dev/null) || true

    if [[ -z "$actions_json" || "$actions_json" == "null" ]]; then
      (( consecutive_json_fails++ )) || true
      echo "WARNING: LLM returned invalid JSON (attempt $consecutive_json_fails)"

      if (( consecutive_json_fails >= 3 )); then
        # Phase 2 Level 1: Enhanced format error feedback with example
        echo "ERROR: $consecutive_json_fails consecutive JSON failures — aborting"
        append_learning "JSON parse failures: $consecutive_json_fails consecutive. Model: $LLM_MODEL"
        return 1
      fi

      # Phase 2 Level 1: Stronger recovery feedback with example
      local recovery_msg
      recovery_msg="ERROR: Your response was NOT valid JSON. You MUST respond with ONLY a JSON object like this:
{
  \"thoughts\": \"your reasoning here\",
  \"actions\": [{\"type\": \"read_file\", \"path\": \"prd.json\"}]
}
Do NOT include any text outside the JSON. No markdown fences. No explanations. ONLY the JSON object."

      conversation_history=$(echo "$conversation_history" | jq \
        --arg content "$llm_content" \
        '. + [{"role": "assistant", "content": $content}]')
      conversation_history=$(echo "$conversation_history" | jq \
        --arg msg "$recovery_msg" \
        '. + [{"role": "user", "content": $msg}]')

      (( iteration_count++ )) || true
      continue
    else
      consecutive_json_fails=0
    fi

    # Add LLM response to history
    conversation_history=$(echo "$conversation_history" | jq \
      --arg content "$llm_content" \
      '. + [{"role": "assistant", "content": $content}]')

    # Execute actions
    local execution_result
    execution_result=$(execute_nvidia_actions "$actions_json") || true
    local exec_exit=$?

    # Add result to history
    conversation_history=$(echo "$conversation_history" | jq \
      --arg res "$execution_result" \
      '. + [{"role": "user", "content": ("Result:\n" + $res)}]')

    # Check for completion signals
    if [[ $exec_exit -eq 0 ]]; then
      if echo "$execution_result" | tail -n 5 | grep -qF "<promise>COMPLETE</promise>"; then
        append_learning "All stories completed successfully"
        echo "<promise>COMPLETE</promise>"
        return 0
      elif echo "$execution_result" | tail -n 5 | grep -qF "<promise>STORY_COMPLETED</promise>"; then
        echo "Moving to next story..."
        append_learning "Story $_story_id completed, moving to next"
        conversation_history='[]'
        loop_same_errors=0
        loop_last_error_hash=""
        (( iteration_count++ )) || true
        continue
      fi
    fi

    # --- Phase 2: Stuck Detection (Level 3) ---
    if echo "$execution_result" | grep -qiE "FAIL|ERROR|REJECTED|FAILED"; then
      local _err_text
      _err_text=$(echo "$execution_result" | grep -iE "FAIL|ERROR|REJECTED|FAILED" | head -5)
      local _err_hash
      _err_hash=$(compute_error_hash "$_err_text")

      if [[ "$_err_hash" == "$loop_last_error_hash" ]]; then
        (( loop_same_errors++ )) || true
      else
        loop_same_errors=1
        loop_last_error_hash="$_err_hash"
      fi

      debug_log "Error repeat count: $loop_same_errors (threshold: $STUCK_THRESHOLD)"

      if (( loop_same_errors >= SKIP_THRESHOLD )); then
        # Skip this story entirely
        echo "🚫 STUCK: Skipping story $_story_id after $loop_same_errors repeated errors"
        append_learning "SKIPPED story $_story_id: stuck on same error $loop_same_errors times. Error: $_err_text"
        conversation_history='[]'
        loop_same_errors=0
        loop_last_error_hash=""
        # Don't mark as passed — leave for manual intervention
        (( iteration_count++ )) || true
        continue

      elif (( loop_same_errors >= STUCK_THRESHOLD )); then
        # Inject self-reflection prompt
        local _reflection
        _reflection="⚠️ SELF-REFLECTION REQUIRED: You have been stuck for $loop_same_errors iterations with the same error pattern.

Recent error:
$_err_text

Please:
1. Explain WHY you think this error keeps recurring
2. List what you have already tried
3. Propose a COMPLETELY DIFFERENT approach
4. If truly stuck, document the blocker in progress.txt and use finish to move on"

        conversation_history=$(echo "$conversation_history" | jq \
          --arg msg "$_reflection" \
          '. + [{"role": "user", "content": $msg}]')
        echo "🔄 Self-reflection injected (stuck $loop_same_errors iterations)"
      fi
    else
      # No errors — reset stuck counter
      if (( loop_same_errors > 0 )); then
        loop_same_errors=0
        loop_last_error_hash=""
      fi
    fi

    # Context management: adaptive sliding window
    local _ctx_status
    _ctx_status=$(check_context_usage "$conversation_history")
    local hist_len
    hist_len=$(echo "$conversation_history" | jq 'length')

    if [[ "$_ctx_status" == "shrink" ]]; then
      if (( hist_len > 6 )); then
        conversation_history=$(echo "$conversation_history" | jq '.[-6:]')
        debug_log "Context critical — trimmed to 6 messages"
      fi
    elif (( hist_len > 10 )); then
      conversation_history=$(echo "$conversation_history" | jq '.[-10:]')
      debug_log "Trimmed history from $hist_len to 10 messages"
    fi

    # Safety: max inner iterations
    if (( iteration_count++ > 40 )); then
      echo "ERROR: Exceeded maximum agent iterations (40)"
      append_learning "Agent loop hit 40-iteration limit on story $_story_id"
      return 1
    fi
  done
}

# ═══════════════════════════════════════════════════════════════
# SECTION 15: Main Outer Loop
# ═══════════════════════════════════════════════════════════════

CHECKPOINT_FILE="$SCRIPT_DIR/.checkpoint_iter"
START_ITERATION=1

if [ -f "$CHECKPOINT_FILE" ]; then
  START_ITERATION=$(cat "$CHECKPOINT_FILE")
  echo "Resuming from checkpoint: iteration $START_ITERATION"
fi

for i in $(seq "$START_ITERATION" "$MAX_ITERATIONS"); do

  echo "$i" > "$CHECKPOINT_FILE"

  echo ""
  echo "═══════════════════════════════════════════════════════════════"
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "═══════════════════════════════════════════════════════════════"

  CURRENT_ITERATION=$i

  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  elif [[ "$TOOL" == "claude" ]]; then
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  elif [[ "$TOOL" == "llm_direct_nvidia" ]]; then
    _OUTPUT_FILE=$(mktemp)
    run_nvidia_agent_loop 2>&1 | tee "$_OUTPUT_FILE"
    OUTPUT=$(cat "$_OUTPUT_FILE")
  fi

  # Check for completion
  if echo "$OUTPUT" | grep -qF "<promise>COMPLETE</promise>"; then
    print_summary

    # Archive runtime files
    if [ -f "$PRD_FILE" ]; then
      CURRENT_BRANCH=$(jq -r '.branchName // "completed"' "$PRD_FILE" 2>/dev/null)
      FOLDER_NAME=$(echo "$CURRENT_BRANCH" | sed 's|^ralph/||')
      CLEANUP_DIR="$ARCHIVE_DIR/$(date +%Y-%m-%d)-$FOLDER_NAME"

      echo "Archiving runtime files to: $CLEANUP_DIR"
      mkdir -p "$CLEANUP_DIR"
      [ -f "$PRD_FILE" ] && mv "$PRD_FILE" "$CLEANUP_DIR/"
      [ -f "$PROGRESS_FILE" ] && mv "$PROGRESS_FILE" "$CLEANUP_DIR/"
      [ -f "$LAST_BRANCH_FILE" ] && mv "$LAST_BRANCH_FILE" "$CLEANUP_DIR/last-branch.txt"
      [ -f "$SCRIPT_DIR/.rejection_count" ] && mv "$SCRIPT_DIR/.rejection_count" "$CLEANUP_DIR/"
      [ -f "$CHECKPOINT_FILE" ] && mv "$CHECKPOINT_FILE" "$CLEANUP_DIR/"
      # [ -f "$SCRIPT_DIR/.api_stats" ] && mv "$SCRIPT_DIR/.api_stats" "$CLEANUP_DIR/"
      # NOTE: learnings.md and AGENTS.md are NOT archived — they persist across runs
      echo "Runtime files archived. learnings.md and AGENTS.md preserved."
    fi

    exit 0
  fi

  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE and $LEARNINGS_FILE for status."
print_summary
exit 1