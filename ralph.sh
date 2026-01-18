#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--worker amp|cursor] [max_iterations]
#
# Workers:
#   cursor (default) - Uses Cursor CLI 'agent' command
#   amp              - Uses Amp CLI 'amp' command

set -e

# ═══════════════════════════════════════════════════════
# Parse Arguments
# ═══════════════════════════════════════════════════════

WORKER="cursor"  # Default worker
MAX_ITERATIONS=10

while [[ $# -gt 0 ]]; do
  case $1 in
    --worker|-w)
      WORKER="$2"
      shift 2
      ;;
    --help|-h)
      echo "Usage: ./ralph.sh [--worker amp|cursor] [max_iterations]"
      echo ""
      echo "Workers:"
      echo "  cursor (default) - Uses Cursor CLI 'agent' command"
      echo "  amp              - Uses Amp CLI 'amp' command"
      echo ""
      echo "Examples:"
      echo "  ./ralph.sh                    # Run with cursor, 10 iterations"
      echo "  ./ralph.sh 20                 # Run with cursor, 20 iterations"
      echo "  ./ralph.sh --worker amp 15    # Run with amp, 15 iterations"
      echo "  ./ralph.sh -w cursor 10       # Run with cursor, 10 iterations"
      exit 0
      ;;
    *)
      # Assume it's max_iterations if it's a number
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      else
        echo "Unknown option: $1"
        echo "Use --help for usage"
        exit 1
      fi
      shift
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ═══════════════════════════════════════════════════════
# Worker Configuration
# ═══════════════════════════════════════════════════════

# Validate worker and check for required commands
case $WORKER in
  cursor)
    if ! command -v agent &> /dev/null; then
      echo "Error: 'agent' command not found. Please install Cursor CLI: https://cursor.com/docs/cli"
      exit 1
    fi
    WORKER_NAME="Cursor CLI"
    ;;
  amp)
    if ! command -v amp &> /dev/null; then
      echo "Error: 'amp' command not found. Please install Amp: https://ampcode.com"
      exit 1
    fi
    WORKER_NAME="Amp"
    ;;
  # Add new workers here:
  # newworker)
  #   if ! command -v newworker &> /dev/null; then
  #     echo "Error: 'newworker' command not found."
  #     exit 1
  #   fi
  #   WORKER_NAME="New Worker"
  #   ;;
  *)
    echo "Error: Unknown worker '$WORKER'"
    echo "Available workers: cursor, amp"
    exit 1
    ;;
esac

if ! command -v jq &> /dev/null; then
  echo "Error: 'jq' command not found. Please install jq: brew install jq"
  exit 1
fi

# ═══════════════════════════════════════════════════════
# Worker Functions
# ═══════════════════════════════════════════════════════

run_cursor_agent() {
  local project_root="$1"
  local prompt_file="$2"
  
  # --print flag is required for non-interactive mode and enables shell execution
  # --force flag forces allow commands unless explicitly denied
  # --workspace sets the working directory
  agent --print --force --workspace "$project_root" --output-format text "$(cat "$prompt_file")" 2>&1 | tee /dev/stderr
}

run_amp_agent() {
  local project_root="$1"
  local prompt_file="$2"
  
  # Change to project directory for amp
  cd "$project_root"
  
  # amp uses different flags:
  # --yes to auto-approve commands
  # --print for output
  amp --yes --print "$(cat "$prompt_file")" 2>&1 | tee /dev/stderr
}

# Add new worker functions here:
# run_newworker_agent() {
#   local project_root="$1"
#   local prompt_file="$2"
#   newworker --some-flag "$project_root" "$(cat "$prompt_file")" 2>&1 | tee /dev/stderr
# }

run_agent() {
  local project_root="$1"
  local prompt_file="$2"
  
  case $WORKER in
    cursor)
      run_cursor_agent "$project_root" "$prompt_file"
      ;;
    amp)
      run_amp_agent "$project_root" "$prompt_file"
      ;;
    # Add new workers here:
    # newworker)
    #   run_newworker_agent "$project_root" "$prompt_file"
    #   ;;
  esac
}

# ═══════════════════════════════════════════════════════
# Project Setup
# ═══════════════════════════════════════════════════════

# Find project root (where prd.json should be located)
# Check script directory first, then parent directories up to 3 levels
PROJECT_ROOT="$SCRIPT_DIR"
for i in {0..3}; do
  if [ -f "$PROJECT_ROOT/prd.json" ]; then
    break
  fi
  if [ "$PROJECT_ROOT" = "/" ]; then
    # Fallback to script directory if not found
    PROJECT_ROOT="$SCRIPT_DIR"
    break
  fi
  PROJECT_ROOT="$(dirname "$PROJECT_ROOT")"
done

PRD_FILE="$PROJECT_ROOT/prd.json"
PROGRESS_FILE="$PROJECT_ROOT/progress.txt"
ARCHIVE_DIR="$PROJECT_ROOT/archive"
LAST_BRANCH_FILE="$PROJECT_ROOT/.last-branch"
PROMPT_FILE="$SCRIPT_DIR/prompt.md"

# ═══════════════════════════════════════════════════════
# Git Branch Setup (runs once at start)
# ═══════════════════════════════════════════════════════

setup_git_branch() {
  if [ ! -f "$PRD_FILE" ]; then
    echo "Warning: No prd.json found. Skipping git branch setup."
    return 0
  fi
  
  local target_branch
  target_branch=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  
  if [ -z "$target_branch" ]; then
    echo "Warning: No branchName in prd.json. Skipping git branch setup."
    return 0
  fi
  
  local current_branch
  current_branch=$(git branch --show-current 2>/dev/null || echo "")
  
  if [ "$current_branch" = "$target_branch" ]; then
    echo "✓ Already on branch: $target_branch"
    return 0
  fi
  
  echo "Setting up git branch: $target_branch"
  
  # Save old branch files for archiving BEFORE switching (if branch changed)
  # These will be used later for archiving the previous run
  local old_prd_file=""
  local old_progress_file=""
  if [ -n "$current_branch" ] && [ "$current_branch" != "$target_branch" ]; then
    if [ -f "$PRD_FILE" ]; then
      old_prd_file="${PRD_FILE}.ralph_archive_tmp"
      cp "$PRD_FILE" "$old_prd_file" 2>/dev/null || true
    fi
    if [ -f "$PROGRESS_FILE" ]; then
      old_progress_file="${PROGRESS_FILE}.ralph_archive_tmp"
      cp "$PROGRESS_FILE" "$old_progress_file" 2>/dev/null || true
    fi
  fi
  
  # Check for ANY changes (tracked OR untracked)
  local has_changes=0
  if ! git diff --quiet 2>/dev/null || ! git diff --cached --quiet 2>/dev/null; then
    has_changes=1
  fi
  
  # Check for untracked files (critical for prd.json/progress.txt)
  if [ -n "$(git ls-files --others --exclude-standard 2>/dev/null)" ]; then
    has_changes=1
  fi
  
  # Stash any changes (tracked or untracked)
  local STASHED=0
  if [ "$has_changes" = "1" ]; then
    echo "   Stashing uncommitted changes (including untracked files)..."
    git stash push --include-untracked -m "ralph: auto-stash before branch switch to $target_branch"
    STASHED=1
  fi
  
  # Store temp file paths for later use in archive management
  export RALPH_OLD_PRD_FILE="$old_prd_file"
  export RALPH_OLD_PROGRESS_FILE="$old_progress_file"
  
  # Check if branch exists locally
  if git show-ref --verify --quiet "refs/heads/$target_branch" 2>/dev/null; then
    echo "   Switching to existing branch..."
    if ! git checkout "$target_branch"; then
      echo "   ERROR: Failed to checkout branch $target_branch"
      if [ "$STASHED" = "1" ]; then
        echo "   Your changes are still stashed. Use 'git stash list' to view."
      fi
      # Clean up temp files
      [ -n "$old_prd_file" ] && [ -f "$old_prd_file" ] && rm -f "$old_prd_file"
      [ -n "$old_progress_file" ] && [ -f "$old_progress_file" ] && rm -f "$old_progress_file"
      exit 1
    fi
  else
    # Create branch from main (or master, or current)
    local base_branch="main"
    if ! git show-ref --verify --quiet "refs/heads/main" 2>/dev/null; then
      if git show-ref --verify --quiet "refs/heads/master" 2>/dev/null; then
        base_branch="master"
      else
        base_branch="$current_branch"
      fi
    fi
    echo "   Creating new branch from $base_branch..."
    if ! git checkout -b "$target_branch" "$base_branch"; then
      echo "   ERROR: Failed to create branch $target_branch from $base_branch"
      if [ "$STASHED" = "1" ]; then
        echo "   Your changes are still stashed. Use 'git stash list' to view."
      fi
      # Clean up temp files
      [ -n "$old_prd_file" ] && [ -f "$old_prd_file" ] && rm -f "$old_prd_file"
      [ -n "$old_progress_file" ] && [ -f "$old_progress_file" ] && rm -f "$old_progress_file"
      exit 1
    fi
  fi
  
  # Restore stashed changes
  if [ "$STASHED" = "1" ]; then
    echo "   Restoring stashed changes..."
    if ! git stash pop; then
      echo "   WARNING: Stash restore failed (possible conflicts)"
      echo "   Your stashed changes are still in git stash."
      echo "   Use 'git stash list' to view and 'git stash pop' to restore manually."
    fi
  fi
  
  # Verify critical files exist after branch switch
  if [ ! -f "$PRD_FILE" ]; then
    echo "   WARNING: prd.json not found after branch switch!"
    if [ "$STASHED" = "1" ]; then
      echo "   Check git stash: git stash list"
    fi
  fi
  
  if [ ! -f "$PROGRESS_FILE" ]; then
    echo "   WARNING: progress.txt not found after branch switch!"
    if [ "$STASHED" = "1" ]; then
      echo "   Check git stash: git stash list"
    fi
  fi
  
  echo "✓ Now on branch: $target_branch"
}

# ═══════════════════════════════════════════════════════
# Git Branch Setup Execution (runs before archive)
# ═══════════════════════════════════════════════════════

# Setup git branch before archive management
setup_git_branch

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "Worker: $WORKER_NAME" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

# ═══════════════════════════════════════════════════════
# Archive Management (runs after branch setup)
# ═══════════════════════════════════════════════════════

# Archive previous run if branch changed
# This runs AFTER setup_git_branch() to ensure we're on the correct branch
if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")
  
  if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
    # Archive files from the OLD branch (saved before branch switch)
    # Use temp files if they exist, otherwise fall back to current files
    DATE=$(date +%Y-%m-%d)
    # Strip "ralph/" prefix from branch name for folder
    FOLDER_NAME=$(echo "$LAST_BRANCH" | sed 's|^ralph/||')
    ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"
    
    echo "Archiving previous run: $LAST_BRANCH"
    # Use temp files saved before branch switch, if they exist
    old_prd="${RALPH_OLD_PRD_FILE:-}"
    old_progress="${RALPH_OLD_PROGRESS_FILE:-}"
    
    if mkdir -p "$ARCHIVE_FOLDER"; then
      
      # Archive prd.json from old branch
      if [ -n "$old_prd" ] && [ -f "$old_prd" ]; then
        if cp "$old_prd" "$ARCHIVE_FOLDER/prd.json" && [ -f "$ARCHIVE_FOLDER/prd.json" ]; then
          echo "   ✓ Archived prd.json from old branch"
          rm -f "$old_prd"
        else
          echo "   Warning: Failed to copy prd.json from old branch"
        fi
      elif [ -f "$PRD_FILE" ]; then
        # Fallback to current file if temp file doesn't exist
        if cp "$PRD_FILE" "$ARCHIVE_FOLDER/prd.json" && [ -f "$ARCHIVE_FOLDER/prd.json" ]; then
          echo "   ✓ Archived prd.json"
        else
          echo "   Warning: Failed to copy prd.json"
        fi
      fi
      
      # Archive progress.txt from old branch
      if [ -n "$old_progress" ] && [ -f "$old_progress" ]; then
        if cp "$old_progress" "$ARCHIVE_FOLDER/progress.txt" && [ -f "$ARCHIVE_FOLDER/progress.txt" ]; then
          echo "   ✓ Archived progress.txt from old branch"
          # Only reset after successful copy and verification
          echo "# Ralph Progress Log" > "$PROGRESS_FILE"
          echo "Started: $(date)" >> "$PROGRESS_FILE"
          echo "Worker: $WORKER_NAME" >> "$PROGRESS_FILE"
          echo "---" >> "$PROGRESS_FILE"
          rm -f "$old_progress"
        else
          echo "   Warning: Failed to copy progress.txt from old branch - NOT resetting"
        fi
      elif [ -f "$PROGRESS_FILE" ]; then
        # Fallback to current file if temp file doesn't exist
        if cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/progress.txt" && [ -f "$ARCHIVE_FOLDER/progress.txt" ]; then
          echo "   ✓ Archived progress.txt"
          # Only reset after successful copy and verification
          echo "# Ralph Progress Log" > "$PROGRESS_FILE"
          echo "Started: $(date)" >> "$PROGRESS_FILE"
          echo "Worker: $WORKER_NAME" >> "$PROGRESS_FILE"
          echo "---" >> "$PROGRESS_FILE"
        else
          echo "   Warning: Failed to copy progress.txt - NOT resetting"
        fi
      fi
      
      echo "   Archived to: $ARCHIVE_FOLDER"
    else
      echo "   Error: Failed to create archive directory: $ARCHIVE_FOLDER"
    fi
    
    # Clean up any remaining temp files
    [ -n "$old_prd" ] && [ -f "$old_prd" ] && rm -f "$old_prd"
    [ -n "$old_progress" ] && [ -f "$old_progress" ] && rm -f "$old_progress"
    
    # Clear exported variables
    unset RALPH_OLD_PRD_FILE
    unset RALPH_OLD_PROGRESS_FILE
  fi
fi

# Track current branch (after setup completes successfully)
if [ -f "$PRD_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  if [ -n "$CURRENT_BRANCH" ]; then
    echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"
  fi
fi

# ═══════════════════════════════════════════════════════
# Main Loop
# ═══════════════════════════════════════════════════════

echo ""
echo "╔═══════════════════════════════════════════════════════╗"
echo "║  Ralph - Autonomous AI Agent Loop                     ║"
echo "╠═══════════════════════════════════════════════════════╣"
echo "║  Worker: $WORKER_NAME"
printf "║  Max iterations: %-36s║\n" "$MAX_ITERATIONS"
echo "╚═══════════════════════════════════════════════════════╝"

CONSECUTIVE_ERRORS=0
MAX_RETRIES=3
RETRY_DELAY=10
ITERATION=1

while [ $ITERATION -le $MAX_ITERATIONS ]; do
  echo ""
  echo "═══════════════════════════════════════════════════════"
  echo "  Ralph Iteration $ITERATION of $MAX_ITERATIONS ($WORKER_NAME)"
  echo "═══════════════════════════════════════════════════════"
  
  # Run the agent using the configured worker
  OUTPUT=$(run_agent "$PROJECT_ROOT" "$PROMPT_FILE") || true
  
  # Check for connection errors - these mean the iteration didn't actually run
  if echo "$OUTPUT" | grep -qE "ConnectError|ETIMEDOUT|ECONNRESET|ENOTFOUND|connection refused|Connection refused"; then
    CONSECUTIVE_ERRORS=$((CONSECUTIVE_ERRORS + 1))
    echo ""
    echo "⚠️  Connection error detected ($CONSECUTIVE_ERRORS consecutive)"
    
    if [ $CONSECUTIVE_ERRORS -ge $MAX_RETRIES ]; then
      echo "❌ Too many consecutive connection errors. Stopping."
      echo "   Check your network connection and $WORKER_NAME status."
      exit 1
    fi
    
    # Exponential backoff: 10s, 20s, 40s...
    WAIT_TIME=$((RETRY_DELAY * CONSECUTIVE_ERRORS))
    echo "   Waiting ${WAIT_TIME}s before retry..."
    sleep $WAIT_TIME
    
    # Don't increment iteration - retry this one
    continue
  fi
  
  # Reset error counter on successful connection
  CONSECUTIVE_ERRORS=0
  
  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "✅ Ralph completed all tasks!"
    echo "Completed at iteration $ITERATION of $MAX_ITERATIONS"
    exit 0
  fi
  
  echo "Iteration $ITERATION complete. Continuing..."
  ITERATION=$((ITERATION + 1))
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
