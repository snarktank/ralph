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
- read_file: { \"type\": \"read_file\", \"path\": \"relative/path/file.ext\" }
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

SELF-MODIFICATION RULES:
- You CAN read and modify ralph.sh - this is expected for self-improvement tasks
- ALWAYS commit before modifying ralph.sh
- After modifying ralph.sh, read it back to verify your changes are correct
- If you break JSON parsing or bash syntax, the next iteration will fail - be careful"
  
  # Trigger direct LLM path
  TOOL="llm_direct_nvidia"
fi

# Validate tool choice (after potential override by LLM provider)
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" && "$TOOL" != "llm_direct_nvidia" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp', 'claude', or 'llm_direct_nvidia' (via --llm-provider nvidia)."
  exit 1
fi

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

# Helper function for NVIDIA API calls
llm_call_nvidia() {
  local prompt="$1"
  local history="$2"
  
  # Construct messages: system prompt as first message, then history, then current user prompt
  local messages=$(jq -n --arg sys "$LLM_SYSTEM_PROMPT" \
    --argjson hist "$history" \
    --arg prompt "$prompt" \
    '[{role: "system", content: $sys}] + $hist + [{role: "user", content: $prompt}]')
  
  # Call NVIDIA API with retry logic (cold start can return empty)
  local _api_body
  _api_body=$(jq -n \
    --arg model "$LLM_MODEL" \
    --argjson msgs "$messages" \
    --argjson temp "${LLM_TEMPERATURE}" \
    --argjson max_tokens 4096 \
    '{model: $model, messages: $msgs, temperature: $temp, max_tokens: $max_tokens, stream: false}')
  
  local _retry _api_result
  for _retry in 1 2 3; do
    _api_result=$(curl -s --max-time 120 -X POST "${LLM_BASE_URL}/chat/completions" \
      -H "Authorization: Bearer $LLM_API_KEY" \
      -H "Content-Type: application/json" \
      -d "$_api_body")
    
    # Check if we got a valid response with content
    if echo "$_api_result" | jq -e '.choices[0].message.content' > /dev/null 2>&1; then
      echo "$_api_result"
      return 0
    fi
    
    echo "DEBUG: API attempt $_retry failed or empty, retrying in 5s..." >&2
    sleep 5
  done
  
  # Return whatever we got after 3 tries
  echo "$_api_result"
}

# Execute actions returned by the LLM
execute_nvidia_actions() {
  local actions_json="$1"
  
  # Process each action
  echo "$actions_json" | jq -c '.actions[]' | while read -r action; do
    local type
    type=$(echo "$action" | jq -r '.type // empty')
    
    case "$type" in
      "read_file")
        local path
        path=$(echo "$action" | jq -r '.path // empty')
        echo "DEBUG: Attempting to read file: $path"
        if [[ -n "$path" && -f "$path" ]]; then
          cat "$path"
        else
          echo "Error: File not found: $path"
        fi
        ;;
      "edit_file")
        local path
        path=$(echo "$action" | jq -r '.path // empty')
        local changes
        changes=$(echo "$action" | jq -c '.changes // empty')
        
        if [[ -z "$path" || -z "$changes" ]]; then
          echo "Error: Invalid edit_file action"
          continue
        fi
        
        # Apply each change
        echo "$changes" | jq -c '.[]' | while read -r change; do
          local old_str new_str
          old_str=$(echo "$change" | jq -r '.oldString // empty')
          new_str=$(echo "$change" | jq -r '.newString // empty')
          
          if [[ -n "$old_str" && -n "$new_str" && -f "$path" ]]; then
            # Use sed with careful escaping for forward slash and ampersand
            _EP="$path" _EO="$old_str" _EN="$new_str" python3 -c "import os;p=os.environ['_EP'];c=open(p).read().replace(os.environ['_EO'],os.environ['_EN'],1);open(p,'w').write(c)"
            echo "Edited: $path"
          else
            echo "Warning: Skipping invalid change in $path"
          fi
        done
        ;;
      "create_file")
        local path
        path=$(echo "$action" | jq -r '.path // .filename // empty')
        local content
        content=$(echo "$action" | jq -r '.content // empty')
        
        if [[ -z "$path" ]]; then
          echo "Error: Invalid create_file action - missing path"
          continue
        fi
        
        if [[ -z "$content" ]]; then
          echo "Error: Invalid create_file action - missing content"
          continue
        fi
        
        # Create directory if needed
        mkdir -p "$(dirname "$path")"
        
        # Write content to file
        echo "$content" > "$path"
        echo "Created: $path"
        ;;
      "run_command")
        local command expect_success
        command=$(echo "$action" | jq -r '.command // empty')
        expect_success=$(echo "$action" | jq -r '.expects_success // true')
        
        if [[ -n "$command" ]]; then
          echo "Running: $command"
          local output
          output=$(eval "$command" 2>&1)
          local exit_code=$?
          
          if [[ "$expect_success" == "true" && $exit_code -ne 0 ]]; then
            echo "Command failed (exit $exit_code): $output"
          else
            echo "Command succeeded: $output"
          fi
        fi
        ;;
      "finish")
        _sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$SCRIPT_DIR/prd.json" 2>/dev/null)
        if [[ -n "$_sid" && "$_sid" != "null" ]]; then
          _tmp=$(mktemp)
          jq --arg id "$_sid" '(.userStories[] | select(.id == $id)).passes = true' "$SCRIPT_DIR/prd.json" > "$_tmp" && mv "$_tmp" "$SCRIPT_DIR/prd.json"
          echo "Story $_sid marked as PASSED"
        fi
        echo "<promise>COMPLETE</promise>"
        return 0
        ;;
      *)
        echo "Warning: Unknown action type: $type"
        ;;
    esac
  done
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
  
  echo "DEBUG: Starting agent loop"
  
  # Main agent loop
  while true; do
    echo "DEBUG: Entering agent loop iteration $iteration_count"
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
    echo "DEBUG: LLM response content length: ${#llm_content}"
    if [[ -n "$llm_content" && "$llm_content" != "null" ]]; then
      echo "DEBUG: LLM content (first 200 chars): ${llm_content:0:200}"
      echo "DEBUG: LLM content (last 50 chars): ${llm_content: -50}"
    else
      echo "DEBUG: LLM content is empty or null"
      echo "DEBUG: Full response: $llm_response"
    fi
    
    # If we got empty content, show error and break
    if [[ -z "$llm_content" || "$llm_content" == "null" ]]; then
      echo "Error: Empty response from NVIDIA API"
      echo "Response: $llm_response"
      return 1
    fi
    
    # Debug: show what we got
    echo "DEBUG: LLM response content length: ${#llm_content}"
    if [[ -n "$llm_content" && "$llm_content" != "null" ]]; then
      echo "DEBUG: LLM content (first 200 chars): ${llm_content:0:200}"
      echo "DEBUG: LLM content (last 50 chars): ${llm_content: -50}"
    else
      echo "DEBUG: LLM content is empty or null"
      echo "DEBUG: Full response: $llm_response"
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
      echo "Warning: LLM did not return valid JSON. Response:"
      echo "$llm_content"
      echo "Attempting to continue..."
      # Simple fallback: finish if contains completion signal
      if echo "$llm_content" | grep -q "<promise>COMPLETE</promise>"; then
        echo "<promise>COMPLETE</promise>"
        return 0
      fi
      # Otherwise, ask for clarification in next turn
      actions_json='{"thoughts": "Invalid JSON response from LLM. Please respond with valid JSON matching the required format.", "actions": []}'
    fi
    
    # Add LLM response to conversation history
    conversation_history=$(echo "$conversation_history" | jq --arg content "$llm_content" '. + [{"role": "assistant", "content": $content}]')
    
    # Execute actions
    local execution_result
    execution_result=$(execute_nvidia_actions "$actions_json")

    local exit_code=$?
    
    # Add execution result to history
    conversation_history=$(echo "$conversation_history" | jq --arg res "$execution_result" '. + [{"role": "user", "content": ("Action execution result: " + $res)}]')
    
    # Check for completion signal via exit code from execute_nvidia_actions
    # (grep on execution_result is unsafe - reading ralph.sh would match the string in source code)
    if [[ $exit_code -eq 0 ]] && echo "$execution_result" | tail -1 | grep -qF "<promise>COMPLETE</promise>"; then
      return 0
    fi
    
    # Safety check: prevent infinite loops
    if (( iteration_count++ > 40 )); then
      echo "Error: Exceeded maximum agent iterations"
      return 1
    fi
  done
}

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="
  
  # Run the selected tool with the ralph prompt
  echo "DEBUG: TOOL is '$TOOL'" >&2
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  elif [[ "$TOOL" == "claude" ]]; then
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  elif [[ "$TOOL" == "llm_direct_nvidia" ]]; then
    OUTPUT=$(run_nvidia_agent_loop) || true
    echo "DEBUG: OUTPUT from agent loop: $OUTPUT" >&2
  fi
  
  # Check for completion signal
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