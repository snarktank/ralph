#!/bin/bash
# task-utils.sh
# Bash utilities for Ralph task system integration

##
# Get or create a deterministic task list ID based on project and branch
##
get_or_create_task_list_id() {
  local prd_file="${1:-prd.json}"
  local cache_file=".ralph-task-list-id"

  # If cache exists and is recent (< 1 hour old), use it
  if [ -f "$cache_file" ] && [ -n "$(find "$cache_file" -mmin -60 2>/dev/null)" ]; then
    cat "$cache_file"
    return 0
  fi

  # Extract project name and branch from prd.json
  local project_name=""
  local branch_name=""

  if [ -f "$prd_file" ]; then
    project_name=$(jq -r '.project // "unknown"' "$prd_file" 2>/dev/null || echo "unknown")
    branch_name=$(jq -r '.branchName // "main"' "$prd_file" 2>/dev/null || echo "main")
  else
    # Fallback: use current git branch
    project_name=$(basename "$(pwd)")
    branch_name=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")
  fi

  # Generate deterministic hash
  local task_list_id
  if command -v md5 > /dev/null 2>&1; then
    # macOS
    task_list_id=$(echo -n "${project_name}-${branch_name}" | md5 | cut -c1-16)
  elif command -v md5sum > /dev/null 2>&1; then
    # Linux
    task_list_id=$(echo -n "${project_name}-${branch_name}" | md5sum | cut -d' ' -f1 | cut -c1-16)
  else
    # Fallback: simple hash
    task_list_id=$(echo -n "${project_name}-${branch_name}" | cksum | cut -d' ' -f1)
  fi

  # Cache the result
  echo "$task_list_id" > "$cache_file"
  echo "$task_list_id"
}

##
# Check if Claude Code task system is available
##
check_task_system_available() {
  # Check if claude command exists
  if ! command -v claude > /dev/null 2>&1; then
    return 1
  fi

  # Check if task commands work
  if claude task list --json > /dev/null 2>&1; then
    return 0
  else
    return 1
  fi
}

##
# Export task environment variables
##
export_task_env() {
  local task_list_id="$1"

  if [ -n "$task_list_id" ]; then
    export CLAUDE_CODE_TASK_LIST_ID="$task_list_id"
    echo "✓ Task List ID: $task_list_id"
    return 0
  else
    echo "✗ No task list ID provided"
    return 1
  fi
}

##
# Initialize task system for Ralph
##
init_task_system() {
  local prd_file="${1:-prd.json}"

  echo "Initializing task system..."

  # Check availability
  if ! check_task_system_available; then
    echo "⚠  Task system unavailable (Claude Code not installed or tasks disabled)"
    echo "   Falling back to prd.json-only mode"
    return 1
  fi

  # Get task list ID
  local task_list_id
  task_list_id=$(get_or_create_task_list_id "$prd_file")

  if [ -z "$task_list_id" ]; then
    echo "✗ Failed to generate task list ID"
    return 1
  fi

  # Export environment variable
  export_task_env "$task_list_id"

  return 0
}

##
# Check if tasks exist for current PRD
##
tasks_exist_for_prd() {
  local prd_file="${1:-prd.json}"

  if [ ! -f "$prd_file" ]; then
    return 1
  fi

  # Get project name from PRD
  local project_name
  project_name=$(jq -r '.project // "unknown"' "$prd_file" 2>/dev/null || echo "unknown")

  # Check if any tasks exist with matching metadata
  local task_count
  task_count=$(claude task list --json 2>/dev/null | jq '[.[] | select(.metadata.type == "parent")] | length' 2>/dev/null || echo "0")

  if [ "$task_count" -gt 0 ]; then
    return 0
  else
    return 1
  fi
}
