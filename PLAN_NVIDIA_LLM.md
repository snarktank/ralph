# Plan: NVIDIA LLM API Integration for Ralph

## Objective
Enable Ralph to use NVIDIA LLM APIs (NVIDIA NIM, AI Foundation Models) via direct API calls instead of requiring Amp or Claude Code agents, allowing users to utilize their NVIDIA API keys.

## Key Decisions
- **Provider**: NVIDIA only (as requested)
- **.env Support**: Yes, for development convenience with strict gitignore protection
- **API Compatibility**: Leverage NVIDIA's OpenAI-compatible chat completions API
- **Backward Compatibility**: Preserve existing Amp/Claude Code functionality unchanged

## Implementation Overview
Modify `ralph.sh` to:
1. Accept NVIDIA-specific command-line arguments
2. Optionally load variables from `.env` file (development only)
3. Invoke NVIDIA API directly when `--llm-provider nvidia` is specified
4. Use a custom agent loop that interprets LLM responses as structured actions
5. Execute actions (file edits, command runs) and continue until completion

## Detailed Changes to ralph.sh

### 1. Extended Argument Parsing
Add near line 11 (after existing argument parsing):
```bash
# LLM Provider Arguments
while [[ $# -gt 0 ]]; do
  case $1 in
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
      # Handle existing args (--tool, max_iterations) as before
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      fi
      shift
      ;;
  esac
done
```

### 2. Validation and Defaults
Add after argument parsing (around line 29):
```bash
# Validate and set defaults for NVIDIA provider
if [[ -n "$LLM_PROVIDER" ]]; then
  if [[ "$LLM_PROVIDER" != "nvidia" ]]; then
    echo "Error: Only '--llm-provider nvidia' is currently supported."
    echo "To use Amp or Claude Code, omit --llm-provider or use --tool amp/claude."
    exit 1
  fi
  
  # Require API key
  if [[ -z "$LLM_API_KEY" ]]; then
    echo "Error: --llm-api-key is required when using --llm-provider nvidia"
    echo "You can also set NVIDIA_API_KEY environment variable or use .env file"
    exit 1
  fi
  
  # Set NVIDIA-specific defaults
  LLM_BASE_URL="${LLM_BASE_URL:-https://integrate.api.nvidia.com/v1}"
  LLM_MODEL="${LLM_MODEL:-nvidia/nemotron-4-340b-instruct}"
  LLM_TEMPERATURE="${LLM_TEMPERATURE:-0.7}"
  
  # Trigger direct LLM path
  TOOL="llm_direct_nvidia"
fi
```

### 3. Optional .env Loading
Add at the very top of the script (after `set -e`):
```bash
# Load .env file for development convenience (NOT for production)
# SECURITY: .env files MUST NOT be committed to git
if [ -f ".env" ]; then
  # shellcheck disable=SC1091
  . .env
fi
```

### 4. NVIDIA API Helper Function
Add before the main loop (around line 80):
```bash
llm_call_nvidia() {
  local prompt="$1"
  local history="$2"
  
  # Construct messages: system prompt + history + current user prompt
  local messages=$(jq -n --arg sys "$LLM_SYSTEM_PROMPT" \
    --argjson hist "$history" \
    --arg prompt "$prompt" \
    '[$sys] + $hist + [{"role": "user", "content": $prompt}]')
  
  # Call NVIDIA API (OpenAI-compatible format)
  curl -s -X POST "${LLM_BASE_URL}/chat/completions" \
    -H "Authorization: Bearer $LLM_API_KEY" \
    -H "Content-Type: application/json" \
    -d "$(jq -n \
      --arg model "$LLM_MODEL" \
      --argjson msgs "$messages" \
      --argjson temp "${LLM_TEMPERATURE}" \
      '{model: $model, messages: $msgs, temperature: $temp, stream: false}')"
}
```

### 5. Custom Agent Loop
Replace the tool invocation block (lines 91-96 in original) with:
```bash
if [[ "$TOOL" == "llm_direct_nvidia" ]]; then
  OUTPUT=$(run_nvidia_agent_loop) || true
else
  # Existing Amp/Claude Code paths unchanged
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/prompt.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  else
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  fi
fi
```

### 6. Agent Loop Implementation
Add new function before main loop:
```bash
run_nvidia_agent_loop() {
  local conversation_history='[]'  # JSON array of message objects
  
  # Main agent loop
  while true; do
    # Get current story context
    local context=$(get_story_context)
    
    # Build prompt for LLM
    local full_prompt=$(construct_nvidia_prompt "$context")
    
    # Call NVIDIA API
    local llm_response=$(llm_call_nvidia "$full_prompt" "$conversation_history")
    
    # Extract content from response (handle API format)
    local llm_content=$(echo "$llm_response" | jq -r '.choices[0].message.content // empty')
    
    # If we got empty content, show error and break
    if [[ -z "$llm_content" || "$llm_content" == "null" ]]; then
      echo "Error: Empty response from NVIDIA API"
      echo "Response: $llm_response"
      return 1
    fi
    
    # Try to parse as JSON action
    local actions_json
    actions_json=$(echo "$llm_content" | jq -r '. // empty' 2>/dev/null)
    
    # If not valid JSON, try to extract JSON from text
    if [[ -z "$actions_json" || "$actions_json" == "null" ]]; then
      # Look for JSON block in response
      actions_json=$(echo "$llm_content" | sed -n 's/.*```json\n\(.*\)\n```.*/\1/p')
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
    conversation_history=$(echo "$conversation_history" | jq --argjson new "{\"role\": \"assistant\", \"content\": \"$llm_content\"}" '. + [$new]')
    
    # Execute actions
    local execution_result=$(execute_nvidia_actions "$actions_json")
    local exit_code=$?
    
    # Add execution result to history
    conversation_history=$(echo "$conversation_history" | jq --arg res "$execution_result" '. + [{"role": "user", "content": "Action execution result: " + $res}]')
    
    # Check for completion signal
    if echo "$execution_result" | grep -q "<promise>COMPLETE</promise>"; then
      return 0
    fi
    
    # Safety check: prevent infinite loops
    if (( iteration_count++ > 20 )); then
      echo "Error: Exceeded maximum agent iterations"
      return 1
    fi
  done
}
```

### 7. Supporting Functions
Add these helper functions before the agent loop:

#### Context Extraction
```bash
get_story_context() {
  local prd_file="$SCRIPT_DIR/prd.json"
  local progress_file="$SCRIPT_DIR/progress.txt"
  
  # Get highest priority story where passes: false
  local story=$(jq -r '.userStories[] | select(.passes == false) | .' "$prd_file" 2>/dev/null | head -n 1)
  
  if [[ -z "$story" ]]; then
    echo "{}"
    return
  fi
  
  # Format as JSON for prompt
  echo "$story" | jq -c '{id, title, description, acceptanceCriteria, branchName}'
}
```

#### Prompt Construction
```bash
construct_nvidia_prompt() {
  local context_json="$1"
  
  # Read instructions (simplified version)
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
}
```

#### Action Execution
```bash
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
            # Use sed with careful escaping
            sed -i "s|$(printf '%s' "$old_str" | sed 's/[&/\]/\\&/g')|$(printf '%s' "$new_str" | sed 's/[&/\]/\\&/g')|g" "$path"
            echo "Edited: $path"
          else
            echo "Warning: Skipping invalid change in $path"
          fi
        done
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
        echo "<promise>COMPLETE</promise>"
        return 0
        ;;
      *)
        echo "Warning: Unknown action type: $type"
        ;;
    esac
  done
}
```

## .env File Handling

### Recommended .env Format
```
# .env (NEVER commit this file)
NVIDIA_API_KEY="your-nvidia-api-key-here"
# Optional overrides:
# LLM_MODEL="nvidia/nemotron-4-340b-instruct"
# LLM_TEMPERATURE="0.2"
```

### .gitupdate
Add to `.gitignore`:
```
# Environment files (containing secrets - NEVER commit)
.env
.env.*
! .env.example  # Allow template file
```

### Usage Examples
```bash
# Method 1: Command line (most secure)
./ralph.sh --llm-provider nvidia --llm-api-key "$NVIDIA_API_KEY"

# Method 2: Environment variable
export NVIDIA_API_KEY="key-here"
./ralph.sh --llm-provider nvidia

# Method 3: .env file (development only)
echo "NVIDIA_API_KEY=your-key" > .env
./ralph.sh --llm-provider nvidia
```

## Security Considerations
1. **Never commit .env files** - already addressed in .gitignore plan
2. **API key treatment**: Treat NVIDIA API keys like passwords
3. **Production use**: Recommend environment variables or secret managers over .env
4. **Least privilege**: Consider restricting API keys to specific models if supported
5. **Monitoring**: Advise users to monitor API usage for unexpected charges

## Testing Strategy
1. **Unit tests**: Mock NVIDIA API responses to verify action parsing/execution
2. **Integration test**: Use a real NVIDIA API key with a trivial PRD (e.g., "create a hello world file")
3. **Fallback verification**: Ensure Amp/Claude Code paths still work when --llm-provider omitted
4. **.env testing**: Verify .env loading works and doesn't interfere with command-line args

## Assumptions & Requirements
1. **NVIDIA API compatibility**: Assumes OpenAI-compatible chat completions endpoint
2. **Model availability**: User must have access to specified NVIDIA model
3. **curl dependency**: Requires curl for HTTP calls (standard on most systems)
4. **jq dependency**: Already required by Ralph for prd.json handling
5. **bash compatibility**: Uses standard bash features available on Linux/macOS/WSL

## Estimated Effort
- Core changes: ~150 lines added to ralph.sh
- Backward compatibility: Zero impact on existing Amp/Claude Code usage
- Documentation: Update README.md with NVIDIA usage examples

This plan provides a secure, functional path to NVIDIA LLM integration while maintaining Ralph's existing functionality and autonomy principles.