#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop with multi-PRD support
# Usage: ./ralph.sh [--tool amp|claude] [--multi] [max_iterations]
#
# Multi-PRD mode: Place PRD files in prds/ directory with a "priority" field.
# Ralph processes PRDs from highest priority (lowest number) to lowest.

set -e

# Parse arguments
TOOL="amp"  # Default to amp for backwards compatibility
MAX_ITERATIONS=10
MULTI_PRD_MODE=false

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
    --multi)
      MULTI_PRD_MODE=true
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
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp' or 'claude'."
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"
PRDS_DIR="$SCRIPT_DIR/prds"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"
COMPLETED_PRDS_FILE="$SCRIPT_DIR/.completed-prds"

# =============================================================================
# Multi-PRD Helper Functions
# =============================================================================

# Check if a PRD is complete (all user stories have passes: true)
is_prd_complete() {
  local prd_file="$1"
  if [ ! -f "$prd_file" ]; then
    return 1
  fi

  local incomplete_count
  incomplete_count=$(jq '[.userStories[] | select(.passes == false)] | length' "$prd_file" 2>/dev/null || echo "-1")

  if [ "$incomplete_count" = "0" ]; then
    return 0  # Complete
  else
    return 1  # Not complete
  fi
}

# Get the next PRD to work on (highest priority = lowest number, not yet complete)
get_next_prd() {
  if [ ! -d "$PRDS_DIR" ]; then
    echo ""
    return
  fi

  # Initialize completed PRDs tracking file
  touch "$COMPLETED_PRDS_FILE"

  # Find all PRD files, sort by priority (lowest first), return first incomplete one
  local next_prd=""
  local lowest_priority=999999

  for prd in "$PRDS_DIR"/*.json; do
    [ -f "$prd" ] || continue

    local prd_name
    prd_name=$(basename "$prd")

    # Skip if already marked as completed
    if grep -q "^${prd_name}$" "$COMPLETED_PRDS_FILE" 2>/dev/null; then
      continue
    fi

    # Check if PRD is complete
    if is_prd_complete "$prd"; then
      echo "$prd_name" >> "$COMPLETED_PRDS_FILE"
      continue
    fi

    # Get priority (default to 999 if not specified)
    local priority
    priority=$(jq -r '.priority // 999' "$prd" 2>/dev/null || echo "999")

    if [ "$priority" -lt "$lowest_priority" ]; then
      lowest_priority=$priority
      next_prd="$prd"
    fi
  done

  echo "$next_prd"
}

# Count remaining incomplete PRDs
count_remaining_prds() {
  if [ ! -d "$PRDS_DIR" ]; then
    echo "0"
    return
  fi

  local count=0
  for prd in "$PRDS_DIR"/*.json; do
    [ -f "$prd" ] || continue

    local prd_name
    prd_name=$(basename "$prd")

    # Skip if already marked as completed
    if grep -q "^${prd_name}$" "$COMPLETED_PRDS_FILE" 2>/dev/null; then
      continue
    fi

    if ! is_prd_complete "$prd"; then
      ((count++))
    fi
  done

  echo "$count"
}

# Activate a PRD (copy to prd.json for the agent to work on)
activate_prd() {
  local prd_file="$1"
  if [ -z "$prd_file" ] || [ ! -f "$prd_file" ]; then
    return 1
  fi

  local prd_name
  prd_name=$(basename "$prd_file")
  local project_name
  project_name=$(jq -r '.project // "Unknown"' "$prd_file")
  local priority
  priority=$(jq -r '.priority // "N/A"' "$prd_file")

  echo ""
  echo "==============================================================="
  echo "  Activating PRD: $prd_name"
  echo "  Project: $project_name | Priority: $priority"
  echo "==============================================================="

  cp "$prd_file" "$PRD_FILE"
  return 0
}

# Sync changes back to source PRD file
sync_prd_changes() {
  local source_prd="$1"
  if [ -n "$source_prd" ] && [ -f "$source_prd" ] && [ -f "$PRD_FILE" ]; then
    cp "$PRD_FILE" "$source_prd"
  fi
}

# =============================================================================
# Archive Functions
# =============================================================================

archive_previous_run() {
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
}

track_current_branch() {
  if [ -f "$PRD_FILE" ]; then
    CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
    if [ -n "$CURRENT_BRANCH" ]; then
      echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"
    fi
  fi
}

# =============================================================================
# Main Execution
# =============================================================================

# Auto-detect multi-PRD mode if prds/ directory exists
if [ -d "$PRDS_DIR" ] && [ "$(ls -A "$PRDS_DIR"/*.json 2>/dev/null)" ]; then
  MULTI_PRD_MODE=true
  echo "Detected prds/ directory - enabling multi-PRD mode"
fi

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# Ralph Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

# Track current PRD source for multi-PRD mode
CURRENT_PRD_SOURCE=""

if [ "$MULTI_PRD_MODE" = true ]; then
  echo ""
  echo "╔═══════════════════════════════════════════════════════════════╗"
  echo "║           RALPH - Multi-PRD Mode                              ║"
  echo "╠═══════════════════════════════════════════════════════════════╣"
  echo "║  Tool: $TOOL"
  echo "║  Max iterations per PRD: $MAX_ITERATIONS"
  echo "║  PRDs directory: $PRDS_DIR"
  echo "╚═══════════════════════════════════════════════════════════════╝"

  # List all PRDs with their priorities
  echo ""
  echo "PRD Queue (by priority):"
  echo "─────────────────────────────────────────────────────────────────"
  for prd in "$PRDS_DIR"/*.json; do
    [ -f "$prd" ] || continue
    prd_name=$(basename "$prd")
    priority=$(jq -r '.priority // 999' "$prd" 2>/dev/null || echo "?")
    project=$(jq -r '.project // "?"' "$prd" 2>/dev/null || echo "?")
    story_count=$(jq '.userStories | length' "$prd" 2>/dev/null || echo "?")
    complete_count=$(jq '[.userStories[] | select(.passes == true)] | length' "$prd" 2>/dev/null || echo "0")

    if grep -q "^${prd_name}$" "$COMPLETED_PRDS_FILE" 2>/dev/null; then
      status="[DONE]"
    elif is_prd_complete "$prd"; then
      status="[DONE]"
    else
      status="[${complete_count}/${story_count}]"
    fi

    printf "  Priority %s: %-30s %s %s\n" "$priority" "$project" "$status" "$prd_name"
  done | sort -t: -k1 -n
  echo "─────────────────────────────────────────────────────────────────"

  # Get next PRD
  CURRENT_PRD_SOURCE=$(get_next_prd)

  if [ -z "$CURRENT_PRD_SOURCE" ]; then
    echo ""
    echo "All PRDs are complete!"
    exit 0
  fi

  activate_prd "$CURRENT_PRD_SOURCE"
else
  echo "Starting Ralph - Tool: $TOOL - Max iterations: $MAX_ITERATIONS"
  archive_previous_run
fi

track_current_branch

# Main iteration loop
TOTAL_ITERATIONS=0
PRD_ITERATIONS=0

while true; do
  ((TOTAL_ITERATIONS++))
  ((PRD_ITERATIONS++))

  # Check iteration limits
  if [ $PRD_ITERATIONS -gt $MAX_ITERATIONS ]; then
    if [ "$MULTI_PRD_MODE" = true ]; then
      echo ""
      echo "Max iterations ($MAX_ITERATIONS) reached for current PRD."

      # Sync changes back to source PRD
      sync_prd_changes "$CURRENT_PRD_SOURCE"

      # Try to get next PRD
      CURRENT_PRD_SOURCE=$(get_next_prd)

      if [ -z "$CURRENT_PRD_SOURCE" ]; then
        echo "All PRDs are complete!"
        exit 0
      fi

      # Reset iteration counter for new PRD
      PRD_ITERATIONS=1
      activate_prd "$CURRENT_PRD_SOURCE"
      track_current_branch
    else
      echo ""
      echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
      echo "Check $PROGRESS_FILE for status."
      exit 1
    fi
  fi

  echo ""
  echo "==============================================================="
  if [ "$MULTI_PRD_MODE" = true ]; then
    remaining=$(count_remaining_prds)
    current_prd_name=$(basename "$CURRENT_PRD_SOURCE" 2>/dev/null || echo "prd.json")
    echo "  Ralph Iteration $PRD_ITERATIONS/$MAX_ITERATIONS ($TOOL)"
    echo "  PRD: $current_prd_name | Remaining PRDs: $remaining"
  else
    echo "  Ralph Iteration $PRD_ITERATIONS of $MAX_ITERATIONS ($TOOL)"
  fi
  echo "==============================================================="

  # Run the selected tool with the ralph prompt
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  else
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  fi

  # Sync changes back to source PRD in multi-PRD mode
  if [ "$MULTI_PRD_MODE" = true ] && [ -n "$CURRENT_PRD_SOURCE" ]; then
    sync_prd_changes "$CURRENT_PRD_SOURCE"
  fi

  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    if [ "$MULTI_PRD_MODE" = true ]; then
      current_prd_name=$(basename "$CURRENT_PRD_SOURCE" 2>/dev/null || echo "prd.json")
      echo ""
      echo "PRD completed: $current_prd_name"

      # Mark this PRD as completed
      echo "$current_prd_name" >> "$COMPLETED_PRDS_FILE"

      # Archive the completed PRD
      DATE=$(date +%Y-%m-%d)
      BRANCH_NAME=$(jq -r '.branchName // "unknown"' "$PRD_FILE" 2>/dev/null || echo "unknown")
      FOLDER_NAME=$(echo "$BRANCH_NAME" | sed 's|^ralph/||')
      ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"

      mkdir -p "$ARCHIVE_FOLDER"
      cp "$PRD_FILE" "$ARCHIVE_FOLDER/"
      [ -f "$PROGRESS_FILE" ] && cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/"
      echo "   Archived to: $ARCHIVE_FOLDER"

      # Reset progress file for next PRD
      echo "# Ralph Progress Log" > "$PROGRESS_FILE"
      echo "Started: $(date)" >> "$PROGRESS_FILE"
      echo "---" >> "$PROGRESS_FILE"

      # Try to get next PRD
      CURRENT_PRD_SOURCE=$(get_next_prd)

      if [ -z "$CURRENT_PRD_SOURCE" ]; then
        echo ""
        echo "╔═══════════════════════════════════════════════════════════════╗"
        echo "║           ALL PRDs COMPLETE!                                  ║"
        echo "╠═══════════════════════════════════════════════════════════════╣"
        echo "║  Total iterations: $TOTAL_ITERATIONS"
        echo "╚═══════════════════════════════════════════════════════════════╝"
        exit 0
      fi

      # Reset iteration counter for new PRD
      PRD_ITERATIONS=0
      activate_prd "$CURRENT_PRD_SOURCE"
      track_current_branch
    else
      echo ""
      echo "Ralph completed all tasks!"
      echo "Completed at iteration $PRD_ITERATIONS of $MAX_ITERATIONS"
      exit 0
    fi
  else
    echo "Iteration $PRD_ITERATIONS complete. Continuing..."
  fi

  sleep 2
done
