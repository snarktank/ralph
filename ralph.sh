#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool codex|amp|claude] [max_iterations]

set -Eeuo pipefail

TOOL="codex"
MAX_ITERATIONS=10

CODEX_BIN="${CODEX_BIN:-codex}"
RALPH_MODEL="${RALPH_MODEL:-}"
RALPH_PROFILE="${RALPH_PROFILE:-}"
RALPH_CODEX_FLAGS="${RALPH_CODEX_FLAGS:-}"
RALPH_MAX_RETRIES="${RALPH_MAX_RETRIES:-2}"
RALPH_MAX_STORY_FAILURES="${RALPH_MAX_STORY_FAILURES:-3}"
RALPH_RETRY_BASE_SECONDS="${RALPH_RETRY_BASE_SECONDS:-5}"
RALPH_SLEEP_SECONDS="${RALPH_SLEEP_SECONDS:-2}"
RALPH_REQUIRE_CLEAN="${RALPH_REQUIRE_CLEAN:-0}"

usage() {
  cat <<'USAGE'
Usage: ./ralph.sh [--tool codex|amp|claude] [max_iterations]

Environment:
  CODEX_BIN                 Codex executable to use (default: codex)
  RALPH_MODEL               Optional Codex model, passed as --model
  RALPH_PROFILE             Optional Codex profile, passed as --profile
  RALPH_CODEX_FLAGS         Extra flags appended to codex exec
  RALPH_MAX_RETRIES         Retry count for transient CLI failures (default: 2)
  RALPH_MAX_STORY_FAILURES  Consecutive failures allowed for one story (default: 3)
  RALPH_REQUIRE_CLEAN       Set to 1 to stop when git status is dirty
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h)
      usage
      exit 0
      ;;
    --tool)
      TOOL="${2:-}"
      shift 2
      ;;
    --tool=*)
      TOOL="${1#*=}"
      shift
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      else
        echo "Error: Unknown argument '$1'." >&2
        usage >&2
        exit 1
      fi
      shift
      ;;
  esac
done

if [[ "$TOOL" != "codex" && "$TOOL" != "amp" && "$TOOL" != "claude" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'codex', 'amp', or 'claude'." >&2
  exit 1
fi

if ! [[ "$RALPH_MAX_RETRIES" =~ ^[0-9]+$ ]]; then
  echo "Error: RALPH_MAX_RETRIES must be a non-negative integer." >&2
  exit 1
fi

if ! [[ "$RALPH_MAX_STORY_FAILURES" =~ ^[0-9]+$ ]] || [[ "$RALPH_MAX_STORY_FAILURES" -eq 0 ]]; then
  echo "Error: RALPH_MAX_STORY_FAILURES must be a positive integer." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$PROJECT_ROOT" ]]; then
  PROJECT_ROOT="$SCRIPT_DIR"
fi

PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"
LAST_PRD_FILE="$SCRIPT_DIR/.last-prd.json"
LOCK_FILE="$SCRIPT_DIR/.ralph.lock"
STATE_FILE="$SCRIPT_DIR/.ralph-state.json"
RUNS_DIR="$SCRIPT_DIR/runs"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
RUN_DIR="$RUNS_DIR/$RUN_ID"

log() {
  printf '%s\n' "$*"
}

require_command() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "Error: Required command '$name' was not found in PATH." >&2
    exit 1
  fi
}

cleanup_lock() {
  if [[ -f "$LOCK_FILE" ]] && [[ "$(cat "$LOCK_FILE" 2>/dev/null || true)" == "$$" ]]; then
    rm -f "$LOCK_FILE"
  fi
}

snapshot_current_prd() {
  if [[ ! -f "$PRD_FILE" ]]; then
    return 0
  fi

  local current_branch
  current_branch="$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")"
  if [[ -n "$current_branch" ]]; then
    echo "$current_branch" > "$LAST_BRANCH_FILE"
    cp "$PRD_FILE" "$LAST_PRD_FILE"
  fi
}

acquire_lock() {
  if [[ -f "$LOCK_FILE" ]]; then
    local existing_pid
    existing_pid="$(cat "$LOCK_FILE" 2>/dev/null || true)"
    if [[ "$existing_pid" =~ ^[0-9]+$ ]] && kill -0 "$existing_pid" 2>/dev/null; then
      echo "Error: Ralph is already running with PID $existing_pid." >&2
      exit 1
    fi
    echo "Removing stale Ralph lock: $LOCK_FILE"
    rm -f "$LOCK_FILE"
  fi

  printf '%s\n' "$$" > "$LOCK_FILE"
  trap cleanup_lock EXIT INT TERM
}

all_stories_complete() {
  jq -e '[.userStories[] | select(.passes != true)] | length == 0' "$PRD_FILE" >/dev/null
}

current_story_id() {
  jq -r '[.userStories[] | select(.passes != true)] | sort_by(.priority // 999999)[0].id // empty' "$PRD_FILE"
}

current_story_title() {
  local story_id="$1"
  jq -r --arg id "$story_id" '.userStories[] | select(.id == $id) | .title // ""' "$PRD_FILE"
}

story_passes() {
  local story_id="$1"
  jq -e --arg id "$story_id" '.userStories[] | select(.id == $id) | .passes == true' "$PRD_FILE" >/dev/null
}

reset_story_passes_false() {
  local story_id="$1"
  local reason="$2"
  local tmp_file
  tmp_file="$(mktemp)"
  jq --arg id "$story_id" --arg reason "$reason" '
    (.userStories[] | select(.id == $id)) |= (
      .passes = false
      | .notes = ((.notes // "") + "\n" + $reason)
    )
  ' "$PRD_FILE" > "$tmp_file"
  mv "$tmp_file" "$PRD_FILE"
}

write_status() {
  local file="$1"
  local status="$2"
  local iteration="$3"
  local story_id="$4"
  local story_title="$5"
  local failure_type="$6"
  local exit_code="$7"
  local attempts="$8"
  local started_at="$9"
  local finished_at="${10}"
  local head_before="${11}"
  local head_after="${12}"

  jq -n \
    --arg runId "$RUN_ID" \
    --arg runDir "$RUN_DIR" \
    --arg tool "$TOOL" \
    --arg status "$status" \
    --argjson iteration "$iteration" \
    --arg storyId "$story_id" \
    --arg storyTitle "$story_title" \
    --arg failureType "$failure_type" \
    --argjson exitCode "$exit_code" \
    --argjson attempts "$attempts" \
    --arg startedAt "$started_at" \
    --arg finishedAt "$finished_at" \
    --arg headBefore "$head_before" \
    --arg headAfter "$head_after" \
    '{
      runId: $runId,
      runDir: $runDir,
      tool: $tool,
      status: $status,
      iteration: $iteration,
      storyId: $storyId,
      storyTitle: $storyTitle,
      failureType: $failureType,
      exitCode: $exitCode,
      attempts: $attempts,
      startedAt: $startedAt,
      finishedAt: $finishedAt,
      headBefore: $headBefore,
      headAfter: $headAfter
    }' > "$file"
  cp "$file" "$STATE_FILE"
}

append_runner_progress() {
  local iteration="$1"
  local story_id="$2"
  local failure_type="$3"
  local iter_dir="$4"

  {
    echo ""
    echo "## $(date) - Ralph runner"
    echo "- Iteration: $iteration"
    echo "- Story: ${story_id:-unknown}"
    echo "- Status: $failure_type"
    echo "- Logs: $iter_dir"
    echo "---"
  } >> "$PROGRESS_FILE"
}

completion_signal_seen() {
  local output_file="$1"
  local last_message_file="$2"
  grep -q "<promise>COMPLETE</promise>" "$output_file" 2>/dev/null \
    || grep -q "<promise>COMPLETE</promise>" "$last_message_file" 2>/dev/null
}

classify_failure() {
  local exit_code="$1"
  local output_file="$2"

  if [[ "$exit_code" -eq 0 ]]; then
    echo "agent"
    return
  fi

  if [[ "$exit_code" -eq 126 || "$exit_code" -eq 127 ]]; then
    echo "cli"
    return
  fi

  if grep -Eiq 'command not found|not recognized|no such file|cannot execute|executable file not found' "$output_file"; then
    echo "cli"
    return
  fi

  if grep -Eiq 'network|connection|timed out|timeout|temporarily unavailable|rate limit|429|502|503|504' "$output_file"; then
    echo "transient"
    return
  fi

  if grep -Eiq 'permission|operation not permitted|access is denied|eacces|eperm|approval|sandbox' "$output_file"; then
    echo "permission"
    return
  fi

  if grep -Eiq 'auth|login|not logged in|api key|unauthorized|forbidden' "$output_file"; then
    echo "auth"
    return
  fi

  echo "agent"
}

is_retryable_failure() {
  local failure_type="$1"
  [[ "$failure_type" == "cli" || "$failure_type" == "transient" || "$failure_type" == "permission" || "$failure_type" == "auth" ]]
}

select_prompt_source() {
  case "$TOOL" in
    codex) echo "$SCRIPT_DIR/CODEX.md" ;;
    amp) echo "$SCRIPT_DIR/prompt.md" ;;
    claude) echo "$SCRIPT_DIR/CLAUDE.md" ;;
  esac
}

build_iteration_prompt() {
  local prompt_source="$1"
  local prompt_file="$2"
  local iteration="$3"
  local story_id="$4"
  local story_title="$5"
  local iter_dir="$6"

  {
    cat "$prompt_source"
    echo ""
    echo "---"
    echo ""
    echo "# Ralph Runtime Context"
    echo ""
    echo "- Iteration: $iteration of $MAX_ITERATIONS"
    echo "- Tool: $TOOL"
    echo "- Project root: $PROJECT_ROOT"
    echo "- Ralph script directory: $SCRIPT_DIR"
    echo "- PRD file: $PRD_FILE"
    echo "- Progress file: $PROGRESS_FILE"
    echo "- Run directory: $RUN_DIR"
    echo "- Iteration log directory: $iter_dir"
    echo "- Current story id: ${story_id:-none}"
    echo "- Current story title: ${story_title:-none}"
    echo ""
    echo "Use the paths above as the source of truth for this iteration."
  } > "$prompt_file"
}

run_agent_once() {
  local prompt_file="$1"
  local output_file="$2"
  local last_message_file="$3"

  case "$TOOL" in
    codex)
      local cmd=("$CODEX_BIN" exec "--dangerously-bypass-approvals-and-sandbox" "--cd" "$PROJECT_ROOT" "--color" "never" "-o" "$last_message_file")
      if [[ -n "$RALPH_MODEL" ]]; then
        cmd+=("--model" "$RALPH_MODEL")
      fi
      if [[ -n "$RALPH_PROFILE" ]]; then
        cmd+=("--profile" "$RALPH_PROFILE")
      fi
      if [[ -n "$RALPH_CODEX_FLAGS" ]]; then
        # shellcheck disable=SC2206
        local extra_flags=($RALPH_CODEX_FLAGS)
        cmd+=("${extra_flags[@]}")
      fi
      cmd+=("-")
      "${cmd[@]}" < "$prompt_file" 2>&1 | tee "$output_file"
      return "${PIPESTATUS[0]}"
      ;;
    amp)
      amp --dangerously-allow-all < "$prompt_file" 2>&1 | tee "$output_file"
      return "${PIPESTATUS[0]}"
      ;;
    claude)
      claude --dangerously-skip-permissions --print < "$prompt_file" 2>&1 | tee "$output_file"
      return "${PIPESTATUS[0]}"
      ;;
  esac
}

preflight() {
  require_command git
  require_command jq

  if ! git -C "$SCRIPT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Error: Ralph must run inside a git repository." >&2
    exit 1
  fi

  PROJECT_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"

  if [[ ! -f "$PRD_FILE" ]]; then
    echo "Error: Missing $PRD_FILE. Create it from prd.json.example before running Ralph." >&2
    exit 1
  fi

  jq -e '.userStories and (.userStories | type == "array")' "$PRD_FILE" >/dev/null

  local prompt_source
  prompt_source="$(select_prompt_source)"
  if [[ ! -f "$prompt_source" ]]; then
    echo "Error: Missing prompt file for tool '$TOOL': $prompt_source" >&2
    exit 1
  fi

  case "$TOOL" in
    codex)
      if ! command -v "$CODEX_BIN" >/dev/null 2>&1; then
        echo "Error: Codex executable '$CODEX_BIN' was not found. Set CODEX_BIN if needed." >&2
        exit 1
      fi
      "$CODEX_BIN" exec --help >/dev/null
      ;;
    amp)
      require_command amp
      ;;
    claude)
      require_command claude
      ;;
  esac

  local dirty_status
  dirty_status="$(git -C "$PROJECT_ROOT" status --porcelain)"
  if [[ -n "$dirty_status" ]]; then
    if [[ "$RALPH_REQUIRE_CLEAN" == "1" ]]; then
      echo "Error: Worktree has uncommitted changes. Commit, stash, or set RALPH_REQUIRE_CLEAN=0." >&2
      echo "$dirty_status" >&2
      exit 1
    fi
    echo "Warning: Worktree has uncommitted changes; Ralph will continue so failed stories can be resumed."
  fi

  mkdir -p "$RUN_DIR"
}

archive_previous_run_if_needed() {
  if [[ -f "$PRD_FILE" && -f "$LAST_BRANCH_FILE" ]]; then
    local current_branch
    local last_branch
    current_branch="$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")"
    last_branch="$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")"

    if [[ -n "$current_branch" && -n "$last_branch" && "$current_branch" != "$last_branch" ]]; then
      local date_part
      local folder_name
      local archive_folder
      date_part="$(date +%Y-%m-%d)"
      folder_name="$(echo "$last_branch" | sed 's|^ralph/||')"
      archive_folder="$ARCHIVE_DIR/$date_part-$folder_name"

      echo "Archiving previous run: $last_branch"
      mkdir -p "$archive_folder"
      if [[ -f "$LAST_PRD_FILE" ]]; then
        cp "$LAST_PRD_FILE" "$archive_folder/prd.json"
      else
        echo "No previous PRD snapshot found; archiving progress only."
      fi
      [[ -f "$PROGRESS_FILE" ]] && cp "$PROGRESS_FILE" "$archive_folder/"
      echo "Archived to: $archive_folder"

      {
        echo "# Ralph Progress Log"
        echo "Started: $(date)"
        echo "---"
      } > "$PROGRESS_FILE"
    fi
  fi

  snapshot_current_prd
}

initialize_progress_file() {
  if [[ ! -f "$PROGRESS_FILE" ]]; then
    {
      echo "# Ralph Progress Log"
      echo "Started: $(date)"
      echo "---"
    } > "$PROGRESS_FILE"
  fi
}

main() {
  preflight
  acquire_lock
  archive_previous_run_if_needed
  initialize_progress_file

  echo "Starting Ralph - Tool: $TOOL - Max iterations: $MAX_ITERATIONS"
  echo "Project root: $PROJECT_ROOT"
  echo "Run logs: $RUN_DIR"

  if all_stories_complete; then
    echo "All PRD stories already pass."
    echo "<promise>COMPLETE</promise>"
    snapshot_current_prd
    exit 0
  fi

  local last_failed_story=""
  local consecutive_story_failures=0

  for ((iteration = 1; iteration <= MAX_ITERATIONS; iteration++)); do
    local story_id
    local story_title
    story_id="$(current_story_id)"
    story_title=""
    if [[ -n "$story_id" ]]; then
      story_title="$(current_story_title "$story_id")"
    fi

    if [[ -z "$story_id" ]]; then
      echo "No remaining stories."
      echo "<promise>COMPLETE</promise>"
      exit 0
    fi

    local iter_dir
    local prompt_file
    local output_file
    local last_message_file
    local status_file
    local started_at
    local finished_at
    local head_before
    local head_after
    local prompt_source
    iter_dir="$RUN_DIR/iteration-$iteration"
    prompt_file="$iter_dir/prompt.md"
    output_file="$iter_dir/output.log"
    last_message_file="$iter_dir/last-message.md"
    status_file="$iter_dir/status.json"
    mkdir -p "$iter_dir"

    started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    head_before="$(git -C "$PROJECT_ROOT" rev-parse HEAD)"
    prompt_source="$(select_prompt_source)"
    build_iteration_prompt "$prompt_source" "$prompt_file" "$iteration" "$story_id" "$story_title" "$iter_dir"

    echo ""
    echo "==============================================================="
    echo "  Ralph Iteration $iteration of $MAX_ITERATIONS ($TOOL)"
    echo "  Story: $story_id - $story_title"
    echo "==============================================================="

    : > "$output_file"
    local attempt
    local exit_code=0
    local failure_type="none"
    local final_attempts=0
    local retry_exhausted=0

    for ((attempt = 1; attempt <= RALPH_MAX_RETRIES + 1; attempt++)); do
      final_attempts="$attempt"
      local attempt_output
      attempt_output="$iter_dir/output-attempt-$attempt.log"
      echo "===== Attempt $attempt at $(date) =====" | tee -a "$output_file"

      set +e
      run_agent_once "$prompt_file" "$attempt_output" "$last_message_file"
      exit_code="$?"
      set -e

      cat "$attempt_output" >> "$output_file"
      if [[ ! -s "$last_message_file" ]]; then
        cp "$attempt_output" "$last_message_file"
      fi

      failure_type="$(classify_failure "$exit_code" "$attempt_output")"

      if [[ "$exit_code" -eq 0 ]]; then
        break
      fi

      if ! is_retryable_failure "$failure_type"; then
        break
      fi

      if (( attempt <= RALPH_MAX_RETRIES )); then
        local sleep_seconds
        sleep_seconds=$((RALPH_RETRY_BASE_SECONDS * attempt))
        echo "Retryable $failure_type failure. Retrying in ${sleep_seconds}s..." | tee -a "$output_file"
        sleep "$sleep_seconds"
      else
        retry_exhausted=1
      fi
    done

    git -C "$PROJECT_ROOT" status --short > "$iter_dir/git-status.txt"
    git -C "$PROJECT_ROOT" diff --stat HEAD > "$iter_dir/diff-stat.txt" || true
    head_after="$(git -C "$PROJECT_ROOT" rev-parse HEAD)"
    finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    if [[ "$retry_exhausted" -eq 1 ]]; then
      write_status "$status_file" "stopped" "$iteration" "$story_id" "$story_title" "$failure_type" "$exit_code" "$final_attempts" "$started_at" "$finished_at" "$head_before" "$head_after"
      append_runner_progress "$iteration" "$story_id" "$failure_type" "$iter_dir"
      snapshot_current_prd
      echo "Ralph stopped after retryable $failure_type failures. See $iter_dir."
      exit 1
    fi

    local completion_signal=0
    local prd_complete=0
    if completion_signal_seen "$output_file" "$last_message_file"; then
      completion_signal=1
    fi
    if all_stories_complete; then
      prd_complete=1
    fi

    if [[ "$exit_code" -eq 0 ]] && story_passes "$story_id" && [[ "$head_before" == "$head_after" ]]; then
      local reset_reason
      reset_reason="Ralph runner reset passes=false because $story_id was marked complete without a commit. See $iter_dir."
      reset_story_passes_false "$story_id" "$reset_reason"
      failure_type="story_failure"
      prd_complete=0
    elif [[ "$exit_code" -eq 0 && "$prd_complete" -eq 1 ]]; then
      write_status "$status_file" "complete" "$iteration" "$story_id" "$story_title" "none" "$exit_code" "$final_attempts" "$started_at" "$finished_at" "$head_before" "$head_after"
      snapshot_current_prd
      echo ""
      echo "Ralph completed all tasks!"
      echo "Completed at iteration $iteration of $MAX_ITERATIONS"
      echo "<promise>COMPLETE</promise>"
      exit 0
    elif [[ "$exit_code" -eq 0 && "$completion_signal" -eq 1 && "$prd_complete" -ne 1 ]]; then
      failure_type="story_failure"
    elif [[ "$exit_code" -eq 0 ]] && story_passes "$story_id" && [[ "$head_before" != "$head_after" ]]; then
      failure_type="none"
      write_status "$status_file" "story_complete" "$iteration" "$story_id" "$story_title" "$failure_type" "$exit_code" "$final_attempts" "$started_at" "$finished_at" "$head_before" "$head_after"
      snapshot_current_prd
      last_failed_story=""
      consecutive_story_failures=0
      echo "Story $story_id completed. Continuing..."
      sleep "$RALPH_SLEEP_SECONDS"
      continue
    else
      failure_type="story_failure"
    fi

    if [[ "$story_id" == "$last_failed_story" ]]; then
      consecutive_story_failures=$((consecutive_story_failures + 1))
    else
      last_failed_story="$story_id"
      consecutive_story_failures=1
    fi

    write_status "$status_file" "story_failure" "$iteration" "$story_id" "$story_title" "$failure_type" "$exit_code" "$final_attempts" "$started_at" "$finished_at" "$head_before" "$head_after"
    append_runner_progress "$iteration" "$story_id" "$failure_type" "$iter_dir"
    snapshot_current_prd

    if (( consecutive_story_failures >= RALPH_MAX_STORY_FAILURES )); then
      echo "Ralph stopped after $consecutive_story_failures consecutive failures for $story_id."
      echo "See $iter_dir for details."
      exit 1
    fi

    echo "Iteration $iteration ended with story failure. Continuing so the next fresh context can repair it..."
    sleep "$RALPH_SLEEP_SECONDS"
  done

  echo ""
  echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
  echo "Check $PROGRESS_FILE and $RUN_DIR for status."
  snapshot_current_prd
  exit 1
}

main
