#!/bin/bash

# Ralph Wiggum v2.0 — Autonomous AI Agent Loop
# Clean rewrite with Phases 0-3
#
# Usage: ./ralph-v2.sh --llm-provider nvidia [--max-iterations N] [--debug]

set -e

# ═══════════════════════════════════════════════════════════════
# SECTION 1: Dependencies
# ═══════════════════════════════════════════════════════════════

if ! command -v jq &>/dev/null; then
  echo "Error: jq required. Install: sudo apt install jq"; exit 1
fi
if ! command -v curl &>/dev/null; then
  echo "Error: curl required. Install: sudo apt install curl"; exit 1
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
    --tool)          TOOL="$2"; shift 2 ;;
    --tool=*)        TOOL="${1#*=}"; shift ;;
    --llm-provider)  LLM_PROVIDER="$2"; shift 2 ;;
    --llm-api-key)   LLM_API_KEY="$2"; shift 2 ;;
    --llm-model)     LLM_MODEL="$2"; shift 2 ;;
    --llm-base-url)  LLM_BASE_URL="$2"; shift 2 ;;
    --llm-temperature) LLM_TEMPERATURE="$2"; shift 2 ;;
    --max-iterations)  MAX_ITERATIONS="$2"; shift 2 ;;
    --max-iterations=*) MAX_ITERATIONS="${1#*=}"; shift ;;
    --debug)         export RALPH_DEBUG=1; shift ;;
    --version|-v)    echo "Ralph v2.0"; exit 0 ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then MAX_ITERATIONS="$1"; fi
      shift ;;
  esac
done

# ═══════════════════════════════════════════════════════════════
# SECTION 3: Environment
# ═══════════════════════════════════════════════════════════════

if [ -f ".env" ]; then
  # shellcheck disable=SC1091
  . .env
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 4: NVIDIA Provider Setup + System Prompt
# ═══════════════════════════════════════════════════════════════

if [[ -n "$LLM_PROVIDER" ]]; then
  if [[ "$LLM_PROVIDER" != "nvidia" ]]; then
    echo "Error: Only '--llm-provider nvidia' is supported."; exit 1
  fi

  [[ -z "$LLM_API_KEY" ]] && LLM_API_KEY="${NVIDIA_API_KEY:-}"
  if [[ -z "$LLM_API_KEY" ]]; then
    echo "Error: NVIDIA_API_KEY or --llm-api-key required"; exit 1
  fi

  LLM_BASE_URL="${LLM_BASE_URL:-https://integrate.api.nvidia.com/v1}"
  LLM_MODEL="${LLM_MODEL:-moonshotai/kimi-k2-instruct}"

  read -r -d '' LLM_SYSTEM_PROMPT << 'SYSPROMPT' || true
You are an autonomous coding agent implementing user stories from a PRD.
You MUST respond with ONLY a valid JSON object — no text before or after.

RESPONSE FORMAT:
{
  "thoughts": "your reasoning about what to do next",
  "actions": [action1, action2, ...]
}

ACTION TYPES:

1. plan (ALWAYS use before implementing):
   { "type": "plan", "todos": "1. Step one\n2. Step two" }

2. read_file:
   { "type": "read_file", "path": "file.ext" }
   Optional: "line_start": N, "line_end": M for ranges.

3. list_files:
   { "type": "list_files", "path": "directory/" }

4. edit_file:
   { "type": "edit_file", "path": "file.ext", "changes": [
     { "oldString": "exact text", "newString": "replacement" }
   ]}

5. create_file:
   { "type": "create_file", "path": "file.ext", "content": "content" }

6. run_command:
   { "type": "run_command", "command": "shell command" }

7. finish:
   { "type": "finish" }
   Triggers verificationCommand if defined. MUST git commit BEFORE calling this.

EXAMPLE:
{
  "thoughts": "Need to create a stats parser. Plan first, then read code.",
  "actions": [
    {"type": "plan", "todos": "1. Read prd.json\n2. Create script\n3. Test\n4. Commit\n5. Finish"},
    {"type": "read_file", "path": "prd.json"}
  ]
}

WORKFLOW:
1. Read prd.json — find first story where passes is false
2. Read progress.txt, learnings.md, AGENTS.md for context
3. Plan your approach BEFORE implementing
4. Implement with minimal, focused changes
5. Verify changes work (read file back, run tests)
6. Git commit: run_command "git add -A && git commit -m feat-STORYID-title"
7. Call finish — triggers verificationCommand
8. Append summary to progress.txt
9. Update AGENTS.md with patterns/gotchas

MEMORY FILES (read these for context, update after stories):
- progress.txt: progress log
- learnings.md: append-only insights (what worked, what failed)
- AGENTS.md: knowledge base (patterns, gotchas, conventions)

PROTECTED FILES (NEVER edit):
- .env, .git/*, .gitignore

GIT RULES (MANDATORY):
- Before .sh edits: run_command "git add -A && git commit -m checkpoint-before-edit"
- BEFORE calling finish: run_command "git add -A && git commit -m feat-STORYID-description"
- NEVER call finish without committing first.

SELF-MODIFICATION (ralph.sh):
- Edits go to staging copy, validated with bash -n
- ALWAYS commit before editing
- Make SMALL incremental changes
SYSPROMPT

  TOOL="llm_direct_nvidia"
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 5: Validate Tool
# ═══════════════════════════════════════════════════════════════

if [[ "$TOOL" != "amp" && "$TOOL" != "claude" && "$TOOL" != "llm_direct_nvidia" ]]; then
  echo "Error: Invalid tool '$TOOL'."; exit 1
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 6: Paths + Constants
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/ralph-debug.log"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"
LEARNINGS_FILE="$SCRIPT_DIR/learnings.md"
AGENTS_FILE="$SCRIPT_DIR/AGENTS.md"
STEERING_FILE="$SCRIPT_DIR/STEERING.md"
CHECKPOINT_FILE="$SCRIPT_DIR/.checkpoint_iter"

SCRIPT_START_SECONDS=$SECONDS
CURRENT_ITERATION=0

MAX_CONTEXT_TOKENS=${MAX_CONTEXT_TOKENS:-200000}
CONTEXT_WARN_THRESHOLD=70
CONTEXT_SHRINK_THRESHOLD=80
CONSECUTIVE_API_ERRORS=0

STUCK_THRESHOLD=3
SKIP_THRESHOLD=7

LLM_FALLBACK_MODELS=(
  "abacusai/dracarys-llama-3.1-70b-instruct"
  "microsoft/phi-3-medium-128k-instruct"
  "microsoft/phi-3-mini-128k-instruct"
)

# ═══════════════════════════════════════════════════════════════
# SECTION 7: Helper Functions
# ═══════════════════════════════════════════════════════════════

log_action() {
  echo "$(date '+%Y-%m-%d %H:%M:%S') | $1 | $2 | $3" >> "$LOG_FILE"
}

debug_log() {
  [[ "${RALPH_DEBUG:-0}" == "1" ]] && echo "DEBUG: $*" >&2
}

# Status lines go to stderr so they're visible inside $() captures
status_line() {
  local icon="$1" action="$2" detail="$3" result="$4"
  local prefix=""
  [[ -n "$CURRENT_ITERATION" ]] && prefix="[iter $CURRENT_ITERATION/$MAX_ITERATIONS] "
  echo "${prefix}${icon} ${action}: ${detail} ... ${result}" >&2
}

estimate_tokens() {
  echo $(( ${#1} / 4 ))
}

check_context_usage() {
  local history_json="$1"
  local hist_text
  hist_text=$(echo "$history_json" | jq -r '.[].content' 2>/dev/null)
  local total=$(( $(estimate_tokens "$LLM_SYSTEM_PROMPT") + $(estimate_tokens "$hist_text") ))
  local pct=$(( total * 100 / MAX_CONTEXT_TOKENS ))

  if (( pct >= CONTEXT_SHRINK_THRESHOLD )); then
    echo "CONTEXT: ⚠️ ${pct}% (~${total}/${MAX_CONTEXT_TOKENS} tokens) — shrinking" >&2
    echo "shrink"
  elif (( pct >= CONTEXT_WARN_THRESHOLD )); then
    echo "CONTEXT: ⚡ ${pct}% (~${total}/${MAX_CONTEXT_TOKENS} tokens)" >&2
    echo "warn"
  else
    echo "ok"
  fi
}

append_learning() {
  echo "- [$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LEARNINGS_FILE"
}

compute_error_hash() {
  echo "${#1}:${1:0:80}"
}

check_protected_file() {
  local filepath="$1" action="$2"
  case "$filepath" in
    .env|.env.*|.git|.git/*)
      echo "🛡️ BLOCKED: Cannot $action protected file: $filepath"
      return 1 ;;
    prd.json|./prd.json)
      echo "⚠️ WARNING: Direct edit of prd.json. Use 'finish' to mark stories." >&2
      return 0 ;;
  esac
  return 0
}

print_summary() {
  local total_seconds=$(( SECONDS - SCRIPT_START_SECONDS ))
  local minutes=$(( total_seconds / 60 ))
  local seconds=$(( total_seconds % 60 ))
  local stories_total=0 stories_passed=0

  if [ -f "$PRD_FILE" ]; then
    stories_total=$(jq '.userStories | length' "$PRD_FILE" 2>/dev/null || echo 0)
    stories_passed=$(jq '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE" 2>/dev/null || echo 0)
  fi

  # Bulletproof: count from debug log (immune to subshell issues)
  local llm_rounds=0
  if [ -f "$LOG_FILE" ]; then
    llm_rounds=$(grep -c "| llm_call |.*| ok" "$LOG_FILE" 2>/dev/null || echo 0)
  fi

  echo ""
  echo "═══════════════════════════════════════"
  echo "  Ralph v2.0 Summary"
  echo "───────────────────────────────────────"
  echo "  Stories:    ${stories_passed}/${stories_total} completed"
  echo "  Iterations: ${CURRENT_ITERATION:-?}/${MAX_ITERATIONS} used"
  echo "  LLM calls:  ${llm_rounds}"
  echo "  Total time: ${minutes}m ${seconds}s"
  echo "  Model:      ${LLM_MODEL:-N/A}"
  echo "═══════════════════════════════════════"
}

# ═══════════════════════════════════════════════════════════════
# SECTION 8: Initialize Memory + Progress
# ═══════════════════════════════════════════════════════════════

if [ ! -f "$LEARNINGS_FILE" ]; then
  cat > "$LEARNINGS_FILE" << 'EOF'
# Learnings Log
# Append-only insights from each Ralph iteration
---
EOF
  debug_log "Created $LEARNINGS_FILE"
fi

if [ ! -f "$AGENTS_FILE" ]; then
  cat > "$AGENTS_FILE" << 'EOF'
# AGENTS.md — Ralph Agent Knowledge Base

Read by Ralph at the start of each iteration.
Updated with discovered patterns, gotchas, conventions.

## Discovered Patterns

## Known Gotchas

## Project Conventions
EOF
  debug_log "Created $AGENTS_FILE"
fi

if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 9: Archive Previous Run if Branch Changed
# ═══════════════════════════════════════════════════════════════

if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
  _cur_branch=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  _last_branch=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")

  if [[ -n "$_cur_branch" && -n "$_last_branch" && "$_cur_branch" != "$_last_branch" ]]; then
    _folder="$ARCHIVE_DIR/$(date +%Y-%m-%d)-$(echo "$_last_branch" | sed 's|^ralph/||')"
    echo "Archiving previous run: $_last_branch → $_folder"
    mkdir -p "$_folder"
    [ -f "$PRD_FILE" ] && cp "$PRD_FILE" "$_folder/"
    [ -f "$PROGRESS_FILE" ] && cp "$PROGRESS_FILE" "$_folder/"
    echo "# Ralph Progress Log" > "$PROGRESS_FILE"
    echo "Started: $(date)" >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"
  fi
fi

if [ -f "$PRD_FILE" ]; then
  _cur_branch=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  [[ -n "$_cur_branch" ]] && echo "$_cur_branch" > "$LAST_BRANCH_FILE"
fi

# ═══════════════════════════════════════════════════════════════
# SECTION 10: Banner
# ═══════════════════════════════════════════════════════════════

echo ""
echo "═══════════════════════════════════════"
echo "  Ralph v2.0 — Tool: $TOOL"
echo "  Max iterations: $MAX_ITERATIONS"
[[ "$TOOL" == "llm_direct_nvidia" ]] && echo "  Model: $LLM_MODEL"
[[ "${RALPH_DEBUG:-0}" == "1" ]] && echo "  Debug: ON"
echo "═══════════════════════════════════════"
echo ""

# ═══════════════════════════════════════════════════════════════
# SECTION 11: LLM API Call
# ═══════════════════════════════════════════════════════════════

llm_call_nvidia() {
  local prompt="$1"
  local history="$2"

  local messages
  messages=$(jq -n --arg sys "$LLM_SYSTEM_PROMPT" \
    --argjson hist "$history" \
    --arg prompt "$prompt" \
    '[{role: "system", content: $sys}] + $hist + [{role: "user", content: $prompt}]')

  local api_body
  api_body=$(jq -n \
    --arg model "$LLM_MODEL" \
    --argjson msgs "$messages" \
    --argjson temp "${LLM_TEMPERATURE}" \
    --argjson max_tokens 8192 \
    '{model: $model, messages: $msgs, temperature: $temp, max_tokens: $max_tokens, stream: false}')

  local retry api_result backoff=5
  for retry in 1 2 3; do
    local call_start=$SECONDS

    api_result=$(curl -s --max-time 120 -X POST "${LLM_BASE_URL}/chat/completions" \
      -H "Authorization: Bearer $LLM_API_KEY" \
      -H "Content-Type: application/json" \
      -d "$api_body")

    local call_dur=$(( SECONDS - call_start ))

    # Log every API call to file (bulletproof counter)
    log_action "llm_call" "model=$LLM_MODEL attempt=$retry dur=${call_dur}s" "sent"

    # Check for API error
    local api_error
    api_error=$(echo "$api_result" | jq -r '.error.message // empty' 2>/dev/null)
    if [[ -n "$api_error" ]]; then
      debug_log "API error (attempt $retry): $api_error"
      log_action "llm_call" "error: $api_error" "fail"
      (( CONSECUTIVE_API_ERRORS++ )) || true

      # Fallback model after 3 consecutive errors
      if (( CONSECUTIVE_API_ERRORS >= 3 )); then
        for fb_model in "${LLM_FALLBACK_MODELS[@]}"; do
          if [[ "$fb_model" != "$LLM_MODEL" ]]; then
            status_line "🔄" "fallback" "$LLM_MODEL → $fb_model" "after $CONSECUTIVE_API_ERRORS errors"
            append_learning "Model fallback: $LLM_MODEL → $fb_model after $CONSECUTIVE_API_ERRORS errors"
            LLM_MODEL="$fb_model"
            CONSECUTIVE_API_ERRORS=0
            api_body=$(jq -n \
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

    # Check for valid response
    if echo "$api_result" | jq -e '.choices[0].message.content' >/dev/null 2>&1; then
      CONSECUTIVE_API_ERRORS=0
      log_action "llm_call" "model=$LLM_MODEL dur=${call_dur}s" "ok"
      echo "$api_result"
      return 0
    fi

    debug_log "Attempt $retry failed, retry in ${backoff}s"
    sleep "$backoff"
    backoff=$(( backoff * 3 ))
  done

  echo "$api_result"
}

# ═══════════════════════════════════════════════════════════════
# SECTION 12: Execute Actions
# ═══════════════════════════════════════════════════════════════

execute_nvidia_actions() {
  local actions_json="$1"
  local action_count=0

  while read -r action; do
    local type
    type=$(echo "$action" | jq -r '.type // empty')
    (( action_count++ )) || true
    debug_log "Action #$action_count: $type"

    case "$type" in

      "plan"|"write_todos")
        local todos
        todos=$(echo "$action" | jq -r '.todos // .plan // empty')
        echo "📋 PLAN:"
        echo "$todos"
        status_line "📋" "plan" "recorded" "OK"
        log_action "plan" "${todos:0:80}" "ok"
        ;;

      "list_files")
        local path
        path=$(echo "$action" | jq -r '.path // "."')
        if [[ -d "$path" ]]; then
          ls -la "$path"
          status_line "📂" "list_files" "$path" "OK"
          log_action "list_files" "$path" "ok"
        else
          echo "ERROR [list_files]: Not found: $path"
          status_line "📂" "list_files" "$path" "FAIL"
          log_action "list_files" "$path" "fail"
        fi
        ;;

      "read_file")
        local path
        path=$(echo "$action" | jq -r '.path // empty')
        local line_start line_end
        line_start=$(echo "$action" | jq -r '.line_start // empty')
        line_end=$(echo "$action" | jq -r '.line_end // empty')

        # Sandbox: staged ralph.sh
        if [[ ("$path" == "ralph.sh" || "$path" == "./ralph.sh") && -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
          path="$SCRIPT_DIR/.staging/ralph.sh"
          echo "SANDBOX: Reading staged version"
        fi

        if [[ -n "$path" && -f "$path" ]]; then
          local total_lines
          total_lines=$(wc -l < "$path")
          if [[ -n "$line_start" && -n "$line_end" ]]; then
            sed -n "${line_start},${line_end}p" "$path"
            echo "[Lines $line_start-$line_end of $total_lines]"
          elif (( total_lines > 200 )); then
            head -n 50 "$path"
            echo ""
            echo "[... $(( total_lines - 100 )) lines omitted ...]"
            echo "[Use line_start/line_end to read specific sections]"
            echo ""
            tail -n 50 "$path"
          else
            cat "$path"
          fi
          status_line "📖" "read_file" "$path ($total_lines lines)" "OK"
          log_action "read_file" "$path" "ok"
        else
          echo "ERROR [read_file]: Not found: $path"
          status_line "📖" "read_file" "$path" "FAIL"
          log_action "read_file" "$path" "fail"
        fi
        ;;

      "edit_file")
        local path
        path=$(echo "$action" | jq -r '.path // empty')
        local changes
        changes=$(echo "$action" | jq -c '.changes // empty')

        if [[ -z "$path" || -z "$changes" || "$changes" == "null" ]]; then
          echo "ERROR [edit_file]: Missing path or changes"
          log_action "edit_file" "$path" "fail"
          continue
        fi

        # Guardrail
        if ! check_protected_file "$path" "edit_file"; then
          log_action "edit_file" "BLOCKED $path" "fail"
          continue
        fi

        # Sandbox for .sh files
        if [[ "$path" == "ralph.sh" || "$path" == "./ralph.sh" ]]; then
          mkdir -p "$SCRIPT_DIR/.staging"
          if [[ ! -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
            cp "$SCRIPT_DIR/ralph.sh" "$SCRIPT_DIR/.staging/ralph.sh"
            echo "SANDBOX: Created staging copy"
          fi
          path="$SCRIPT_DIR/.staging/ralph.sh"
          echo "SANDBOX: Editing staging copy"
        fi

        echo "$changes" | jq -c '.[]' | while read -r change; do
          local old_str new_str
          old_str=$(echo "$change" | jq -r '.oldString // empty')
          new_str=$(echo "$change" | jq -r '.newString // empty')
          if [[ -n "$old_str" && -f "$path" ]]; then
            _EP="$path" _EO="$old_str" _EN="$new_str" python3 -c \
              "import os;p=os.environ['_EP'];c=open(p).read().replace(os.environ['_EO'],os.environ['_EN'],1);open(p,'w').write(c)"
            status_line "✏️" "edit_file" "$path" "OK"
            log_action "edit_file" "$path" "ok"
          else
            echo "WARNING [edit_file]: Invalid change in $path"
            log_action "edit_file" "$path" "fail"
          fi
        done
        ;;

      "create_file")
        local path content
        path=$(echo "$action" | jq -r '.path // .filename // empty')
        content=$(echo "$action" | jq -r '.content // empty')

        if [[ -z "$path" ]]; then
          echo "ERROR [create_file]: Missing path"; log_action "create_file" "?" "fail"; continue
        fi
        if [[ -z "$content" ]]; then
          echo "ERROR [create_file]: Missing content"; log_action "create_file" "$path" "fail"; continue
        fi

        if ! check_protected_file "$path" "create_file"; then
          log_action "create_file" "BLOCKED $path" "fail"; continue
        fi

        mkdir -p "$(dirname "$path")"
        printf '%s\n' "$content" > "$path"
        status_line "📝" "create_file" "$path" "OK"
        log_action "create_file" "$path" "ok"
        ;;

      "run_command")
        local command
        command=$(echo "$action" | jq -r '.command // empty')
        if [[ -z "$command" ]]; then
          echo "ERROR [run_command]: Empty command"; log_action "run_command" "?" "fail"; continue
        fi

        debug_log "Running: $command"
        local cmd_start=$SECONDS output exit_code
        output=$(cd "$SCRIPT_DIR" && bash -c "$command" 2>&1) || true
        exit_code=$?
        local cmd_dur=$(( SECONDS - cmd_start ))

        echo "$output"
        if [[ $exit_code -ne 0 ]]; then
          status_line "🏃" "run_command" "$command" "FAIL (exit $exit_code, ${cmd_dur}s)"
          log_action "run_command" "$command" "fail:$exit_code"
        else
          status_line "🏃" "run_command" "$command" "OK (${cmd_dur}s)"
          log_action "run_command" "$command" "ok"
        fi
        ;;

      "finish")
        log_action "finish" "start" "pending"

        # Sandbox validation
        if [[ -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
          echo "SANDBOX: Validating staged ralph.sh..."
          local sandbox_err
          sandbox_err=$(bash -n "$SCRIPT_DIR/.staging/ralph.sh" 2>&1)
          if [[ $? -eq 0 ]]; then
            echo "SANDBOX: ✅ Syntax OK — applying"
            diff "$SCRIPT_DIR/ralph.sh" "$SCRIPT_DIR/.staging/ralph.sh" || true
            cp "$SCRIPT_DIR/.staging/ralph.sh" "$SCRIPT_DIR/ralph.sh"
            rm -rf "$SCRIPT_DIR/.staging"
          else
            echo "SANDBOX: ❌ SYNTAX ERROR — REJECTED: $sandbox_err"
            rm -rf "$SCRIPT_DIR/.staging"
            log_action "finish" "sandbox rejected" "fail"
            continue
          fi
        fi

        # Get current failing story
        local sid
        sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$SCRIPT_DIR/prd.json" 2>/dev/null)

        local story_verified=false

        if [[ -n "$sid" && "$sid" != "null" ]]; then
          # Check for verificationCommand
          local vcmd
          vcmd=$(jq -r --arg id "$sid" \
            '.userStories[] | select(.id == $id) | .verificationCommand // empty' \
            "$SCRIPT_DIR/prd.json" 2>/dev/null)

          if [[ -n "$vcmd" && "$vcmd" != "null" ]]; then
            echo "VERIFY: Running for story $sid: $vcmd"

            # Rejection counter
            [[ ! -f "$SCRIPT_DIR/.rejection_count" ]] && echo 0 > "$SCRIPT_DIR/.rejection_count"
            local rejections
            rejections=$(cat "$SCRIPT_DIR/.rejection_count")

            local verify_output verify_exit
            verify_output=$(cd "$SCRIPT_DIR" && bash -c "$vcmd" 2>&1) || true
            verify_exit=$?

            if [[ $verify_exit -eq 0 ]]; then
              echo "VERIFY: ✅ PASSED — Story $sid" | tee /dev/stderr
              echo "$verify_output"
              echo 0 > "$SCRIPT_DIR/.rejection_count"
              story_verified=true
              append_learning "Story $sid PASSED verification"
            else
              local new_rej=$(( rejections + 1 ))
              echo "$new_rej" > "$SCRIPT_DIR/.rejection_count"
              echo "VERIFY: ❌ FAILED (exit $verify_exit, rejection $new_rej)" | tee /dev/stderr
              echo "VERIFY OUTPUT:"
              echo "$verify_output"

              if (( new_rej >= 5 )); then
                echo "VERIFY: 🚨 CRITICAL — $new_rej failures. Try a DIFFERENT approach."
              fi

              append_learning "Story $sid FAILED verification (attempt $new_rej)"
              log_action "finish" "verification failed" "fail"
              continue
            fi
          else
            echo "VERIFY: No verificationCommand — trusting agent"
            story_verified=true
          fi

          # Single code path for marking passed
          if [[ "$story_verified" == "true" ]]; then
            local tmp
            tmp=$(mktemp)
            jq --arg id "$sid" \
              '(.userStories[] | select(.id == $id)).passes = true' \
              "$SCRIPT_DIR/prd.json" > "$tmp" && mv "$tmp" "$SCRIPT_DIR/prd.json"
            echo "✅ Story $sid marked as PASSED" | tee /dev/stderr
            echo 0 > "$SCRIPT_DIR/.rejection_count" 2>/dev/null || true
          fi
        else
          echo "FINISH: No failing stories — all done"
        fi

        # Check remaining
        local remaining
        remaining=$(jq '[.userStories[] | select(.passes == false)] | length' "$SCRIPT_DIR/prd.json" 2>/dev/null || echo 0)

        if (( remaining == 0 )); then
          echo "<promise>COMPLETE</promise>"
        else
          echo "STORY_COMPLETED: $sid ($remaining remaining)"
          echo "<promise>STORY_COMPLETED</promise>"
        fi
        log_action "finish" "story=$sid remaining=$remaining" "ok"
        return 0
        ;;

      *)
        echo "WARNING: Unknown action: $type"
        log_action "unknown" "$type" "skip"
        ;;
    esac

  done < <(echo "$actions_json" | jq -c '.actions[]')

  debug_log "Executed $action_count actions"
}

# ═══════════════════════════════════════════════════════════════
# SECTION 13: Construct Prompt (Context Engineering)
# ═══════════════════════════════════════════════════════════════

construct_nvidia_prompt() {
  local context_json="$1"

  # STEERING.md — live human override (highest priority)
  if [ -f "$STEERING_FILE" ] && [ -s "$STEERING_FILE" ]; then
    echo "═══ STEERING (PRIORITY — follow first) ═══"
    cat "$STEERING_FILE"
    echo "═══ END STEERING ═══"
    echo ""
  fi

  echo "═══ CURRENT STORY ═══"
  echo "$context_json"
  echo ""

  # AGENTS.md — knowledge base
  if [ -f "$AGENTS_FILE" ]; then
    local agents_size
    agents_size=$(wc -c < "$AGENTS_FILE")
    if (( agents_size < 3000 )); then
      echo "═══ AGENT KNOWLEDGE (AGENTS.md) ═══"
      cat "$AGENTS_FILE"
    else
      echo "═══ AGENT KNOWLEDGE (last 30 lines) ═══"
      tail -n 30 "$AGENTS_FILE"
    fi
    echo "═══ END AGENTS.md ═══"
    echo ""
  fi

  # Recent learnings
  if [ -f "$LEARNINGS_FILE" ]; then
    local learn_lines
    learn_lines=$(wc -l < "$LEARNINGS_FILE")
    if (( learn_lines > 5 )); then
      echo "═══ RECENT LEARNINGS (last 10) ═══"
      tail -n 10 "$LEARNINGS_FILE"
      echo "═══ END LEARNINGS ═══"
      echo ""
    fi
  fi

  echo "═══ RECENT PROGRESS ═══"
  tail -n 8 "$PROGRESS_FILE" 2>/dev/null || echo "(none)"
  echo ""

  echo "═══ PROJECT FILES ═══"
  find "$SCRIPT_DIR" -maxdepth 2 -type f \
    \( -name "*.sh" -o -name "*.json" -o -name "*.md" -o -name "*.txt" -o -name "*.py" \) \
    ! -path "*/.git/*" ! -path "*/node_modules/*" ! -path "*/archive/*" \
    2>/dev/null | sort | head -30
  echo ""

  echo "═══ GIT STATUS ═══"
  cd "$SCRIPT_DIR" && git status --short 2>/dev/null | head -10
  echo ""
  echo "═══ LAST 3 COMMITS ═══"
  cd "$SCRIPT_DIR" && git log --oneline -3 2>/dev/null || echo "(no git)"
  echo ""
}

# ═══════════════════════════════════════════════════════════════
# SECTION 14: Get Story Context
# ═══════════════════════════════════════════════════════════════

get_story_context() {
  if [ ! -f "$SCRIPT_DIR/prd.json" ]; then
    echo '{"error": "PRD not found"}'
    return
  fi

  local story
  story=$(jq -c '[.userStories[] | select(.passes == false)][0]' "$SCRIPT_DIR/prd.json" 2>/dev/null)

  if [[ -z "$story" || "$story" == "null" ]]; then
    echo "{}"
  else
    echo "$story" | jq -c '{id, title, description, acceptanceCriteria, verificationCommand, branchName}'
  fi
}

# ═══════════════════════════════════════════════════════════════
# SECTION 15: Main Agent Loop
# ═══════════════════════════════════════════════════════════════

run_nvidia_agent_loop() {
  local conversation_history='[]'
  local iter=0
  local json_fails=0
  local stuck_count=0
  local stuck_hash=""

  debug_log "Starting agent loop"

  while true; do
    debug_log "Agent loop iteration $iter"

    local context
    context=$(get_story_context)

    # No more stories
    if [[ "$context" == "{}" || -z "$context" ]]; then
      echo "All stories completed!"
      echo "<promise>COMPLETE</promise>"
      return 0
    fi

    local story_id story_title
    story_id=$(echo "$context" | jq -r '.id // "?"')
    story_title=$(echo "$context" | jq -r '.title // "?"')
    (( iter % 5 == 0 )) && echo "📌 Working on: [$story_id] $story_title (inner iter $iter)"

    # Build prompt
    local full_prompt
    full_prompt=$(construct_nvidia_prompt "$context")

    # Call LLM
    local llm_response
    llm_response=$(llm_call_nvidia "$full_prompt" "$conversation_history")

    # Extract content
    local llm_content
    llm_content=$(echo "$llm_response" | jq -r \
      '(.choices[0].message.content // .choices[0].message.reasoning_content // .choices[0].message.reasoning) // empty' \
      2>/dev/null) || true

    if [[ -z "$llm_content" || "$llm_content" == "null" ]]; then
      echo "ERROR: Empty LLM response" >&2
      if (( iter < 3 )); then
        echo "Retrying..." >&2
        (( iter++ )) || true
        sleep 5
        continue
      fi
      return 1
    fi

    debug_log "LLM content length: ${#llm_content}"

    # Strip markdown fences
    llm_content=$(echo "$llm_content" | sed '/^```[a-z]*$/d' | sed '/^```$/d')

    # Parse JSON
    local actions_json
    actions_json=$(echo "$llm_content" | jq -e '.' 2>/dev/null) || true

    if [[ -z "$actions_json" || "$actions_json" == "null" ]]; then
      (( json_fails++ )) || true

      if (( json_fails >= 3 )); then
        echo "ERROR: $json_fails consecutive JSON failures" >&2
        append_learning "JSON failures: $json_fails consecutive. Model: $LLM_MODEL"
        return 1
      fi

      local recovery_msg='ERROR: Invalid JSON. Respond with ONLY: {"thoughts":"...","actions":[{"type":"read_file","path":"prd.json"}]}'
      conversation_history=$(echo "$conversation_history" | jq \
        --arg c "$llm_content" '. + [{"role":"assistant","content":$c}]')
      conversation_history=$(echo "$conversation_history" | jq \
        --arg m "$recovery_msg" '. + [{"role":"user","content":$m}]')
      (( iter++ )) || true
      continue
    else
      json_fails=0
    fi

    # Add to history
    conversation_history=$(echo "$conversation_history" | jq \
      --arg c "$llm_content" '. + [{"role":"assistant","content":$c}]')

    # Execute
    local exec_result
    exec_result=$(execute_nvidia_actions "$actions_json") || true

    # Add result to history
    conversation_history=$(echo "$conversation_history" | jq \
      --arg r "$exec_result" '. + [{"role":"user","content":("Result:\n" + $r)}]')

    # Check completion signals
    if echo "$exec_result" | tail -5 | grep -qF "<promise>COMPLETE</promise>"; then
      append_learning "All stories completed"
      echo "<promise>COMPLETE</promise>"
      return 0
    elif echo "$exec_result" | tail -5 | grep -qF "<promise>STORY_COMPLETED</promise>"; then
      echo "Moving to next story..."
      append_learning "Story $story_id completed"
      conversation_history='[]'
      stuck_count=0; stuck_hash=""
      (( iter++ )) || true
      continue
    fi

    # Stuck detection
    if echo "$exec_result" | grep -qiE "FAIL|ERROR|REJECTED"; then
      local err_text err_hash
      err_text=$(echo "$exec_result" | grep -iE "FAIL|ERROR|REJECTED" | head -3)
      err_hash=$(compute_error_hash "$err_text")

      if [[ "$err_hash" == "$stuck_hash" ]]; then
        (( stuck_count++ )) || true
      else
        stuck_count=1; stuck_hash="$err_hash"
      fi

      if (( stuck_count >= SKIP_THRESHOLD )); then
        echo "🚫 SKIP: Story $story_id after $stuck_count repeated errors" >&2
        append_learning "SKIPPED $story_id: stuck $stuck_count times on: $err_text"
        conversation_history='[]'
        stuck_count=0; stuck_hash=""
        (( iter++ )) || true
        continue

      elif (( stuck_count >= STUCK_THRESHOLD )); then
        local reflection="⚠️ STUCK for $stuck_count iterations. Error: $err_text
Explain WHY this keeps happening. Propose a DIFFERENT approach."

        conversation_history=$(echo "$conversation_history" | jq \
          --arg m "$reflection" '. + [{"role":"user","content":$m}]')
        echo "🔄 Self-reflection injected (stuck $stuck_count)" >&2
      fi
    else
      stuck_count=0; stuck_hash=""
    fi

    # Context management
    local ctx_status hist_len
    ctx_status=$(check_context_usage "$conversation_history")
    hist_len=$(echo "$conversation_history" | jq 'length')

    if [[ "$ctx_status" == "shrink" ]]; then
      (( hist_len > 6 )) && conversation_history=$(echo "$conversation_history" | jq '.[-6:]')
    elif (( hist_len > 10 )); then
      conversation_history=$(echo "$conversation_history" | jq '.[-10:]')
    fi

    # Max inner iterations
    if (( iter++ > 40 )); then
      echo "ERROR: Max inner iterations (40) reached" >&2
      append_learning "Hit 40-iteration limit on $story_id"
      return 1
    fi
  done
}

# ═══════════════════════════════════════════════════════════════
# SECTION 16: Outer Loop
# ═══════════════════════════════════════════════════════════════

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
    _OUT_FILE=$(mktemp)
    run_nvidia_agent_loop 2>&1 | tee "$_OUT_FILE"
    OUTPUT=$(cat "$_OUT_FILE")
    rm -f "$_OUT_FILE"
  fi

  if echo "$OUTPUT" | grep -qF "<promise>COMPLETE</promise>"; then
    print_summary

    if [ -f "$PRD_FILE" ]; then
      _branch=$(jq -r '.branchName // "completed"' "$PRD_FILE" 2>/dev/null)
      _dir="$ARCHIVE_DIR/$(date +%Y-%m-%d)-$(echo "$_branch" | sed 's|^ralph/||')"
      echo "Archiving to: $_dir"
      mkdir -p "$_dir"
      [ -f "$PRD_FILE" ] && mv "$PRD_FILE" "$_dir/"
      [ -f "$PROGRESS_FILE" ] && mv "$PROGRESS_FILE" "$_dir/"
      [ -f "$LAST_BRANCH_FILE" ] && mv "$LAST_BRANCH_FILE" "$_dir/"
      [ -f "$SCRIPT_DIR/.rejection_count" ] && mv "$SCRIPT_DIR/.rejection_count" "$_dir/"
      [ -f "$CHECKPOINT_FILE" ] && mv "$CHECKPOINT_FILE" "$_dir/"
      echo "Archived. learnings.md and AGENTS.md preserved."
    fi

    exit 0
  fi

  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS)."
print_summary
exit 1