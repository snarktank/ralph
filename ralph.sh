#!/bin/bash
# Ralph Harness — Autonomous AI agent loop with Generator-Evaluator architecture
# Usage: ./ralph.sh [--mode harness|simple] [--tool amp|claude]
#                  [--audit] [--track-cost] [--max-retries N] [--max-contract-rounds N]
#                  [--degradation-threshold N] [max_iterations]

# set -e 已移至主流程入口（run_harness_mode / run_simple_mode）。
# 初始化/清理代码不受 set -e 约束 —— BashFAQ/105

# Force UTF-8 encoding for Chinese/Unicode on Windows MSYS2
# Detect available locale: C.UTF-8 is glibc built-in, en_US.UTF-8 is common fallback
# set +e: background mode may have different locale environment
set +e
if locale -a 2>/dev/null | grep -qi "C.UTF-8\|C.utf8"; then
  export LC_ALL=C.UTF-8
elif locale -a 2>/dev/null | grep -qi "en_US.UTF-8\|en_US.utf8\|en_US.utf-8"; then
  export LC_ALL=$(locale -a 2>/dev/null | grep -i "en_US.utf" | head -1 || echo "C")
else
  export LC_ALL=C
fi
set -e
export LANG="$LC_ALL"

# ============================================================
# Parse arguments
# ============================================================
TOOL="claude"
MODE="harness"
MAX_ITERATIONS=10
MAX_RETRIES=3
MAX_CONTRACT_ROUNDS=5
AUDIT=false
TRACK_COST=false
ONE_SHOT=false
DEGRADATION_THRESHOLD=2  # abort retries if score drops N times in a row

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
    --mode)
      MODE="$2"
      shift 2
      ;;
    --mode=*)
      MODE="${1#*=}"
      shift
      ;;
    --max-retries)
      MAX_RETRIES="$2"
      shift 2
      ;;
    --max-retries=*)
      MAX_RETRIES="${1#*=}"
      shift
      ;;
    --max-contract-rounds)
      MAX_CONTRACT_ROUNDS="$2"
      shift 2
      ;;
    --max-contract-rounds=*)
      MAX_CONTRACT_ROUNDS="${1#*=}"
      shift
      ;;
    --audit)
      AUDIT=true
      shift
      ;;
    --track-cost)
      TRACK_COST=true
      shift
      ;;
    --one-shot)
      ONE_SHOT=true
      shift
      ;;
    --degradation-threshold)
      DEGRADATION_THRESHOLD="$2"
      shift 2
      ;;
    --degradation-threshold=*)
      DEGRADATION_THRESHOLD="${1#*=}"
      shift
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      fi
      shift
      ;;
  esac
done

# Validate arguments
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp' or 'claude'."
  exit 1
fi

if [[ "$MODE" != "harness" && "$MODE" != "simple" ]]; then
  echo "Error: Invalid mode '$MODE'. Must be 'harness' or 'simple'."
  exit 1
fi

# ============================================================
# Paths
# ============================================================
# SCRIPT_DIR: where Ralph's own files live (generator-prompt.md, evaluator-prompt.md, etc.)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# PROJECT_DIR: the user's project root (where prd.json and progress.txt live)
# Defaults to current working directory — works for both standalone and submodule usage.
# Override with RALPH_PROJECT_DIR env var if needed.
PROJECT_DIR="${RALPH_PROJECT_DIR:-$(pwd)}"
RALPH_DIR="$PROJECT_DIR/.ralph"
PRD_FILE="$PROJECT_DIR/prd.json"
PROGRESS_FILE="$PROJECT_DIR/progress.txt"
CONTRACT_FILE="$RALPH_DIR/contract.json"
EVALUATION_FILE="$RALPH_DIR/evaluation.json"
PHASE_FILE="$RALPH_DIR/phase"
ARCHIVE_DIR="$PROJECT_DIR/archive"
LAST_BRANCH_FILE="$RALPH_DIR/last-branch"
CHANGES_FILE="$RALPH_DIR/changes-summary.txt"
GENERATOR_PROMPT="$SCRIPT_DIR/generator-prompt.md"
EVALUATOR_PROMPT="$SCRIPT_DIR/evaluator-prompt.md"
LEGACY_PROMPT="$SCRIPT_DIR/CLAUDE.md"

# Ensure runtime directory exists
mkdir -p "$RALPH_DIR"

# ============================================================
# Archive previous run if branch changed
# ============================================================
archive_previous_run() {
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
      [ -f "$CONTRACT_FILE" ] && cp "$CONTRACT_FILE" "$ARCHIVE_FOLDER/"
      [ -f "$EVALUATION_FILE" ] && cp "$EVALUATION_FILE" "$ARCHIVE_FOLDER/"
      echo "   Archived to: $ARCHIVE_FOLDER"

      echo "# Ralph Progress Log" > "$PROGRESS_FILE"
      echo "Started: $(date)" >> "$PROGRESS_FILE"
      echo "---" >> "$PROGRESS_FILE"
    fi
  fi
}

archive_previous_run

# Track current branch
if [ -f "$PRD_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  if [ -n "$CURRENT_BRANCH" ]; then
    echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"
  fi
fi

# Initialize progress file if needed
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

# ============================================================
# Signal handling & cleanup
# ============================================================
RALPH_NORMAL_EXIT=false
PLAYWRIGHT_MCP_PORT=8931
PLAYWRIGHT_MCP_PID_FILE="${RALPH_DIR}/playwright-mcp.pid"
# 清理上次 crash 遗留的 PID 文件 + Playwright 端口占用
rm -f "${RALPH_DIR}/agent-pid.txt"
if command -v tasklist >/dev/null 2>&1; then
  _port_pid=$(netstat -ano 2>/dev/null | grep ":${PLAYWRIGHT_MCP_PORT}" | grep LISTENING | awk '{print $NF}' | head -1 || true)
  if [ -n "$_port_pid" ] && [ "$_port_pid" != "0" ]; then
    taskkill /PID "$_port_pid" /F 2>/dev/null || true
  fi
fi

trap 'cleanup SIGINT' SIGINT
trap 'cleanup SIGTERM' SIGTERM
trap 'cleanup EXIT' EXIT

cleanup() {
  set +e  # 清理期间不因单条命令失败而触发 set -e 退出
  local signal="$1"

  echo ""
  echo "==============================================================="
  if [ "$signal" = "EXIT" ]; then
    echo "  Ralph exiting. Final cleanup..."
  else
    echo "  Interrupted ($signal). Cleaning up..."
  fi
  echo "==============================================================="

  stop_playwright_mcp_server
  kill_claude_subprocesses
  save_interrupt_state

  echo "  Ralph stopped."
  echo "==============================================================="

  if [ "$signal" != "EXIT" ]; then
    exit 130
  fi
}

# Check if a process is alive using multiple detection methods.
# On MSYS2/Windows, both kill -0 and ps -p can falsely report dead processes
# as alive (both depend on the same /proc emulation layer).  tasklist queries
# the Windows kernel process table directly and is the ground truth.
# Returns 0 if the process is alive, 1 if it is dead.
_is_process_alive() {
  local pid="$1"

  # Windows: tasklist queries the kernel directly, immune to MSYS2 /proc lag
  if command -v tasklist >/dev/null 2>&1; then
    if tasklist /FI "PID eq $pid" 2>/dev/null | grep -qw "$pid"; then
      return 0
    fi
    return 1
  fi

  # Linux/macOS fallback
  if ps -p "$pid" >/dev/null 2>&1; then
    return 0
  fi
  if kill -0 "$pid" 2>/dev/null; then
    return 0
  fi
  return 1
}

# Terminate a process and all its descendants (process tree).
# Windows: taskkill /T does native tree termination.
# Linux/macOS: recursively enumerate and kill children first, then parent.
_kill_process_tree() {
  local pid="$1"
  [ -z "$pid" ] && return 0

  if command -v tasklist >/dev/null 2>&1; then
    taskkill /T /PID "$pid" /F 2>/dev/null
  else
    local children
    children=$(ps -o pid= --ppid "$pid" 2>/dev/null)
    for child in $children; do
      _kill_process_tree "$child"
    done
    kill -9 "$pid" 2>/dev/null || true
  fi
}

kill_claude_subprocesses() {
  echo "  Terminating subprocesses..."

  # 精准杀死 ralph 记录的 agent PID（不改动其他 Claude Code 窗口）
  if [ -f "${RALPH_DIR}/agent-pid.txt" ]; then
    local pid
    pid=$(cat "${RALPH_DIR}/agent-pid.txt")
    if [ -n "$pid" ] && _is_process_alive "$pid"; then
      _kill_process_tree "$pid"
      echo "  Terminated agent PID tree: $pid"
    fi
    rm -f "${RALPH_DIR}/agent-pid.txt"
  fi

  # 备用：jobs 清理
  local child_pids
  child_pids=$(jobs -p 2>/dev/null) || true
  if [ -n "$child_pids" ]; then
    for cpid in $child_pids; do
      _kill_process_tree "$cpid"
    done
    echo "  Terminated child jobs tree: $child_pids"
  fi
}

save_interrupt_state() {
  local state_file="${RALPH_DIR}/interrupt-state.json"
  local current_story="unknown"

  if [ -f "$PRD_FILE" ]; then
    current_story=$(jq -r '[.userStories[] | select(.passes == false)] | first.id // "all-done"' "$PRD_FILE" 2>/dev/null) || true
  fi

  jq -n \
    --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg story "$current_story" \
    --arg mode "$MODE" \
    --argjson contract "$([ -f "$CONTRACT_FILE" ] && echo true || echo false)" \
    --argjson eval "$([ -f "$EVALUATION_FILE" ] && echo true || echo false)" \
    '{
      timestamp: $ts,
      lastStory: $story,
      mode: $mode,
      contractFileExists: $contract,
      evaluationFileExists: $eval
    }' > "$state_file"

  echo "  State saved to $state_file"
}

# ============================================================
# Contract failure — save state and exit for user resolution
# ============================================================
write_contract_failure_summary() {
  local story_id="$1"
  local story_title="$2"
  local summary_file="${RALPH_DIR}/contract-failure-summary.md"

  cat > "$summary_file" << SUMMARYEOF
# Contract Negotiation Failure

## Story
- **ID:** $story_id
- **Title:** $story_title

## Round History
$(if [ -f "${RALPH_DIR}/contract-scores.txt" ]; then
    echo "| Round | Score | Contract File |"
    echo "|-------|-------|---------------|"
    while read -r round score; do
      echo "| $round | $score/100 | contract-round-${round}.json |"
    done < "${RALPH_DIR}/contract-scores.txt"
  else
    echo "No contract rounds were recorded (Generator may have crashed)."
  fi)

## Last Contract State
\`\`\`json
$(cat "$CONTRACT_FILE" 2>/dev/null | jq '.' 2>/dev/null || echo "No contract.json exists")
\`\`\`

## PRD Context
- **Story passes:** $(jq -r --arg id "$story_id" '.userStories[] | select(.id == $id) | .passes' "$PRD_FILE")
- **Story retries:** $(jq -r --arg id "$story_id" '.userStories[] | select(.id == $id) | .retryCount // 0' "$PRD_FILE")
- **Acceptance criteria:**
$(jq -r --arg id "$story_id" '.userStories[] | select(.id == $id) | .acceptanceCriteria[]? | "- \(.)"' "$PRD_FILE")

## What You Need to Decide

Please discuss in plan mode and produce a resolution covering:

1. **Which contract to use?** (pick a specific round, or describe a new contract)
2. **What should change?** (scope, acceptance criteria, verification steps)
3. **Why will this work?** (address why previous rounds failed)

Save the final resolution to \`.ralph/user-resolution.md\`.
SUMMARYEOF

  echo "  Contract failure summary saved to $summary_file"
}

exit_for_user_resolution() {
  local story_id="$1"
  local story_title="$2"

  write_contract_failure_summary "$story_id" "$story_title"

  echo ""
  echo "==============================================================="
  echo "  CONTRACT NEGOTIATION FAILED"
  echo "==============================================================="
  echo "  Story: $story_id - $story_title"
  echo ""
  echo "  All rounds of contract negotiation have been exhausted."
  echo "  A summary has been saved to:"
  echo "    ${RALPH_DIR}/contract-failure-summary.md"
  echo ""
  echo "  Next step: Enter plan mode in this Claude Code session"
  echo "  and discuss a resolution. Save the agreed plan to:"
  echo "    ${RALPH_DIR}/user-resolution.md"
  echo ""
  echo "  Then re-run ralph.sh to continue."
  echo "==============================================================="

  RALPH_NORMAL_EXIT=true
  exit 2  # 2 = contract negotiation failed, needs human intervention
}

write_evaluator_feedback() {
  local story_id="$1"
  local feedback_file="${RALPH_DIR}/evaluator-feedback.md"

  cat > "$feedback_file" << FBEOF
# Evaluator Feedback on User Resolution

## Story: $story_id

$(if [ -f "$CONTRACT_FILE" ]; then
    echo "## Evaluator's Response"
    jq -r '.history[-1].message // "No specific feedback"' "$CONTRACT_FILE" 2>/dev/null || echo "N/A"
    echo ""
    echo "## Current Contract Status"
    jq -r '.status // "unknown"' "$CONTRACT_FILE" 2>/dev/null || echo "unknown"
  else
    echo "Evaluator did not modify contract.json"
  fi)

## Next Steps

Please enter plan mode, discuss the feedback above, and revise the resolution.
Save the updated plan to \`.ralph/user-resolution.md\` and re-run ralph.sh.
FBEOF

  echo "  Evaluator feedback saved to $feedback_file"
}

run_evaluator_user_resolution() {
  local story_id="$1"
  local mode="$2"

  set_phase "evaluator-user-resolution"

  local prompt="Current phase: evaluator-user-resolution (see evaluator-prompt.md Phase 1.5).

The contract negotiation for story '$story_id' failed to reach agreement.
A human user and Claude Code have discussed the situation in plan mode
and produced the following resolution. Your job is to formally review it.

## User Resolution
$(cat "${RALPH_DIR}/user-resolution.md")

## Contract Round History
$(cat "${RALPH_DIR}/contract-scores.txt" 2>/dev/null || echo "No rounds recorded")

## Current Contract State
$(cat "$CONTRACT_FILE" 2>/dev/null | jq '.' 2>/dev/null || echo "No contract.json exists")

## Instructions
- If the resolution is sound → lock contract.json (status: locked, evaluatorSignature: user-resolution-approved)
- If the resolution has issues → write specific feedback in contract.json history (action: user-resolution-returned)
- Do NOT add new requirements beyond the original story scope."

  echo "$prompt" > "${RALPH_DIR}/user-resolution-prompt.md"

  run_agent "${RALPH_DIR}/user-resolution-prompt.md" "user-resolution-eval-${story_id}"

  if [ -f "$CONTRACT_FILE" ]; then
    local status
    status=$(jq -r '.status // "unknown"' "$CONTRACT_FILE" 2>/dev/null) || status="parse_error"
    [ "$status" = "locked" ] && return 0
  fi
  return 1
}

# ============================================================
# Phase discipline verification functions
# ============================================================

# Check Generator's output after contract phase — must create contract.json, never write source code
verify_contract_phase_output() {
  if [ ! -f "$CONTRACT_FILE" ]; then
    echo "  ERROR: Generator did not create contract.json"

    # Check if Generator violated phase discipline by writing source code
    if git rev-parse --git-dir >/dev/null 2>&1; then
      local changed_files
      changed_files=$(git diff --name-only HEAD 2>/dev/null | grep -v ".ralph/" | grep -v "CLAUDE.md" | grep -v "AGENTS.md" | grep -v "prd.json" | grep -v "progress.txt" | head -5)
      if [ -n "$changed_files" ]; then
        echo ""
        echo "==============================================================="
        echo "  PHASE VIOLATION DETECTED"
        echo "==============================================================="
        echo "  Generator skipped the contract phase and wrote code directly:"
        echo "$changed_files" | sed 's/^/    /'
        echo ""
        echo "  This is a phase discipline violation."
        echo "  The Generator must create .ralph/contract.json FIRST."
        echo ""
        echo "  Actions:"
        echo "    1. Revert these files: git checkout -- <files>"
        echo "    2. Re-run ralph.sh to retry contract negotiation"
        echo "==============================================================="
        return 1
      fi
    fi

    return 1
  fi
  return 0
}

# Check Evaluator's output after contract review — must preserve contract.json, never write source code
verify_evaluator_contract_output() {
  if [ ! -f "$CONTRACT_FILE" ]; then
    echo "  ERROR: Evaluator removed or did not preserve contract.json"
    return 1
  fi

  if git rev-parse --git-dir >/dev/null 2>&1; then
    local changed_files
    changed_files=$(git diff --name-only HEAD 2>/dev/null | grep -v ".ralph/" | grep -v "CLAUDE.md" | grep -v "AGENTS.md" | grep -v "prd.json" | grep -v "progress.txt" | head -5)
    if [ -n "$changed_files" ]; then
      echo ""
      echo "==============================================================="
      echo "  EVALUATOR PHASE VIOLATION"
      echo "==============================================================="
      echo "  Evaluator modified source files during contract review:"
      echo "$changed_files" | sed 's/^/    /'
      echo ""
      echo "  Evaluator must only review contract.json, never modify code."
      echo "  Reverting these changes..."
      git checkout -- $changed_files 2>/dev/null
      echo "==============================================================="
    fi
  fi
  return 0
}

# Check Evaluator's output after evaluation — must create evaluation.json, never write source code
verify_evaluator_evaluate_output() {
  if [ ! -f "$EVALUATION_FILE" ]; then
    echo "  WARNING: Evaluator did not create evaluation.json"
  fi

  if git rev-parse --git-dir >/dev/null 2>&1; then
    local changed_files
    changed_files=$(git diff --name-only HEAD 2>/dev/null | grep -v ".ralph/" | grep -v "CLAUDE.md" | grep -v "AGENTS.md" | grep -v "prd.json" | grep -v "progress.txt" | head -5)
    if [ -n "$changed_files" ]; then
      echo ""
      echo "==============================================================="
      echo "  EVALUATOR PHASE VIOLATION"
      echo "==============================================================="
      echo "  Evaluator modified source files during evaluation:"
      echo "$changed_files" | sed 's/^/    /'
      echo ""
      echo "  Evaluator must ONLY evaluate, never modify code."
      echo "  Reverting these changes..."
      git checkout -- $changed_files 2>/dev/null
      echo "==============================================================="
    fi
  fi
  return 0
}

# ============================================================
# Agent wait with output detection + heartbeat
# ============================================================

# Check if agent has produced contract output (contract phases only).
# Build and evaluate phases use natural exit — see wait_for_agent().
_check_agent_output() {
  local pid="$1"
  local phase
  phase=$(cat "$PHASE_FILE" 2>/dev/null)

  case "$phase" in
    generator-contract)
      if [ -f "$CONTRACT_FILE" ] && [ -s "$CONTRACT_FILE" ]; then
        local status
        status=$(jq -r '.status // empty' "$CONTRACT_FILE" 2>/dev/null || echo "")
        if [ -n "$status" ]; then
          echo "  [DETECT] contract.json ready (status: $status). Proceeding..."
          _kill_process_tree "$pid"
          return 0
        fi
      fi
      ;;
    evaluator-contract|evaluator-user-resolution)
      if [ -f "$CONTRACT_FILE" ] && [ -s "$CONTRACT_FILE" ]; then
        local status
        status=$(jq -r '.status // empty' "$CONTRACT_FILE" 2>/dev/null || echo "")
        case "$status" in
          locked|generator_revise)
            echo "  [DETECT] contract.json review done (status: $status). Proceeding..."
            _kill_process_tree "$pid"
            return 0
            ;;
        esac
      fi
      ;;
  esac
  return 1
}

# Wait for agent completion.
# Build/evaluate: file-based signals (immune to launcher/worker PID mismatch).
# Contract: process polling + _check_agent_output.
wait_for_agent() {
  local pid="$1"
  local phase_label="$2"
  local phase
  phase=$(cat "$PHASE_FILE" 2>/dev/null)

  case "$phase" in
    generator-build)    _wait_for_file "${RALPH_DIR}/build-done" "$phase_label" "Generator build" ;;
    evaluator-evaluate) _wait_for_evaluation "$phase_label" ;;
    *)                  _wait_for_process_or_output "$pid" "$phase_label" ;;
  esac
}

# Wait for a sentinel file (build/evaluate phases — no PID dependency).
_wait_for_file() {
  local file="$1" phase_label="$2" desc="$3"
  local elapsed=0 tick=60 timeout=7200
  rm -f "$file"
  while [ $elapsed -lt $timeout ]; do
    sleep $tick; elapsed=$((elapsed + tick))
    [ $((elapsed % 600)) -eq 0 ] && echo "  [HEARTBEAT] $phase_label — $desc, $((elapsed / 60)) min..."
    [ -f "$file" ] && echo "  $desc complete ($((elapsed / 60)) min)." && return 0
  done
  echo "  [TIMEOUT] $desc did not complete within $((timeout / 60)) min."
  return 1
}

# Wait for evaluation.json to appear with valid score.
_wait_for_evaluation() {
  local phase_label="$1" elapsed=0 tick=60 timeout=7200
  while [ $elapsed -lt $timeout ]; do
    sleep $tick; elapsed=$((elapsed + tick))
    [ $((elapsed % 600)) -eq 0 ] && echo "  [HEARTBEAT] $phase_label — waiting for evaluation, $((elapsed / 60)) min..."
    if [ -f "$EVALUATION_FILE" ] && [ -s "$EVALUATION_FILE" ]; then
      local s; s=$(jq -r '.overallScore // -1' "$EVALUATION_FILE" 2>/dev/null || echo "-1")
      [ "$s" != "-1" ] && [ "$s" != "null" ] && echo "  Evaluation complete (score: $s, $((elapsed / 60)) min)." && return 0
    fi
  done
  echo "  [TIMEOUT] No evaluation.json after $((timeout / 60)) min."
  return 1
}

# Process-polling wait for contract phases.
_wait_for_process_or_output() {
  local pid="$1" phase_label="$2"
  local output_ready=false elapsed=0 tick=60 heartbeat=600
  while _is_process_alive "$pid"; do
    sleep $tick; elapsed=$((elapsed + tick))
    [ $((elapsed % heartbeat)) -eq 0 ] && echo "  [HEARTBEAT] $phase_label — PID $pid running $((elapsed / 60)) min..."
    [ "$output_ready" = false ] && [ $elapsed -ge 120 ] && _check_agent_output "$pid" && output_ready=true
  done
  wait "$pid" 2>/dev/null
}

# ============================================================
# Context assembly: pre-load all required files into stdin
# ============================================================
assemble_agent_context() {
  local prompt_file="$1"
  local phase
  phase=$(cat "$PHASE_FILE" 2>/dev/null)

  # 1. Agent prompt (always first)
  cat "$prompt_file"

  # 2. prd.json — summary of all stories + full details of current story only
  if [ -f "$PRD_FILE" ]; then
    echo ""; echo "=== PROJECT CONTEXT ==="
    jq '{
      projectName,
      branchName,
      techStack,
      storyProgress: [.userStories[] | {id, title, passes, bestEffort, retryCount}],
      currentStory: [.userStories[] | select(.passes == false)] | first
    }' "$PRD_FILE" 2>/dev/null || cat "$PRD_FILE"  # fallback to full file if jq fails
  fi

  # 3. Codebase Patterns from progress.txt
  if [ -f "$PROGRESS_FILE" ]; then
    local patterns
    patterns=$(awk '/^## Codebase Patterns/,/^---$|^## [0-9]/{print}' "$PROGRESS_FILE" 2>/dev/null)
    if [ -n "$patterns" ]; then
      echo ""; echo "=== CODEBASE PATTERNS ==="
      echo "$patterns"
    fi
  fi

  # 4. Phase-specific files
  case "$phase" in
    generator-contract)
      [ -f "$CONTRACT_FILE" ] && echo "" && echo "=== CURRENT CONTRACT ===" && cat "$CONTRACT_FILE"
      if [ -f "$EVALUATION_FILE" ]; then
        echo ""; echo "=== PREVIOUS EVALUATION ==="
        jq '{overallScore, feedback, verifiedCriteria: [.verifiedCriteria[]?|select(.result=="FAIL")]}' "$EVALUATION_FILE" 2>/dev/null
      fi
      ;;
    generator-build)
      [ -f "$CONTRACT_FILE" ] && echo "" && echo "=== LOCKED CONTRACT (DO NOT MODIFY) ===" && cat "$CONTRACT_FILE"
      if [ -f "$EVALUATION_FILE" ]; then
        echo ""; echo "=== PREVIOUS EVALUATION FEEDBACK ==="
        jq '{overallScore, feedback, verifiedCriteria: [.verifiedCriteria[]?|select(.result=="FAIL")]}' "$EVALUATION_FILE" 2>/dev/null
      fi
      ;;
    evaluator-contract)
      [ -f "$CONTRACT_FILE" ] && echo "" && echo "=== PROPOSED CONTRACT ===" && cat "$CONTRACT_FILE"
      [ -f "${RALPH_DIR}/contract-scores.txt" ] && echo "" && echo "=== ROUND HISTORY ===" && cat "${RALPH_DIR}/contract-scores.txt"
      ;;
    evaluator-evaluate)
      [ -f "$CONTRACT_FILE" ] && echo "" && echo "=== LOCKED CONTRACT ===" && cat "$CONTRACT_FILE"
      if [ -f "$EVALUATION_FILE" ]; then
        echo ""; echo "=== PREVIOUS EVALUATION ==="
        jq '{overallScore, feedback, verifiedCriteria: [.verifiedCriteria[]?|select(.result=="FAIL")]}' "$EVALUATION_FILE" 2>/dev/null
      fi
      ;;
    evaluator-user-resolution)
      [ -f "${RALPH_DIR}/user-resolution.md" ] && echo "" && echo "=== USER RESOLUTION ===" && cat "${RALPH_DIR}/user-resolution.md"
      [ -f "$CONTRACT_FILE" ] && echo "" && echo "=== CURRENT CONTRACT ===" && cat "$CONTRACT_FILE"
      [ -f "${RALPH_DIR}/contract-scores.txt" ] && echo "" && echo "=== ROUND HISTORY ===" && cat "${RALPH_DIR}/contract-scores.txt"
      ;;
  esac
}

# ============================================================
# Helper: Run an AI instance with a given prompt file
# ============================================================
run_agent() {
  local prompt_file="$1"
  local phase_label="$2"

  cost_track_start "$phase_label"

  if [[ "$TOOL" == "amp" ]]; then
    assemble_agent_context "$prompt_file" | amp --dangerously-allow-all 2>&1 || true
  else
    assemble_agent_context "$prompt_file" | claude --dangerously-skip-permissions --print 2>&1 || true &
    local agent_pid=$!
    echo "$agent_pid" > "${RALPH_DIR}/agent-pid.txt"
    wait_for_agent "$agent_pid" "$phase_label"
    _kill_process_tree "$agent_pid" 2>/dev/null || true
    rm -f "${RALPH_DIR}/agent-pid.txt"
  fi

  cost_track_end "$phase_label"

  # Check if agent reported a missing tool
  if [ -f "$TOOL_MISSING_FILE" ]; then
    echo ""
    echo "==============================================="
    echo "  TOOL MISSING — Agent cannot auto-install"
    echo "==============================================="
    cat "$TOOL_MISSING_FILE"
    echo ""
    echo "  Install the tool above, then press ENTER to retry this phase..."
    echo "  (or Ctrl+C to abort Ralph)"
    echo "==============================================="
    if [ -t 0 ]; then
      read -r -p ""
    else
      echo "  [SKIP] stdin unavailable (background mode)."
    fi
    rm -f "$TOOL_MISSING_FILE"
    echo "  Resuming after manual tool install..."
  fi
}

# ============================================================
# Helper: Set current phase
# ============================================================
set_phase() {
  echo "$1" > "$PHASE_FILE"
}

# ============================================================
# Tool auto-install & pause mechanism
# ============================================================
TOOL_MISSING_FILE="$RALPH_DIR/tool-missing.txt"

ensure_tool() {
  local name="$1"
  local check_cmd="$2"
  local install_cmd="$3"
  local category="${4:-CLI}"
  local required_by="${5:-Ralph}"

  # Check if already installed
  if eval "$check_cmd" &>/dev/null; then
    return 0
  fi

  echo ""
  echo "  [MISSING] $name ($category) — required by $required_by"
  echo "  Attempting auto-install..."

  if eval "$install_cmd" &>/dev/null; then
    echo "  [INSTALLED] $name — auto-install succeeded."
    return 0
  fi

  # Auto-install failed — write report and pause
  echo ""
  echo "  ==============================================="
  echo "  TOOL MISSING: $name"
  echo "  ==============================================="
  echo "  Category:    $category"
  echo "  Required by: $required_by"
  echo "  Check:       $check_cmd"
  echo ""
  echo "  Auto-install failed. Manual install needed:"
  echo "    $install_cmd"
  echo ""
  echo "  After installing, press ENTER to continue..."
  echo "  (or Ctrl+C to abort Ralph)"
  echo "  ==============================================="

  # Write report file for external monitoring
  cat > "$TOOL_MISSING_FILE" << EOF
tool: $name
category: $category
required_by: $required_by
install_command: $install_cmd
timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)
auto_install_attempted: true
auto_install_result: failed
action_needed: Run the install command above, then press ENTER in the Ralph session.
EOF

  # Pause for user intervention
  if [ -t 0 ]; then
    read -r -p ""
  else
    echo "  [SKIP] stdin unavailable (background mode) — cannot pause."
    return 1
  fi

  # Re-check after user claims to have fixed it
  if eval "$check_cmd" &>/dev/null; then
    echo "  [OK] $name is now available. Continuing..."
    rm -f "$TOOL_MISSING_FILE"
    return 0
  fi

  echo "  [STILL MISSING] $name after manual intervention."
  echo "  Ralph cannot continue without this tool."
  echo "  Check $TOOL_MISSING_FILE for details."
  return 1
}

# ============================================================
# Playwright MCP HTTP server lifecycle
# ============================================================

start_playwright_mcp_server() {
  # 已有可用服务器 → 直接复用（plain HTTP GET，400 也是有效响应）
  if curl -s --max-time 2 "http://127.0.0.1:${PLAYWRIGHT_MCP_PORT}/mcp" >/dev/null 2>&1; then
    echo "  Playwright MCP already available on port ${PLAYWRIGHT_MCP_PORT}. Reusing."
    return 0
  fi

  # 端口被占但无响应 → 清理僵尸
  if command -v tasklist >/dev/null 2>&1; then
    local port_pid
    port_pid=$(netstat -ano 2>/dev/null | grep ":${PLAYWRIGHT_MCP_PORT}" | grep LISTENING | awk '{print $NF}' | head -1 || true)
    if [ -n "$port_pid" ] && [ "$port_pid" != "0" ]; then
      echo "  Port ${PLAYWRIGHT_MCP_PORT} occupied by zombie PID $port_pid. Cleaning up..."
      taskkill /PID "$port_pid" /F 2>/dev/null || true
      sleep 1
    fi
  fi

  # 确保 Chromium 已安装（@playwright/mcp 不会自动安装浏览器）
  npx playwright install chromium 2>/dev/null || echo "  [WARN] Cannot install Chromium. Server may fail."

  # 启动新服务器
  echo "  Starting Playwright MCP server on port ${PLAYWRIGHT_MCP_PORT}..."
  nohup npx --yes @playwright/mcp@latest \
    --port "${PLAYWRIGHT_MCP_PORT}" \
    --host 127.0.0.1 \
    --allowed-hosts '*' \
    --headless \
    --browser chromium \
    --no-sandbox \
    > "${RALPH_DIR}/playwright-mcp.log" 2>&1 &
  local server_pid=$!
  echo "$server_pid" > "$PLAYWRIGHT_MCP_PID_FILE"

  for _ in $(seq 1 30); do
    if ! _is_process_alive "$server_pid"; then
      echo "  [ERROR] Playwright MCP died during startup."
      tail -20 "${RALPH_DIR}/playwright-mcp.log" 2>/dev/null || true
      rm -f "$PLAYWRIGHT_MCP_PID_FILE"
      return 1
    fi
    if curl -s --max-time 2 "http://127.0.0.1:${PLAYWRIGHT_MCP_PORT}/mcp" >/dev/null 2>&1; then
      echo "  Playwright MCP ready (PID: $server_pid)"
      return 0
    fi
    sleep 1
  done

  echo "  [WARN] Playwright MCP did not start within 30s."
  return 1
}

stop_playwright_mcp_server() {
  if [ -f "$PLAYWRIGHT_MCP_PID_FILE" ]; then
    local server_pid
    server_pid=$(cat "$PLAYWRIGHT_MCP_PID_FILE")
    if [ -n "$server_pid" ]; then
      _kill_process_tree "$server_pid" 2>/dev/null || true
      echo "  Playwright MCP server stopped (PID: $server_pid)."
    fi
    rm -f "$PLAYWRIGHT_MCP_PID_FILE"
  fi
}

run_preflight_checks() {
  echo ""
  echo "--- Pre-flight Tool Checks ---"

  local all_ok=true

  # ---- Required CLI tools ----
  ensure_tool "jq" "jq --version" \
    "brew install jq 2>/dev/null || apt-get install -y jq 2>/dev/null || choco install jq -y 2>/dev/null" \
    "CLI" "JSON processing (ralph.sh core)" || all_ok=false

  ensure_tool "git" "git --version" \
    "brew install git 2>/dev/null || apt-get install -y git 2>/dev/null || choco install git -y 2>/dev/null" \
    "CLI" "version control" || all_ok=false

  # ---- Optional tools (warn but don't block) ----
  # Phase 1: npm 包可用性检查
  ensure_tool "Playwright MCP" "npx --yes @playwright/mcp@latest --help 2>/dev/null" \
    "npx --yes @playwright/mcp@latest install 2>/dev/null || npx playwright install chromium 2>/dev/null" \
    "MCP" "browser testing (Evaluator)" || echo "  [WARN] Playwright npm package not available"

  # Phase 2: 启动 HTTP 服务器（包可用才尝试）
  if npx --yes @playwright/mcp@latest --help >/dev/null 2>&1; then
    start_playwright_mcp_server || echo "  [WARN] Playwright MCP server failed to start. Evaluator UI testing will be degraded."
  fi

  ensure_tool "Context7 MCP" "npx --yes @upstash/context7-mcp --help 2>/dev/null" \
    "true" \
    "MCP" "documentation lookup (Generator)" || echo "  [WARN] Context7 not available — Generator will rely on WebSearch only"

  if [ "$all_ok" = false ]; then
    echo ""
    echo "ERROR: Required tools are missing and could not be auto-installed."
    echo "Fix the issues above and re-run Ralph."
    exit 1
  fi

  echo "  All required tools present."
}

# ============================================================


# ============================================================
# Cost tracking
# ============================================================
COST_LOG_FILE="$RALPH_DIR/cost-log.txt"
AUDIT_LOG_FILE="$RALPH_DIR/audit-log.txt"

cost_track_start() {
  if [ "$TRACK_COST" = true ]; then
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)|start|$1" >> "$COST_LOG_FILE"
  fi
}

cost_track_end() {
  if [ "$TRACK_COST" = true ]; then
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)|end|$1" >> "$COST_LOG_FILE"
  fi
}

audit_log() {
  if [ "$AUDIT" = true ]; then
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)|$1" >> "$AUDIT_LOG_FILE"
  fi
}

generate_cost_report() {
  if [ "$TRACK_COST" = false ]; then
    return
  fi

  echo ""
  echo "==============================================================="
  echo "  Cost Report"
  echo "==============================================================="

  local total_phases=0
  local total_duration=0

  while IFS='|' read -r ts action phase; do
    if [ "$action" = "start" ]; then
      local start_ts="$ts"
      # Find matching end
      while IFS='|' read -r ts2 action2 phase2; do
        if [ "$action2" = "end" ] && [ "$phase2" = "$phase" ]; then
          local start_epoch end_epoch
          start_epoch=$(date -d "$start_ts" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$start_ts" +%s 2>/dev/null || echo 0)
          end_epoch=$(date -d "$ts2" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$ts2" +%s 2>/dev/null || echo 0)
          if [ "$start_epoch" != "0" ] && [ "$end_epoch" != "0" ]; then
            local duration=$((end_epoch - start_epoch))
            local minutes=$((duration / 60))
            local seconds=$((duration % 60))
            echo "  $phase: ${minutes}m ${seconds}s"
            total_phases=$((total_phases + 1))
            total_duration=$((total_duration + duration))
          fi
          break
        fi
      done < "$COST_LOG_FILE"
    fi
  done < "$COST_LOG_FILE"

  if [ "$total_duration" -gt 0 ]; then
    local total_min=$((total_duration / 60))
    local total_sec=$((total_duration % 60))
    echo "  ---"
    echo "  Total phases: $total_phases"
    echo "  Total duration: ${total_min}m ${total_sec}s"
    local cost_raw
    cost_raw=$(echo "scale=2; $total_duration / 60 * 0.50" | bc 2>/dev/null || echo "0")
    echo "  (Rough cost estimate: ${cost_raw} at ~$0.50/min avg)"
  fi
}

# ============================================================
# Helper: Get next pending story
# ============================================================
get_pending_story_id() {
  jq -r '[.userStories[] | select(.passes == false)] | sort_by(.priority) | .[0].id // empty' "$PRD_FILE"
}

# ============================================================
# Helper: Count pending stories
# ============================================================
count_pending_stories() {
  jq -r '[.userStories[] | select(.passes == false)] | length' "$PRD_FILE"
}

# ============================================================
# Helper: Check if all stories pass
# ============================================================
all_stories_pass() {
  local pending
  pending=$(count_pending_stories)
  [[ "$pending" -eq 0 ]]
}

# ============================================================
# Helper: Mark story as passed in prd.json
# ============================================================
mark_story_passed() {
  local story_id="$1"
  local tmp_file="${PRD_FILE}.tmp"
  jq --arg id "$story_id" \
    '(.userStories[] | select(.id == $id) | .passes) |= true' \
    "$PRD_FILE" > "$tmp_file"
  mv "$tmp_file" "$PRD_FILE"
}

# ============================================================
# Helper: Update evaluation in prd.json
# ============================================================
update_prd_evaluation() {
  local story_id="$1"
  local tmp_file="${PRD_FILE}.tmp"

  if [ -f "$EVALUATION_FILE" ]; then
    local overall_pass
    overall_pass=$(jq -r '.overallPass // false' "$EVALUATION_FILE" 2>/dev/null || echo "false")
    local overall_score
    overall_score=$(jq -r '.overallScore // 0' "$EVALUATION_FILE" 2>/dev/null || echo "0")
    local retry_attempt
    retry_attempt=$(jq -r '.retryAttempt // 0' "$EVALUATION_FILE" 2>/dev/null || echo "0")

    jq --arg id "$story_id" \
       --argjson pass "$overall_pass" \
       --argjson score "$overall_score" \
       --argjson retry "$retry_attempt" \
       --argjson func_score "$(jq -r '.scores.functionality.score // 0' "$EVALUATION_FILE")" \
       --argjson func_pass "$(jq -r '.scores.functionality.pass // false' "$EVALUATION_FILE")" \
       --argjson code_score "$(jq -r '.scores.codeQuality.score // 0' "$EVALUATION_FILE")" \
       --argjson code_pass "$(jq -r '.scores.codeQuality.pass // false' "$EVALUATION_FILE")" \
       --argjson design_score "$(jq -r '.scores.designQuality.score // 0' "$EVALUATION_FILE")" \
       --argjson design_pass "$(jq -r '.scores.designQuality.pass // false' "$EVALUATION_FILE")" \
       --argjson depth_score "$(jq -r '.scores.productDepth.score // 0' "$EVALUATION_FILE")" \
       --argjson depth_pass "$(jq -r '.scores.productDepth.pass // false' "$EVALUATION_FILE")" \
       --arg feedback "$(jq -r '.feedback // ""' "$EVALUATION_FILE")" \
      '(.userStories[] | select(.id == $id) | .retryCount) |= $retry |
       (.userStories[] | select(.id == $id) | .evaluation.overallScore) |= $score |
       (.userStories[] | select(.id == $id) | .evaluation.overallPass) |= $pass |
       (.userStories[] | select(.id == $id) | .evaluation.feedback) |= $feedback |
       (.userStories[] | select(.id == $id) | .evaluation.functionality.score) |= $func_score |
       (.userStories[] | select(.id == $id) | .evaluation.functionality.pass) |= $func_pass |
       (.userStories[] | select(.id == $id) | .evaluation.codeQuality.score) |= $code_score |
       (.userStories[] | select(.id == $id) | .evaluation.codeQuality.pass) |= $code_pass |
       (.userStories[] | select(.id == $id) | .evaluation.designQuality.score) |= $design_score |
       (.userStories[] | select(.id == $id) | .evaluation.designQuality.pass) |= $design_pass |
       (.userStories[] | select(.id == $id) | .evaluation.productDepth.score) |= $depth_score |
       (.userStories[] | select(.id == $id) | .evaluation.productDepth.pass) |= $depth_pass' \
      "$PRD_FILE" > "$tmp_file"
    mv "$tmp_file" "$PRD_FILE"
  fi
}

# ============================================================
# Helper: Get contract status
# ============================================================
get_contract_status() {
  if [ -f "$CONTRACT_FILE" ]; then
    jq -r '.status // "unknown"' "$CONTRACT_FILE" 2>/dev/null || echo "parse_error"
  else
    echo "none"
  fi
}

# ============================================================
# Helper: Verify contract integrity (detect tampering after lock)
# ============================================================
verify_contract_integrity() {
  local expected_status="$1"
  local actual_status
  actual_status=$(get_contract_status)

  if [[ "$actual_status" != "$expected_status" ]]; then
    echo "ERROR: contract.json status is '$actual_status', expected '$expected_status'. Contract may have been tampered with."
    return 1
  fi
  return 0
}

# ============================================================
# MODE: simple (original single-agent behavior)
# ============================================================
run_simple_mode() {
  set -e  # 核心业务逻辑：启用错误检测
  echo "Starting Ralph - Mode: $MODE - Tool: $TOOL - Max iterations: $MAX_ITERATIONS"

  for i in $(seq 1 $MAX_ITERATIONS); do
    echo ""
    echo "==============================================================="
    echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL, simple)"
    echo "==============================================================="

    set_phase "simple"
    OUTPUT=$(run_agent "$LEGACY_PROMPT" "simple-iteration-$i")

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
}

# ============================================================
# Audit report generator
# ============================================================
generate_audit_report() {
  if [ "$AUDIT" = false ]; then
    return
  fi

  echo ""
  echo "==============================================================="
  echo "  Evaluator Audit Report"
  echo "==============================================================="

  if [ ! -f "$PRD_FILE" ]; then
    echo "  No prd.json found. Skipping audit."
    return
  fi

  # Per-story evaluation summary
  echo ""
  echo "  Story Evaluation Summary:"
  echo "  -------------------------"
  jq -r '.userStories[] | "  \(.id): \(.title)\n    Passed: \(.passes) | BestEffort: \(.bestEffort // false) | Retries: \(.retryCount // 0)\n    Score: \(.evaluation.overallScore // "N/A") | F:\(.evaluation.functionality.score // "?") | C:\(.evaluation.codeQuality.score // "?") | D:\(.evaluation.designQuality.score // "?") | P:\(.evaluation.productDepth.score // "?")"' "$PRD_FILE"

  # Flag potential evaluator strictness/leniency issues
  echo ""
  echo "  Potential Issues:"
  echo "  -----------------"

  # Find stories that passed with very low scores (possible leniency)
  local low_score_passes
  low_score_passes=$(jq -r '[.userStories[] | select(.passes == true and .evaluation.overallScore != null and .evaluation.overallScore < 65)] | length' "$PRD_FILE")
  if [ "$low_score_passes" -gt 0 ]; then
    echo "  WARNING: $low_score_passes story(s) passed with score < 65 (possible evaluator leniency):"
    jq -r '.userStories[] | select(.passes == true and .evaluation.overallScore != null and .evaluation.overallScore < 65) | "    - \(.id) (score: \(.evaluation.overallScore))"' "$PRD_FILE"
  fi

  # Find stories that failed despite high scores (possible evaluator strictness)
  local high_score_fails
  high_score_fails=$(jq -r '[.userStories[] | select(.passes == false and .evaluation.overallScore != null and .evaluation.overallScore >= 70)] | length' "$PRD_FILE")
  if [ "$high_score_fails" -gt 0 ]; then
    echo "  NOTE: $high_score_fails story(s) failed despite score >= 70 (possible evaluator strictness):"
    jq -r '.userStories[] | select(.passes == false and .evaluation.overallScore != null and .evaluation.overallScore >= 70) | "    - \(.id) (score: \(.evaluation.overallScore))"' "$PRD_FILE"
  fi

  # Summary statistics
  echo ""
  echo "  Summary Statistics:"
  echo "  -------------------"
  local total_stories
  total_stories=$(jq -r '.userStories | length' "$PRD_FILE")
  local passed
  passed=$(jq -r '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE")
  local best_effort_count
  best_effort_count=$(jq -r '[.userStories[] | select(.bestEffort == true)] | length' "$PRD_FILE")
  local avg_score
  avg_score=$(jq -r '[.userStories[] | .evaluation.overallScore // 0] | add / length | . * 100 | round / 100' "$PRD_FILE" 2>/dev/null || echo "N/A")

  echo "  Total stories: $total_stories"
  echo "  Passed: $passed"
  echo "  Best-effort passes: $best_effort_count"
  echo "  Average evaluation score: $avg_score"
  echo "  Audit log saved to: $AUDIT_LOG_FILE"
}

# MODE: harness (Generator-Evaluator architecture)
# ============================================================
run_harness_mode() {
  set -e  # 核心业务逻辑：启用错误检测
  echo "Starting Ralph - Mode: $MODE - Tool: $TOOL"
  echo "Max retries per story: $MAX_RETRIES"
  echo "Max contract rounds: $MAX_CONTRACT_ROUNDS"
  if [ "$AUDIT" = true ]; then
    echo "Audit mode: report will be generated"
  fi
  echo ""

  # Check prerequisites
  if [ ! -f "$PRD_FILE" ]; then
    echo "Error: prd.json not found at $PRD_FILE"
    exit 1
  fi

  if [ ! -f "$GENERATOR_PROMPT" ]; then
    echo "Error: generator-prompt.md not found at $GENERATOR_PROMPT"
    exit 1
  fi

  if [ ! -f "$EVALUATOR_PROMPT" ]; then
    echo "Error: evaluator-prompt.md not found at $EVALUATOR_PROMPT"
    exit 1
  fi

  # Pre-flight tool checks (auto-install CLI > MCP, pause if manual intervention needed)
  run_preflight_checks



  local iteration=0

  while true; do
    iteration=$((iteration + 1))

    if [ "$iteration" -gt "$MAX_ITERATIONS" ]; then
      echo ""
      echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
      echo "Check $PROGRESS_FILE for status."
      exit 1
    fi

    # Find next pending story
    local story_id
    story_id=$(get_pending_story_id)

    if [ -z "$story_id" ]; then
      echo ""
      echo "No pending stories found. Checking if all complete..."
      if all_stories_pass; then
        echo "All stories pass! Ralph is done."
        exit 0
      fi
      echo "Unexpected: no pending stories but not all pass. Check prd.json."
      exit 1
    fi

    local story_title
    story_title=$(jq -r --arg id "$story_id" '.userStories[] | select(.id == $id) | .title' "$PRD_FILE")

    echo ""
    echo "==============================================================="
    echo "  Story: $story_id - $story_title (Iteration $iteration)"
    echo "==============================================================="

    contract_locked=false  # Reset for each story

    # Check for pending user resolution from previous failed negotiation
    if [ -f "${RALPH_DIR}/user-resolution.md" ]; then
      echo "  Detected user-resolution.md for $story_id."
      echo ""
      echo "  ==============================================================="
      echo "  USER RESOLUTION CONTENT:"
      echo "  ---------------------------------------------------------------"
      cat "${RALPH_DIR}/user-resolution.md"
      echo "  ---------------------------------------------------------------"
      echo ""
      echo "  This resolution will be sent to the Evaluator for formal review."
      echo "  Only proceed if you (the human user) wrote or approved this."
      echo "  ==============================================================="
      local confirm="n"
      if [ -t 0 ]; then
        read -r -p "  Send to Evaluator? [y/N]: " confirm
      else
        echo "  stdin unavailable — defaulting to 'n' (skip user-resolution)"
      fi
      if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo "  Aborted. Remove .ralph/user-resolution.md and re-run ralph.sh."
        RALPH_NORMAL_EXIT=true
        exit 1
      fi
      echo "  Confirmed. Sending to Evaluator..."
      if run_evaluator_user_resolution "$story_id" "harness"; then
        echo "  Evaluator approved the resolution. Contract LOCKED."
        rm -f "${RALPH_DIR}/user-resolution.md"
        rm -f "${RALPH_DIR}/contract-failure-summary.md"
        contract_locked=true
      else
        echo "  Evaluator rejected the user resolution."
        write_evaluator_feedback "$story_id"
        exit_for_user_resolution "$story_id" "$story_title"
      fi
    else
      # Normal flow: start contract negotiation
      :
    fi

    # ============================================================
    # Phase 1-2: Contract negotiation
    # ============================================================
    echo ""
    echo "--- Contract Negotiation Phase ---"

    local contract_locked="${contract_locked:-false}"

    if [ "$contract_locked" = "true" ]; then
      echo "  Contract already locked (user-resolution). Skipping negotiation."
    else
      # Clean up any leftover contract from previous story
      rm -f "$CONTRACT_FILE"
      rm -f "${RALPH_DIR}/contract-round-"*.json
      rm -f "${RALPH_DIR}/contract-scores.txt"

      for round in $(seq 1 $MAX_CONTRACT_ROUNDS); do
        echo ""
        echo "  Contract Round $round of $MAX_CONTRACT_ROUNDS"

        # Run Generator (contract mode)
        echo "  [Generator] Drafting/revising contract..."
        set_phase "generator-contract"
      run_agent "$GENERATOR_PROMPT" "contract-round-${round}-generator-${story_id}"

      verify_contract_phase_output || break

      # Run Evaluator (contract mode)
      echo "  [Evaluator] Reviewing and scoring contract..."
      set_phase "evaluator-contract"
      run_agent "$EVALUATOR_PROMPT" "contract-round-${round}-evaluator-${story_id}"

      verify_evaluator_contract_output

      # Save this round's contract and its score
      local round_score
      round_score=$(jq -r '.score // 0' "$CONTRACT_FILE" 2>/dev/null || echo "0")
      cp "$CONTRACT_FILE" "${RALPH_DIR}/contract-round-${round}.json"
      echo "$round $round_score" >> "${RALPH_DIR}/contract-scores.txt"
      echo "  Contract score: $round_score/100"

      local contract_status
      contract_status=$(get_contract_status)

      case "$contract_status" in
        locked)
          echo "  Contract LOCKED by Evaluator. Proceeding to build."
          contract_locked=true
          break
          ;;
        generator_revise)
          echo "  Evaluator returned contract for revision."
          # Loop continues — Generator will revise
          ;;
        proposed)
          echo "  WARNING: Evaluator did not change status from 'proposed'. Re-running contract round."
          ;;
        *)
          echo "  ERROR: Unknown contract status: $contract_status"
          break
          ;;
      esac
    done
    fi

    # BUG2 fix: if last round was rejected, give Generator one extra chance to revise
    if [ "$contract_locked" = false ] && [ -f "$CONTRACT_FILE" ]; then
      local last_status
      last_status=$(get_contract_status)
      if [ "$last_status" = "generator_revise" ]; then
        echo ""
        echo "  Last round was rejected. Giving Generator one extra revision chance..."
        set_phase "generator-contract"
        run_agent "$GENERATOR_PROMPT" "contract-extra-round-generator-${story_id}"

        if [ -f "$CONTRACT_FILE" ]; then
          set_phase "evaluator-contract"
          run_agent "$EVALUATOR_PROMPT" "contract-extra-round-evaluator-${story_id}"

          if [ -f "$CONTRACT_FILE" ]; then
            local extra_score
            extra_score=$(jq -r '.score // 0' "$CONTRACT_FILE" 2>/dev/null || echo "0")
            local extra_round=$((MAX_CONTRACT_ROUNDS + 1))
            cp "$CONTRACT_FILE" "${RALPH_DIR}/contract-round-${extra_round}.json"
            echo "$extra_round $extra_score" >> "${RALPH_DIR}/contract-scores.txt"

            local extra_status
            extra_status=$(get_contract_status)
            if [ "$extra_status" = "locked" ]; then
              echo "  Contract LOCKED after extra round. Proceeding to build."
              contract_locked=true
            else
              echo "  Extra round result: $extra_status (score: $extra_score)"
            fi
          fi
        fi
      fi
    fi

    if [ "$contract_locked" = false ]; then
      echo ""
      exit_for_user_resolution "$story_id" "$story_title"
    fi

    # ============================================================
    # Phase 3-4: Build + Evaluate loop
    # ============================================================
    echo ""
    echo "--- Build & Evaluate Phase ---"

    # 清理上次运行残留的 proposed 合同（非 locked = 协商未完成）
    if [ -f "$CONTRACT_FILE" ]; then
      local contract_status
      contract_status=$(jq -r '.status // "unknown"' "$CONTRACT_FILE" 2>/dev/null || echo "unknown")
      if [ "$contract_status" != "locked" ]; then
        echo "  Cleaning up stale contract (status: $contract_status)..."
        rm -f "$CONTRACT_FILE"
      fi
    fi

    # Clean up previous evaluation, build signal, and backups
    rm -f "$EVALUATION_FILE" "${RALPH_DIR}/build-done"
    rm -f "${RALPH_DIR}/evaluation-retry-"*.json
    rm -f "${RALPH_DIR}/evaluation-scores.txt"

    local story_passed=false
    local best_effort=false

    for retry in $(seq 0 $MAX_RETRIES); do
      if [ "$retry" -gt 0 ]; then
        echo ""
        echo "  Retry $retry of $MAX_RETRIES for $story_id"

        # Generate changes-summary for retry
        local changes_file="$CHANGES_FILE"
        echo "# 重试 $retry 增量变更摘要" > "$changes_file"
        echo "## 故事信息" >> "$changes_file"
        echo "- 故事ID: $story_id" >> "$changes_file"
        echo "- 故事标题: $story_title" >> "$changes_file"
        echo "" >> "$changes_file"

        if [ -f "$EVALUATION_FILE" ]; then
          echo "## 上次评估失败的验收标准" >> "$changes_file"
          jq -r '.verifiedCriteria[]? | select(.result == "FAIL") | "- [FAIL] \(.criterion)\n  证据: \(.evidence // "无")"' "$EVALUATION_FILE" >> "$changes_file" 2>/dev/null || echo "  (无明细)" >> "$changes_file"
          echo "" >> "$changes_file"

          echo "## 上次评估已通过的标准（本次跳过）" >> "$changes_file"
          jq -r '.verifiedCriteria[]? | select(.result == "PASS") | "- [PASS] \(.criterion)"' "$EVALUATION_FILE" >> "$changes_file" 2>/dev/null || echo "  (无)" >> "$changes_file"
          echo "" >> "$changes_file"

          echo "## 上次评估反馈摘要" >> "$changes_file"
          local prev_feedback
          prev_feedback=$(jq -r '.feedback // "无"' "$EVALUATION_FILE")
          echo "$prev_feedback" | head -15 >> "$changes_file"
        fi

        echo ""
        echo "  Changes summary generated for retry."
      fi

      # Delete old evaluation and build signal to prevent stale scores
      rm -f "$EVALUATION_FILE" "${RALPH_DIR}/build-done"

      # Verify contract is still locked before build
      if ! verify_contract_integrity "locked"; then
        echo "  Contract integrity check failed. Aborting this story."
        break
      fi

      # Run Generator (build mode)
      echo "  [Generator] Implementing story..."
      set_phase "generator-build"
      local gen_start_head
      gen_start_head=$(git rev-parse HEAD 2>/dev/null)
      run_agent "$GENERATOR_PROMPT" "build-retry-${retry}-generator-${story_id}"

      # Detect Generator crash: no commits AND no uncommitted changes = produced nothing
      local gen_end_head
      gen_end_head=$(git rev-parse HEAD 2>/dev/null)
      local has_work=false
      if [ "$gen_end_head" != "$gen_start_head" ]; then
        has_work=true
      fi
      if ! git diff --quiet 2>/dev/null || [ -n "$(git ls-files --others --exclude-standard 2>/dev/null)" ]; then
        has_work=true
      fi
      if [ "$has_work" = false ]; then
        echo "  [CRASH] Generator exited without producing code. Retrying..."
        continue
      fi

      # Verify contract integrity after Generator run
      if ! verify_contract_integrity "locked"; then
        echo "  WARNING: Generator may have modified locked contract. This build is invalid."
        local tmp_file="${PRD_FILE}.tmp"
        jq --arg id "$story_id" \
          '(.userStories[] | select(.id == $id) | .retryCount) |= (. // 0) + 1' \
          "$PRD_FILE" > "$tmp_file"
        mv "$tmp_file" "$PRD_FILE"
        continue
      fi

      # Update changes-summary with actual changed files
      local changes_file="$CHANGES_FILE"
      if [ "$retry" -gt 0 ]; then
        echo "" >> "$changes_file"
        echo "## 本次重试实际改动的文件（用于 Evaluator 增量评估）" >> "$changes_file"
        git diff --name-only HEAD~1 2>/dev/null | sed 's/^/- /' >> "$changes_file" || echo "- (无法获取 git diff)" >> "$changes_file"
      else
        # First build: capture full diff for reference
        echo "# 首次构建变更摘要 ($story_id)" > "$changes_file"
        echo "## 本故事改动的文件" >> "$changes_file"
        git diff --name-only HEAD~1 2>/dev/null | sed 's/^/- /' >> "$changes_file" || echo "- (首次提交，无 diff)" >> "$changes_file"
      fi
      echo "  Updated changes-summary with git diff."

      # Run Evaluator (evaluate mode)
      echo "  [Evaluator] Testing and scoring..."
      set_phase "evaluator-evaluate"
      run_agent "$EVALUATOR_PROMPT" "build-retry-${retry}-evaluator-${story_id}"

      verify_evaluator_evaluate_output

      # Check evaluation results
      if [ ! -f "$EVALUATION_FILE" ]; then
        echo "  [CRASH] Evaluator exited without writing evaluation.json. Retrying..."
        continue
      fi

      # Save this retry's evaluation and its score
      local retry_score
      retry_score=$(jq -r '.overallScore // 0' "$EVALUATION_FILE")
      local retry_attempt
      retry_attempt=$(jq -r '.retryAttempt // 0' "$EVALUATION_FILE")
      cp "$EVALUATION_FILE" "${RALPH_DIR}/evaluation-retry-${retry_attempt}.json"
      echo "$retry_attempt $retry_score" >> "${RALPH_DIR}/evaluation-scores.txt"

      # Update prd.json with evaluation data
      update_prd_evaluation "$story_id"

      local overall_pass
      overall_pass=$(jq -r '.overallPass // false' "$EVALUATION_FILE")

      echo "  Score: $retry_score/100 | Pass: $overall_pass"

      # Degradation detection: abort if scores are trending down
      if [ "$retry" -ge 2 ]; then
        local prev1 prev2
        prev1=$(grep "^$((retry - 1)) " "${RALPH_DIR}/evaluation-scores.txt" | awk '{print $2}' 2>/dev/null || echo "")
        prev2=$(grep "^$((retry - 2)) " "${RALPH_DIR}/evaluation-scores.txt" | awk '{print $2}' 2>/dev/null || echo "")
        if [ -n "$prev1" ] && [ -n "$prev2" ] && [ -n "$retry_score" ]; then
          if [ "$retry_score" -lt "$prev1" ] && [ "$prev1" -lt "$prev2" ]; then
            echo "  DEGRADATION DETECTED: Scores trending down ($prev2 → $prev1 → $retry_score)"
            echo "  Aborting retries early — Generator is making things worse."
            audit_log "degradation-detected|${story_id}|scores:${prev2}→${prev1}→${retry_score}|retry:${retry}"
            break
          fi
        fi
      fi

      if [ "$overall_pass" = "true" ]; then
        echo "  Story $story_id PASSED evaluation!"
        mark_story_passed "$story_id"
        story_passed=true
        break
      else
        echo "  Story $story_id FAILED evaluation (retry $retry of $MAX_RETRIES)."
        local feedback_preview
        feedback_preview=$(jq -r '.feedback // ""' "$EVALUATION_FILE" | head -3)
        echo "  Feedback: $feedback_preview"
      fi
    done

    if [ "$story_passed" = false ]; then
      echo ""
      echo "All $MAX_RETRIES retries exhausted for $story_id. Selecting best effort..."

      # Find the retry with the highest score
      local best_retry
      best_retry=$(cat "${RALPH_DIR}/evaluation-scores.txt" | sort -k2 -nr | head -1 | awk '{print $1}')

      if [ -n "$best_retry" ] && [ -f "${RALPH_DIR}/evaluation-retry-${best_retry}.json" ]; then
        local best_score
        best_score=$(cat "${RALPH_DIR}/evaluation-scores.txt" | sort -k2 -nr | head -1 | awk '{print $2}')
        echo "  Best result: retry $best_retry (score: $best_score/100)"

        # Restore best evaluation
        cp "${RALPH_DIR}/evaluation-retry-${best_retry}.json" "$EVALUATION_FILE"

        # Update prd.json with best-effort pass
        update_prd_evaluation "$story_id"
        local tmp_file="${PRD_FILE}.tmp"
        jq --arg id "$story_id" \
           --arg note "BEST-EFFORT: ${MAX_RETRIES}次重试均未达标，选择第${best_retry}次（评分${best_score}/100）作为最终结果" \
           '(.userStories[] | select(.id == $id) | .passes) |= true |
            (.userStories[] | select(.id == $id) | .bestEffort) |= true |
            (.userStories[] | select(.id == $id) | .notes) |= (if . == "" then $note else . + " | " + $note end)' \
           "$PRD_FILE" > "$tmp_file"
        mv "$tmp_file" "$PRD_FILE"

        echo "  Story $story_id marked as passed (best-effort)."
        story_passed=true
      else
        echo "  ERROR: No evaluation backups found. Story failed."
        # Force-skip to prevent infinite re-negotiation loop
        local tmp_file="${PRD_FILE}.tmp"
        jq --arg id "$story_id" \
          '(.userStories[] | select(.id == $id) | .passes) |= true |
           (.userStories[] | select(.id == $id) | .bestEffort) |= true |
           (.userStories[] | select(.id == $id) | .notes) |= (if . == "" then "MANUAL REVIEW: Evaluator produced no valid evaluation" else . + " | MANUAL REVIEW: no evaluation" end)' \
          "$PRD_FILE" > "$tmp_file"
        mv "$tmp_file" "$PRD_FILE"
      fi
    fi

    # Check if all stories are done
    if all_stories_pass; then
      echo ""
      echo "==============================================================="
      echo "  ALL STORIES COMPLETE!"
      echo "  Total iterations: $iteration"
      echo "==============================================================="
      generate_audit_report
      generate_cost_report
      exit 0
    fi

    # Clean up for next story
    rm -f "$CONTRACT_FILE" "$EVALUATION_FILE" "$CHANGES_FILE"

    if [ "$ONE_SHOT" = true ]; then
      if all_stories_pass; then
        echo "All stories complete. (one-shot)"
        RALPH_NORMAL_EXIT=true
        exit 0
      else
        echo "One story done. Exit for next invocation. (one-shot)"
        RALPH_NORMAL_EXIT=true
        exit 1
      fi
    fi
  done

  # Max iterations reached without completing
  generate_audit_report
  generate_cost_report
}

# ============================================================
# Main entry point
# ============================================================
case "$MODE" in
  simple)
    run_simple_mode
    ;;
  harness)
    run_harness_mode
    ;;
esac
