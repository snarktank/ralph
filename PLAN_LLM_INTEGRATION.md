# Plan: Integrating Direct LLM API Support into Ralph

## Objective
Modify Ralph to support direct API calls to LLM providers (e.g., OpenAI, Anthropic, Ollama) instead of relying solely on Amp or Claude Code agents. This enables users to use their own API keys with alternative LLM providers while preserving Ralph's autonomous agent functionality.

## Core Challenge
Ralph's current design assumes the invoked tool (Amp/Claude Code) is a fully autonomous coding agent capable of:
- Reading instruction files
- Editing files autonomously
- Running shell commands
- Operating as a continuous agent

Standard LLM APIs only return text completions - they lack built-in tool use, file editing, or persistent agent behavior. We must implement an agent loop ourselves that uses the LLM as a reasoning engine.

## Solution Overview
Replace Ralph's tool invocation phase with a custom agent loop that:
1. Uses LLM API to generate responses
2. Interprets responses as structured actions (file edits, command runs, etc.)
3. Executes actions in the environment
4. Maintains conversation history and context
5. Continues until completion signal is generated

## Detailed Implementation Plan

### Phase 1: Foundation & Minimal Viable Product
**Goal:** Support one provider (OpenAI) with basic action protocol

#### 1.1 Extend Argument Parsing (ralph.sh)
Add new options:
```bash
# LLMSPECIFIC ARGS
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
    # ... existing args ...
  esac
done
```

Add validation:
```bash
if [[ -n "$LLM_PROVIDER" ]]; then
  if [[ -z "$LLM_API_KEY" ]]; then
    echo "Error: --llm-api-key required when using --llm-provider"
    exit 1
  fi
  TOOL="llm_direct"  # Special value to trigger direct API path
fi
```

#### 1.2 Create LLM Provider Abstraction
Add helper functions in ralph.sh:
```bash
llm_call_openai() {
  local prompt="$1"
  local history="$2"  # JSON array of previous messages
  
  # Build messages array: system + history + current prompt
  local messages=$(jq -n --arg sys "$LLM_SYSTEM_PROMPT" \
    --argjson hist "$history" \
    --arg prompt "$prompt" \
    '[$sys] + $hist + [{"role": "user", "content": $prompt}]')
  
  # Call OpenAI API
  curl -s -X POST "${LLM_BASE_URL:-https://api.openai.com/v1}/chat/completions" \
    -H "Authorization: Bearer $LLM_API_KEY" \
    -H "Content-Type: application/json" \
    -d "$(jq -n \
      --arg model "$LLM_MODEL" \
      --argjson msgs "$messages" \
      '{model: $model, messages: $msgs, temperature: 0.7}')"
}

# Similar functions for other providers (Anthropic, Ollama, etc.)
```

#### 1.3 Implement Custom Agent Loop
Replace the simple tool invocation (lines 91-96) with:
```bash
if [[ "$TOOL" == "llm_direct" ]]; then
  OUTPUT=$(run_llm_agent_loop) || true
else
  # Existing amp/claude code paths unchanged
fi
```

Where `run_llm_agent_loop` is a new function that:
- Initializes conversation history
- Enters loop: get LLM response → parse actions → execute actions → check completion
- Returns final output for Ralph to parse

#### 1.4 Define Action Protocol
LLM must respond with structured actions. Simplest approach: expect JSON:
```json
{
  "thoughts": "Reasoning about what to do next",
  "actions": [
    {
      "type": "read_file",
      "path": "src/config.js"
    },
    {
      "type": "edit_file",
      "path": "src/config.js",
      "changes": [
        { "oldString": "const API_URL = '';", "newString": "const API_URL = 'https://api.example.com';" }
      ]
    },
    {
      "type": "run_command",
      "command": "npm test",
      "expects_success": true
    }
  ]
}
```

Special actions:
- `{"type": "finish"}` - signals completion
- `{"type": "finish", "message": "Task complete because..."}` - with optional message

#### 1.5 Action Execution Engine
Add function to interpret and execute actions:
```bash
execute_actions() {
  local actions_json="$1"
  
  # Parse JSON and iterate over actions
  echo "$actions_json" | jq -c '.actions[]' | while read -r action; do
    local type=$(echo "$action" | jq -r '.type')
    
    case "$type" in
      "read_file")
        local path=$(echo "$action" | jq -r '.path')
        cat "$path"  # Output for LLM context in next turn
        ;;
      "edit_file")
        local path=$(echo "$action" | jq -r '.path')
        local changes=$(echo "$action" | jq -c '.changes')
        # Apply each change using sed or similar (careful with quoting!)
        ;;
      "run_command")
        local cmd=$(echo "$action" | jq -r '.command')
        local expect_success=$(echo "$action" | jq -r '.expects_success // true')
        # Run command, check exit code if expect_success
        ;;
      "finish")
        # Signal completion - return special marker
        echo "<promise>COMPLETE</promise>"
        return 0
        ;;
    esac
  done
}
```

#### 1.6 Prompt Construction
Instead of feeding full prompt.md/CLAUDE.md, construct LLM prompt that includes:
- Task context (from prd.json, progress.txt)
- Instructions for action format
- Conversation history
- Current objective (highest priority story)

Example system prompt:
```
You are an autonomous coding agent working on a software project.
Your goal is to implement the highest priority user story where passes: false.

You must respond with JSON containing:
- "thoughts": Your reasoning
- "actions": Array of action objects to execute

Action types:
- read_file: { "path": "filepath" }
- edit_file: { "path": "filepath", "changes": [ { "oldString": "...", "newString": "..." } ] }
- run_command: { "command": "shell command", "expects_success": true/false }
- finish: { }  // Signals task completion

Context:
- PRD: [content from prd.json]
- Progress: [recent progress.txt]
- Current story: [highest priority story where passes=false]
```

### Phase 2: Robustness & Multiple Providers
**Goal:** Support multiple providers, improve reliability

#### 2.1 Provider Registry
Refactor provider functions into a registry:
```bash
declare -A LLM_PROVIDERS
LLM_PROVIDERS["openai"]="llm_call_openai"
LLM_PROVIDERS["anthropic"]="llm_call_anthropic"
LLM_PROVIDERS["ollama"]="llm_call_ollama"
# etc.

llm_call() {
  local provider="$1"
  local prompt="$2"
  local history="$3"
  
  local func="${LLM_PROVIDERS[$provider]}"
  if [[ -z "$func" ]]; then
    echo "Error: Unsupported LLM provider: $provider"
    return 1
  fi
  "$func" "$prompt" "$history"
}
```

#### 2.2 Enhanced Action Protocol
Support more action types:
- `write_file`: Create new file with content
- `delete_file`: Remove file
- `request_human_input`: For cases requiring clarification (with timeout)
- `parallel_actions`: Execute multiple actions concurrently

#### 2.3 Context Management
Implement strategies to prevent context overflow:
- **Sliding window**: Keep only last N exchanges
- **Summarization**: Periodically ask LLM to summarize history
- **File-based context**: For large files, reference rather than include full content

#### 2.4 Error Handling & Retries
- Distinguish between LLM API errors vs. action execution errors
- Implement exponential backoff for API rate limits
- Allow LLM to retry failed actions with modified parameters
- Circuit breaker for repeatedly failing providers

### Phase 3: Polish & Security
**Goal:** Production-ready implementation

#### 3.1 Secure Credential Handling
- Document use of environment variables: `LLM_API_KEY_OPENAI`, etc.
- Warn against hardcoding keys in scripts
- Optional: Support reading from .env file (with proper permissions warning)

#### 3.2 Performance Optimizations
- Connection reuse for API calls (where supported)
- Batch similar actions when safe
- Pre-fetch commonly accessed files

#### 3.3 Comprehensive Testing
- Unit tests for action parsing/execution
- Integration tests with mock LLM responses
- End-to-end test with real provider (user-provided API key in test env)

#### 3.4 Documentation Updates
- Update README.md with new usage examples:
  ```
  # Using OpenAI directly
  ./ralph.sh --llm-provider openai --llm-api-key $OPENAI_API_KEY --llm-model gpt-4
  
  # Using Ollama (local)
  ./ralph.sh --llm-provider ollama --llm-base-url http://localhost:11434
  ```
- Update skills documentation if needed
- Add troubleshooting guide for common issues

## Files to Modify
- **Primary:** `ralph.sh` - Contains all core logic changes
- **Potentially:** New helper libraries if complexity grows significantly
- **Documentation:** `README.md`, possibly `AGENTS.md` templates

## Dependencies
- **bash**: Standard shell (already required)
- **jq**: JSON parsing (already required by Ralph for prd.json handling)
- **curl**: HTTP client for API calls (needs to be verified/installed)
- **sed/awk**: For file editing actions (standard on most systems)

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| **Increased complexity** | Keep direct LLM path separate; preserve existing amp/claude paths unchanged |
| **LLM unpredictability** | Strict action schema validation; fallback to safe defaults; quality gates |
| **Context window limits** | Implement context management strategies (sliding window, summarization) |
| **API key security** | Strong documentation about using env vars; never log keys |
| **Provider instability** | Abstract provider layer; easy to add/remove providers |
| **Action execution errors** | Validate actions before execution; sandbox where possible; dry-run option |

## Open Questions for User Input
1. **Provider Prioritization**: Which LLM providers should be supported first? (Suggested: OpenAI → Anthropic → Ollama)
2. **Action Format**: Should we use JSON (as proposed) or a simpler text-based format?
3. **API Key Handling**: Environment variables only, or also support config file?
4. **Conversation History**: How should we manage LLM context window? (Sliding window size? Summarization frequency?)
5. **Error Tolerance**: How many consecutive failed actions should trigger fallback/abort?
6. **Testing Approach**: Should we include a mock mode for testing without real API keys?

## Implementation Recommendation
Start with **Phase 1 (MVP)** for OpenAI only:
1. Get the basic loop working with hardcoded test values
2. Implement minimal action set (read_file, edit_file, run_command, finish)
3. Verify with a simple PRD (e.g., "add a todo list component")
4. Once working, expand to other providers and add robustness

This approach delivers value quickly while minimizing risk of breaking existing functionality. The existing amp/claude code paths remain untouched during development.
