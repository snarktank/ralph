#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude] [--llm-provider nvidia --llm-api-key KEY ...] [max_iterations]

set -e

# Check for required dependencies
if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed. Please install jq and try again."
  echo "On Windows with Git Binary, you can download from: https://github.com/jqlang/jq/releases"
  exit 1
fi
if ! command -v curl &> /dev/null; then
  echo "Error: curl is required but not installed. Please install curl and try again."
  exit 1
fi

# Parse arguments
TOOL="amp"  # Default to amp for backwards compatibility
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
    *)
      # Assume it's max_iterations if it's a number
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      fi
      shift
      ;;
  esac
done

# Load .env file for development convenience (NOT for production)
# SECURITY: .env files MUST NOT be committed to git
if [ -f ".env" ]; then
  # shellcheck disable=SC1091
  . .env
fi

# Validate and set defaults for NVIDIA provider
if [[ -n "$LLM_PROVIDER" ]]; then
  if [[ "$LLM_PROVIDER" != "nvidia" ]]; then
    echo "Error: Only '--llm-provider nvidia' is currently supported."
    echo "To use Amp or Claude Code, omit --llm-provider or use --tool amp/claude."
    exit 1
  fi
  
  # Require API key (can be from command line, NVIDIA_API_KEY env, LLM_API_KEY env, or .env)
  if [[ -z "$LLM_API_KEY" ]]; then
    # Try to get from environment variables
    LLM_API_KEY="${NVIDIA_API_KEY:-${LLM_API_KEY}}"
  fi
  
  if [[ -z "$LLM_API_KEY" ]]; then
    echo "Error: --llm-api-key is required when using --llm-provider nvidia"
    echo "You can also set NVIDIA_API_KEY environment variable or use .env file"
    exit 1
  fi
  
  # Set NVIDIA-specific defaults
  # Using a model known to work from our testing
  LLM_BASE_URL="${LLM_BASE_URL:-https://integrate.api.nvidia.com/v1}"
  LLM_MODEL="${LLM_MODEL:-abacusai/dracarys-llama-3.1-70b-instruct}"
  LLM_SYSTEM_PROMPT="You are an autonomous coding agent implementing user stories from a PRD. You MUST respond with ONLY a valid JSON object and nothing else. The JSON must have exactly these two fields:
1. 'thoughts': a string explaining your reasoning
2. 'actions': an array of action objects to execute

Each action object MUST have a 'type' field. Supported types:
- read_file: { \"type\": \"read_file\", \"path\": \"relative/path/file.ext\", \"line_start\": 1, \"line_end\": 50 } // line_start/end are optional for chunking
- edit_file: { \"type\": \"edit_file\", \"path\": \"relative/path/file.ext\", \"changes\": [ {\"oldString\": \"exact text to find\", \"newString\": \"replacement text\"} ] }
  (Alternative format also accepted: { \"type\": \"create_file\", \"path\": \"relative/path/file.ext\", \"content\": \"file content\" } )
- run_command: { \"type\": \"run_command\", \"command\": \"shell command to execute\", \"expects_success\": true/false }
- finish: { \"type\": \"finish\" }  // Use when the story is completely implemented

IMPORTANT: The PRD is located at \"prd.json\" and the progress log is located at \"progress.txt\" in the project root. Use these exact file names when referring to these files.

DO NOT include any text outside the JSON object. DO NOT include explanations, markdown formatting, or additional fields. Your entire response must be parseable as JSON.

WORKFLOW RULES:
1. Read prd.json to find the highest priority story where passes is false
2. Read progress.txt for context from previous iterations
3. Implement the story with minimal, focused changes
4. When modifying .sh files, ALWAYS run_command git add -A and git commit BEFORE making changes (safety net)
5. After implementing, verify your changes work (read the file back, run tests if applicable)
6. When story is complete, use edit_file to update prd.json and set passes to true for that story
7. Append a summary to progress.txt describing what you did and what you learned
8. Use finish action to signal completion

GIT RULES:
- Before modifying any .sh file: git add -A && git commit -m checkpoint-before-edit
- After completing a story: git add -A && git commit -m feat-STORYID-title
- Use run_command for all git operations

SELF-MODIFICATION RULES (SANDBOX):
- You CAN read and modify ralph.sh - edits go to a staging copy automatically
- On finish, staging is validated with bash -n before applying
- If syntax is broken, changes are REJECTED and you must fix them
- Make SMALL, incremental changes per action - one edit_file per logical change
- Do NOT try to rewrite entire functions in one edit - split into multiple actions
- After editing, use read_file to verify your staged changes look correct
- ALWAYS commit before modifying ralph.sh: run_command git add -A && git commit -m checkpoint"
  
  # Trigger direct LLM path
  TOOL="llm_direct_nvidia"
fi

# Validate tool choice (after potential override by LLM provider)
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" && "$TOOL" != "llm_direct_nvidia" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp', 'claude', or 'llm_direct_nvidia' (via --llm-provider nvidia)."
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/ralph-debug.log"
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"

# Global timer variables
SCRIPT_START_SECONDS=$SECONDS
TOTAL_API_CALLS=0
TOTAL_API_TIME=0
ITERATION_START_SECONDS=0

# Context monitoring (Faza 1)
MAX_CONTEXT_TOKENS=${MAX_CONTEXT_TOKENS:-32768}
CONTEXT_WARN_THRESHOLD=70  # warn at 70%
CONTEXT_SHRINK_THRESHOLD=80  # shrink history at 80%
CONSECUTIVE_API_ERRORS=0

# Fallback model list (Faza 2) — tried in order when primary fails
LLM_FALLBACK_MODELS=(
  "abacusai/dracarys-llama-3.1-70b-instruct"
  "microsoft/phi-3-medium-128k-instruct"
  "microsoft/phi-3-mini-128k-instruct"
)
LLM_ORIGINAL_MODEL="$LLM_MODEL"

# Estimate token count (~4 chars/token, rough heuristic)
estimate_tokens() {
  local text="$1"
  local char_count=${#text}
  echo $(( char_count / 4 ))
}

# Check context usage and warn/shrink
check_context_usage() {
  local history_json="$1"
  local history_text
  history_text=$(echo "$history_json" | jq -r '.[].content' 2>/dev/null)
  local sys_tokens
  sys_tokens=$(estimate_tokens "$LLM_SYSTEM_PROMPT")
  local hist_tokens
  hist_tokens=$(estimate_tokens "$history_text")
  local total_tokens=$(( sys_tokens + hist_tokens ))
  local usage_pct=$(( total_tokens * 100 / MAX_CONTEXT_TOKENS ))

  if (( usage_pct >= CONTEXT_SHRINK_THRESHOLD )); then
    echo "CONTEXT: ⚠️  ${usage_pct}% used (~${total_tokens}/${MAX_CONTEXT_TOKENS} tokens) — shrinking history" >&2
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
  local action_type="$1"
  local action_details="$2"
  local success="$3"
  local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "$timestamp - $action_type - $action_details - $success" >> "$LOG_FILE"
}

# Debug log — only prints when RALPH_DEBUG=1 (v1.4)
debug_log() {
  [[ "${RALPH_DEBUG:-0}" == "1" ]] && echo "DEBUG: $*" >&2
}

# Clean status line with emoji (v1.4)
status_line() {
  local icon="$1" action="$2" detail="$3" result="$4"
  local iter_prefix=""
  if [[ -n "$CURRENT_ITERATION" && -n "$MAX_ITERATIONS" ]]; then
    iter_prefix="[iter $CURRENT_ITERATION/$MAX_ITERATIONS] "
  fi
  echo "${iter_prefix}${icon} ${action}: ${detail} ... ${result}"
}

# Print end-of-run summary (v1.4)
print_summary() {
  local total_seconds=$(( SECONDS - SCRIPT_START_SECONDS ))
  local minutes=$(( total_seconds / 60 ))
  local seconds=$(( total_seconds % 60 ))
  local stories_total=0 stories_passed=0
  if [ -f "$PRD_FILE" ]; then
    stories_total=$(jq '.userStories | length' "$PRD_FILE" 2>/dev/null || echo 0)
    stories_passed=$(jq '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE" 2>/dev/null || echo 0)
  fi
  local avg_api="0"
  if (( TOTAL_API_CALLS > 0 )); then
    avg_api=$(( TOTAL_API_TIME / TOTAL_API_CALLS ))
  fi
  echo ""
  echo "═══════════════════════════════════════"
  echo "  Ralph Summary"
  echo "───────────────────────────────────────"
  echo "  Stories:    ${stories_passed}/${stories_total} completed"
  echo "  Iterations: ${CURRENT_ITERATION:-?}/${MAX_ITERATIONS} used"
  echo "  API calls:  ${TOTAL_API_CALLS} (avg ${avg_api}s)"
  echo "  Total time: ${minutes}m ${seconds}s"
  echo "  Model:      ${LLM_MODEL:-N/A}"
  echo "═══════════════════════════════════════"
}

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

echo ""
echo "═══════════════════════════════════════"
echo "  Ralph v1.4 — Tool: $TOOL"
echo "  Max iterations: $MAX_ITERATIONS"
[[ "$TOOL" == "llm_direct_nvidia" ]] && echo "  Model: $LLM_MODEL"
echo "═══════════════════════════════════════"
echo ""

# Global iteration tracker (for status_line)
CURRENT_ITERATION=0

# Helper function for NVIDIA API calls
llm_call_nvidia() {
  local prompt="$1"
  local history="$2"
  
  # Construct messages: system prompt as first message, then history, then current user prompt
  local messages=$(jq -n --arg sys "$LLM_SYSTEM_PROMPT" \
    --argjson hist "$history" \
    --arg prompt "$prompt" \
    '[{role: "system", content: $sys}] + $hist + [{role: "user", content: $prompt}]')
  
  # Call NVIDIA API with exponential backoff retry (Faza 2)
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
    (( TOTAL_API_CALLS++ )) || true
    (( TOTAL_API_TIME += _call_duration )) || true
    
    # Check for API error field (Faza 2 — walidacja odpowiedzi)
    local _api_error
    _api_error=$(echo "$_api_result" | jq -r '.error.message // empty' 2>/dev/null)
    if [[ -n "$_api_error" ]]; then
      debug_log "API error: $_api_error"
      (( CONSECUTIVE_API_ERRORS++ )) || true
      
      # Fallback model logic (Faza 2)
      if (( CONSECUTIVE_API_ERRORS >= 3 )); then
        for _fb_model in "${LLM_FALLBACK_MODELS[@]}"; do
          if [[ "$_fb_model" != "$LLM_MODEL" ]]; then
            status_line "🔄" "fallback" "$LLM_MODEL → $_fb_model" "after $CONSECUTIVE_API_ERRORS errors"
            LLM_MODEL="$_fb_model"
            CONSECUTIVE_API_ERRORS=0
            # Rebuild body with new model
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
    
    # Check if we got a valid response with content
    if echo "$_api_result" | jq -e '.choices[0].message.content' > /dev/null 2>&1; then
      CONSECUTIVE_API_ERRORS=0
      echo "$_api_result"
      return 0
    fi
    
    debug_log "API attempt $_retry failed or empty, retrying in ${_backoff_delay}s (exponential backoff)"
    sleep "$_backoff_delay"
    _backoff_delay=$(( _backoff_delay * 3 ))  # 5s → 15s → 45s
  done
  
  # Return whatever we got after 3 tries
  echo "$_api_result"
}

# Execute actions returned by the LLM
execute_nvidia_actions() {
  local actions_start_seconds=$SECONDS
  local actions_json="$1"
  local action_count=0
  log_action "start" "execute_nvidia_actions" "true"
  
  # Process each action (process substitution avoids subshell — return works correctly)
  while read -r action; do
    local type
    type=$(echo "$action" | jq -r '.type // empty')
    (( action_count++ )) || true
    debug_log "Action #$action_count: $type"
    
    case "$type" in
      "list_files")
        log_action "start" "$action" "false"
        local path
        path=$(echo "$action" | jq -r '.path // empty')
        debug_log "Listing files in: $path"
        if [[ -n "$path" && -d "$path" ]]; then
          ls -la "$path"
          status_line "📂" "list_files" "$path" "OK"
          log_action "end" "$action" "true"
        else
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
        # SANDBOX: if staged version exists, read that instead
        if [[ ("$path" == "ralph.sh" || "$path" == "./ralph.sh") && -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
          path="$SCRIPT_DIR/.staging/ralph.sh"
          echo "SANDBOX: Reading staged version of ralph.sh"
        fi
        debug_log "Reading file: $path"
        if [[ -n "$path" && -f "$path" ]]; then
          local total_lines
          total_lines=$(wc -l < "$path")
          if [[ -n "$line_start" && -n "$line_end" ]]; then
            # Specific line range requested
            sed -n "${line_start},${line_end}p" "$path"
            echo "[Read lines $line_start-$line_end of $total_lines total]"
          elif (( total_lines > 200 )); then
            # Auto-chunk: first 50 + last 50
            head -n 50 "$path"
            local omitted=$(( total_lines - 100 ))
            echo ""
            echo "[... $omitted lines omitted (lines 51-$(( total_lines - 50 ))) ...]"
            echo "[Use read_file with line_start and line_end to read specific sections]"
            echo ""
            tail -n 50 "$path"
            echo "[File chunked: $total_lines lines total, showing first 50 + last 50]"
          else
            cat "$path"
          fi
          local _fsize
          _fsize=$(wc -l < "$path" 2>/dev/null || echo "?")
          status_line "📖" "read_file" "$path (${_fsize} lines)" "OK"
          log_action "end" "$action" "true"
        else
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
        if [[ -z "$path" || -z "$changes" ]]; then
          echo "Error: Invalid edit_file action"
          log_action "end" "$action" "false"
          continue
        fi
        # SANDBOX: redirect ralph.sh edits to staging copy
        if [[ "$path" == "ralph.sh" || "$path" == "./ralph.sh" ]]; then
          mkdir -p "$SCRIPT_DIR/.staging"
          if [[ ! -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
            cp "$SCRIPT_DIR/ralph.sh" "$SCRIPT_DIR/.staging/ralph.sh"
            echo "SANDBOX: Created staging copy of ralph.sh"
          fi
          path="$SCRIPT_DIR/.staging/ralph.sh"
          echo "SANDBOX: Editing staging copy (not live code)"
        fi
        # Apply each change
        echo "$changes" | jq -c '.[]' | while read -r change; do
          local old_str new_str
          old_str=$(echo "$change" | jq -r '.oldString // empty')
          new_str=$(echo "$change" | jq -r '.newString // empty')
          if [[ -n "$old_str" && -n "$new_str" && -f "$path" ]]; then
            _EP="$path" _EO="$old_str" _EN="$new_str" python3 -c "import os;p=os.environ['_EP'];c=open(p).read().replace(os.environ['_EO'],os.environ['_EN'],1);open(p,'w').write(c)"
            status_line "✏️" "edit_file" "$path" "OK"
            log_action "end" "$action" "true"
          else
            echo "Warning: Skipping invalid change in $path"
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
          echo "Error: Invalid create_file action - missing path"
          log_action "end" "$action" "false"
          continue
        fi
        if [[ -z "$content" ]]; then
          echo "Error: Invalid create_file action - missing content"
          log_action "end" "$action" "false"
          continue
        fi
        # Create directory if needed
        mkdir -p "$(dirname "$path")"
        echo "$content" > "$path"
        status_line "📝" "create_file" "$path" "OK"
        log_action "end" "$action" "true"
        ;;
      "run_command")
        log_action "start" "$action" "false"
        local command expect_success
        command=$(echo "$action" | jq -r '.command // empty')
        expect_success=$(echo "$action" | jq -r '.expects_success // true')
        if [[ -n "$command" ]]; then
          debug_log "Running: $command"
          local _cmd_start=$SECONDS
          local output
          output=$(eval "$command" 2>&1)
          local exit_code=$?
          local _cmd_dur=$(( SECONDS - _cmd_start ))
          if [[ "$expect_success" == "true" && $exit_code -ne 0 ]]; then
            status_line "🏃" "run_command" "$command" "FAIL (exit $exit_code, ${_cmd_dur}s)"
            echo "$output"
            log_action "end" "$action" "false"
          else
            status_line "🏃" "run_command" "$command" "OK (${_cmd_dur}s)"
            debug_log "Output: $output"
            log_action "end" "$action" "true"
          fi
        fi
        ;;
      "finish")
        log_action "start" "$action" "false"
        # SANDBOX: validate staged ralph.sh before accepting
        if [[ -f "$SCRIPT_DIR/.staging/ralph.sh" ]]; then
          echo "SANDBOX: Validating staged ralph.sh with bash -n..."
          local _sandbox_err
          _sandbox_err=$(bash -n "$SCRIPT_DIR/.staging/ralph.sh" 2>&1)
          if [[ $? -eq 0 ]]; then
            echo "SANDBOX: ✅ Syntax OK — applying changes to live ralph.sh"
            echo "SANDBOX: Changes diff:"
            diff "$SCRIPT_DIR/ralph.sh" "$SCRIPT_DIR/.staging/ralph.sh" || true
            cp "$SCRIPT_DIR/.staging/ralph.sh" "$SCRIPT_DIR/ralph.sh"
            rm -rf "$SCRIPT_DIR/.staging"
          else
            echo "SANDBOX: ❌ SYNTAX ERROR — changes REJECTED"
            echo "SANDBOX: Error: $_sandbox_err"
            rm -rf "$SCRIPT_DIR/.staging"
            echo "Your changes to ralph.sh had syntax errors. Please fix and retry."
            log_action "end" "$action" "false"
            continue
          fi
        fi
        
        # Get current failing story
        _sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$SCRIPT_DIR/prd.json" 2>/dev/null)
        
        # Check if story has verificationCommand
        if [[ -n "$_sid" && "$_sid" != "null" ]]; then
          _verification_cmd=$(jq -r --arg id "$_sid" '.userStories[] | select(.id == $id) | .verificationCommand // empty' "$SCRIPT_DIR/prd.json" 2>/dev/null)
          
          if [[ -n "$_verification_cmd" && "$_verification_cmd" != "null" ]]; then
            echo "VERIFICATION: Running verification command for story $_sid:"
            echo "VERIFICATION: $_verification_cmd"
            
            # Initialize rejection counter if not exists
            if [[ ! -f "$SCRIPT_DIR/.rejection_count" ]]; then
              echo 0 > "$SCRIPT_DIR/.rejection_count"
            fi
            
            _current_rejections=$(cat "$SCRIPT_DIR/.rejection_count")
            
            # Run verification command
            if eval "$_verification_cmd"; then
              # Verification passed
              echo "VERIFICATION: ✅ PASSED - Story $_sid verified successfully"
              
              # Reset rejection counter on success
              echo 0 > "$SCRIPT_DIR/.rejection_count"
              
              # Mark story as passed
              _tmp=$(mktemp)
              jq --arg id "$_sid" '(.userStories[] | select(.id == $id)).passes = true' "$SCRIPT_DIR/prd.json" > "$_tmp" && mv "$_tmp" "$SCRIPT_DIR/prd.json"
              echo "Story $_sid marked as PASSED"
            else
              # Verification failed
              _exit_code=$?
              echo "VERIFICATION: ❌ FAILED - Story $_sid verification failed (exit $_exit_code)"
              
              # Increment rejection counter
              _new_rejections=$((_current_rejections + 1))
              echo $_new_rejections > "$SCRIPT_DIR/.rejection_count"
              
              if [[ $_new_rejections -ge 5 ]]; then
                echo "VERIFICATION: 🚨 CRITICAL HINT - 5 consecutive verification failures"
                echo "VERIFICATION: Consider re-examining your approach or breaking down the problem further"
              fi
              
              log_action "end" "$action" "false"
              continue
            fi
          else
            echo "VERIFICATION: No verificationCommand found for story $_sid - proceeding with completion"
          fi
          
          # Oznacz bieżące story jako ukończone
          _tmp=$(mktemp)
          jq --arg id "$_sid" '(.userStories[] | select(.id == $id)).passes = true' "$SCRIPT_DIR/prd.json" > "$_tmp" && mv "$_tmp" "$SCRIPT_DIR/prd.json"
          echo "Story $_sid marked as PASSED"
        else
          echo "VERIFICATION: No failing stories found - proceeding with completion"
        fi
        
        # Sprawdzamy pozostałe story
        _remaining=$(jq '[.userStories[] | select(.passes == false)] | length' "$SCRIPT_DIR/prd.json" 2>/dev/null || echo 0)
        
        if (( _remaining == 0 )); then
          echo "<promise>COMPLETE</promise>"
        else
          echo "STORY_COMPLETED: $_sid ($_remaining stories left)"
          echo "<promise>STORY_COMPLETED</promise>"
        fi
        log_action "end" "$action" "true"
        return 0
        ;;
      *)
        echo "Warning: Unknown action type: $type"
        log_action "end" "$action" "false"
        ;;
    esac
  done < <(echo "$actions_json" | jq -c '.actions[]')
  log_action "end" "execute_nvidia_actions" "true"
  local actions_duration=$(( SECONDS - actions_start_seconds ))
  debug_log "Executed $action_count actions in ${actions_duration}s"
}

# Construct prompt for NVIDIA LLM
construct_nvidia_prompt() {
  local context_json="$1"
  
  # Read instructions
  local instructions=$(cat <<EOF
You are an autonomous coding agent implementing user stories from a PRD.

Your response MUST be a valid JSON object with exactly these fields:
- "thoughts": string - Your reasoning about what to do next
- "actions": array - List of action objects to execute NOW

Supported action types:
1. read_file: { "type": "read_file", "path": "relative/path/file.ext" }
2. edit_file: { 
     "type": "edit_file", 
     "path": "relative/path/file.ext", 
     "changes": [ 
       { "oldString": "exact text to find", 
         "newString": "replacement text" } 
     ] 
   }
3. run_command: { 
     "type": "run_command", 
     "command": "shell command to execute", 
     "expects_success": true/false 
   }
4. finish: { "type": "finish" }  // Use when story is fully implemented

RULES:
- Output ONLY the JSON object - no extra text before or after
- Make minimal, focused changes
- If uncertain, gather information first with read_file actions
- ALWAYS end with a finish action when done

Current story context:
EOF
  )
  
  echo "$instructions"
  echo "$context_json"
  echo ""
  echo "Recent progress (last 5 lines):"
  tail -n 5 "$SCRIPT_DIR/progress.txt" 2>/dev/null || echo "(no progress yet)"
  echo ""
  echo "Project files:"
  find "$SCRIPT_DIR" -maxdepth 2 -type f \( -name "*.sh" -o -name "*.json" -o -name "*.md" -o -name "*.txt" \) ! -path "*/node_modules/*" ! -path "*/.git/*" ! -path "*/archive/*" 2>/dev/null | head -30
  echo ""
  echo "Git status:"
  cd "$SCRIPT_DIR" && git status --short 2>/dev/null | head -10
}

# Get highest priority story where passes: false
get_story_context() {
  local prd_file="$SCRIPT_DIR/prd.json"
  
  # Check if PRD file exists and is valid JSON
  if [ ! -f "$prd_file" ]; then
    echo '{"error": "PRD file not found"}'
    return
  fi
  
  # Get highest priority story where passes: false
  local story
  story=$(jq -c '[.userStories[] | select(.passes == false)][0]' "$prd_file" 2>/dev/null)
  
  if [[ -z "$story" || "$story" == "null" ]]; then
    # No story found, return empty object
    echo "{}"
    return
  fi
  
  # Format as JSON for prompt
  echo "$story" | jq -c '{id, title, description, acceptanceCriteria, branchName}'
}

# Main agent loop for NVIDIA LLM
run_nvidia_agent_loop() {
  local conversation_history='[]'  # JSON array of message objects
  local iteration_count=0
  local consecutive_json_fails=0
  
  debug_log "Starting agent loop"
  
  # Main agent loop
  while true; do
    debug_log "Entering agent loop iteration $iteration_count"
    # Get current story context
    local context=$(get_story_context)
    
    # Build prompt for LLM
    local full_prompt=$(construct_nvidia_prompt "$context")
    
    # Call NVIDIA API
    local llm_response=$(llm_call_nvidia "$full_prompt" "$conversation_history")
    
    # Extract content from response (handle API format)
    local llm_content
    if ! llm_content=$(echo "$llm_response" | jq -r '(.choices[0].message.content // .choices[0].message.reasoning_content // .choices[0].message.reasoning) // empty' 2>/dev/null); then
        echo "DEBUG: Failed to parse LLM response as JSON. Response was:" >&2
        echo "$llm_response" >&2
        echo "Saving response to /tmp/llm_response_debug.txt" >&2
        echo "$llm_response" > /tmp/llm_response_debug.txt
        return 1
    fi
    
    # Debug: show what we got
    debug_log "LLM response content length: ${#llm_content}"
    if [[ -n "$llm_content" && "$llm_content" != "null" ]]; then
      debug_log "LLM content (first 200 chars): ${llm_content:0:200}"
    else
      debug_log "LLM content is empty or null"
      debug_log "Full response: $llm_response"
    fi
    
    # If we got empty content, show error and break
    if [[ -z "$llm_content" || "$llm_content" == "null" ]]; then
      echo "Error: Empty response from NVIDIA API"
      echo "Response: $llm_response"
      return 1
    fi
    
    # Try to parse as JSON action - handle the case where LLM returns clean JSON
    local actions_json
    # Strip markdown code fences (Qwen3-coder wraps JSON in ```json blocks)
    llm_content=$(echo "$llm_content" | sed '/^```/d')

    actions_json=$(echo "$llm_content" | jq -e '.' 2>/dev/null) || true
    
    # If that failed, try to extract JSON from text (in case LLM wrapped it in markdown)
    if [[ -z "$actions_json" || "$actions_json" == "null" ]]; then
      # Look for JSON block in response
      actions_json=$(echo "$llm_content" | sed -n 's/.*```json\n\(.*\)\n```.*/\1/p')
      
      # If still not valid JSON, try to parse the content directly as JSON
      if [[ -z "$actions_json" || "$actions_json" == "null" ]]; then
        actions_json=$(echo "$llm_content" | jq -e '.' 2>/dev/null) || true
      fi
    fi
    
    # If still not valid JSON, treat as plain text (for debugging)
    if [[ -z "$actions_json" || "$actions_json" == "null" ]]; then
      (( consecutive_json_fails++ )) || true
      echo "Warning: LLM did not return valid JSON. Response:"
      echo "$llm_content"
      
      if (( consecutive_json_fails >= 3 )); then
        echo "Error: LLM repeatedly failed to generate valid JSON ($consecutive_json_fails times). Aborting this LLM run."
        return 1
      fi
      
      echo "Attempting to auto-recover..."
      # Create strong error feedback for LLM
      local recovery_msg="ERROR: Your previous response was NOT valid JSON. You MUST respond with ONLY a valid JSON object matching the requested schema. Do not include markdown blocks wrapping the JSON if they are malformed."
      conversation_history=$(echo "$conversation_history" | jq --arg content "$llm_content" '. + [{"role": "assistant", "content": $content}]')
      conversation_history=$(echo "$conversation_history" | jq --arg msg "$recovery_msg" '. + [{"role": "user", "content": $msg}]')
      continue
    else
      consecutive_json_fails=0
    fi
    
    # Add LLM response to conversation history
    conversation_history=$(echo "$conversation_history" | jq --arg content "$llm_content" '. + [{"role": "assistant", "content": $content}]')
    
    # Execute actions
    local execution_result
    execution_result=$(execute_nvidia_actions "$actions_json")

    local exit_code=$?
    
    # Add execution result to history
    conversation_history=$(echo "$conversation_history" | jq --arg res "$execution_result" '. + [{"role": "user", "content": ("Action execution result: " + $res)}]')
    
    # Check for completions
    if [[ $exit_code -eq 0 ]]; then
      if echo "$execution_result" | tail -n 5 | grep -qF "<promise>COMPLETE</promise>"; then
        # Re-emit completion signal so main loop can detect it in $OUTPUT
        echo "<promise>COMPLETE</promise>"
        return 0
      elif echo "$execution_result" | tail -n 5 | grep -qF "<promise>STORY_COMPLETED</promise>"; then
        # Story finished, but more remain. We can clear context or notify LLM.
        echo "Moving to next story..."
        conversation_history='[]' # Clear history for fresh start on next story
        continue
      fi
    fi
    
    # Adaptive sliding window with context monitoring (Faza 1)
    local _ctx_status
    _ctx_status=$(check_context_usage "$conversation_history")
    local hist_len
    hist_len=$(echo "$conversation_history" | jq 'length')
    
    if [[ "$_ctx_status" == "shrink" ]]; then
      # Aggressive trim — keep only 6 messages when context is critically high
      if (( hist_len > 6 )); then
        conversation_history=$(echo "$conversation_history" | jq '.[-6:]')
        echo "DEBUG: Context high — trimmed history from $hist_len to 6 messages" >&2
      fi
    elif (( hist_len > 10 )); then
      # Normal trim — keep last 10
      conversation_history=$(echo "$conversation_history" | jq '.[-10:]')
      echo "DEBUG: Trimmed conversation history from $hist_len to 10 messages" >&2
    fi
    # Safety check: prevent infinite loops
    if (( iteration_count++ > 40 )); then
      echo "Error: Exceeded maximum agent iterations"
      return 1
    fi
  done
}

CHECKPOINT_FILE="$SCRIPT_DIR/.checkpoint_iter"
START_ITERATION=1

# Resume from checkpoint if present
if [ -f "$CHECKPOINT_FILE" ]; then
  START_ITERATION=$(cat "$CHECKPOINT_FILE")
  echo "v1.6: Resuming from checkpoint: iteration $START_ITERATION"
fi

for i in $(seq "$START_ITERATION" "$MAX_ITERATIONS"); do
  # Update checkpoint
  echo "$i" > "$CHECKPOINT_FILE"
  
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="
  
  # Run the selected tool with the ralph prompt
  CURRENT_ITERATION=$i
  debug_log "TOOL is '$TOOL'"
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  elif [[ "$TOOL" == "claude" ]]; then
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  elif [[ "$TOOL" == "llm_direct_nvidia" ]]; then
    OUTPUT=$(run_nvidia_agent_loop) || true
    debug_log "OUTPUT from agent loop: $OUTPUT"
  fi
  
  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    print_summary
    
    # Auto-cleanup: archive runtime files after successful completion
    if [ -f "$PRD_FILE" ]; then
      CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "completed")
      FOLDER_NAME=$(echo "$CURRENT_BRANCH" | sed 's|^ralph/||')
      CLEANUP_DIR="$ARCHIVE_DIR/$(date +%Y-%m-%d)-$FOLDER_NAME"
      
      echo "Archiving runtime files to: $CLEANUP_DIR"
      mkdir -p "$CLEANUP_DIR"
      [ -f "$PRD_FILE" ] && mv "$PRD_FILE" "$CLEANUP_DIR/"
      [ -f "$PROGRESS_FILE" ] && mv "$PROGRESS_FILE" "$CLEANUP_DIR/"
      [ -f "$LAST_BRANCH_FILE" ] && mv "$LAST_BRANCH_FILE" "$CLEANUP_DIR/last-branch.txt"
      [ -f "$SCRIPT_DIR/.rejection_count" ] && mv "$SCRIPT_DIR/.rejection_count" "$CLEANUP_DIR/rejection_count.txt"
      [ -f "$CHECKPOINT_FILE" ] && mv "$CHECKPOINT_FILE" "$CLEANUP_DIR/"
      echo "Runtime files archived and cleaned up."
    fi
    
    exit 0
  fi
  
  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1