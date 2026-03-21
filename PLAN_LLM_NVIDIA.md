# Plan: Integrating NVIDIA LLM API Support into Ralph

## Objective
Modify Ralph to support direct API calls to NVIDIA LLM services (NVIDIA NIM, AI Foundation Models) instead of relying solely on Amp or Claude Code agents. This enables users to use NVIDIA's API keys with their preferred models while preserving Ralph's autonomous agent functionality.

## Why NVIDIA?
NVIDIA offers:
- High-performance inference via NVIDIA NIM (NVIDIA Inference Microservices)
- Access to state-of-the-art models through NVIDIA AI Enterprise
- Optimized TensorRT-LLM serving for low latency
- Potential cost advantages for specific workloads
- Enterprise-grade security and support

## NVIDIA API Overview
NVIDIA provides LLM access through:
1. **NVIDIA API Catalog** (api.nvidia.com) - Foundation models access
2. **NVIDIA NIM** - Deployment-optimized containers with standard APIs
3. **Both typically use OpenAI-compatible API format** (critical for implementation ease)

Key endpoints:
- Chat Completions: `POST https://integrate.api.nvidia.com/v1/chat/completions`
- Authentication: Bearer token with NVIDIA API key
- Model examples: `mistralai/mixtral-8x22b-instruct`, `nvidia/nemotron-4-340b-instruct`

## Core Challenge (Same as General LLM Integration)
Ralph's current design assumes the invoked tool is a fully autonomous coding agent. Standard LLM APIs only return text completions. We must implement an agent loop ourselves that uses the LLM as a reasoning engine.

## Solution Overview
Replace Ralph's tool invocation phase with a custom agent loop that:
1. Uses NVIDIA LLM API to generate responses
2. Interprets responses as structured actions (file edits, command runs, etc.)
3. Executes actions in the environment
4. Maintains conversation history and context
5. Continues until completion signal is generated

## Detailed Implementation Plan

### Phase 1: Foundation & Minimal Viable Product (NVIDIA Focused)
**Goal:** Support NVIDIA API with basic action protocol

#### 1.1 Extend Argument Parsing (ralph.sh)
Add NVIDIA-specific options:
```bash
# LLMSPECIFIC ARGS
while [[ $# -gt 0 ]]; do
  case $1 in
    --llm-provider)
      LLM_PROVIDER="$2"  # Will validate against supported: nvidia
      shift 2
      ;;
    --llm-api-key)
      LLM_API_KEY="$2"
      shift 2
      ;;
    --llm-model)
      LLM_MODEL="$2"  # Default to a NVIDIA model if not specified
      shift 2
      ;;
    --llm-base-url)
      LLM_BASE_URL="$2"  # Default to NVIDIA endpoint
      shift 2
      ;;
    # ... existing args ...
  esac
done
```

Add validation and defaults:
```bash
if [[ -n "$LLM_PROVIDER" ]]; then
  if [[ "$LLM_PROVIDER" != "nvidia" ]]; then
    echo "Error: Currently only '--llm-provider nvidia' is supported."
    exit 1
  fi
  if [[ -z "$LLM_API_KEY" ]]; then
    echo "Error: --llm-api-key required when using --llm-provider nvidia"
    exit 1
  fi
  # Set NVIDIA-specific defaults if not overridden
  LLM_BASE_URL="${LLM_BASE_URL:-https://integrate.api.nvidia.com/v1}"
  LLM_MODEL="${LLM_MODEL:-nvidia/nemotron-4-340b-instruct}"  # Example model
  TOOL="llm_direct"  # Special value to trigger direct API path
fi
```

#### 1.2 Create NVIDIA Provider Abstraction
Add helper function in ralph.sh:
```bash
llm_call_nvidia() {
  local prompt="$1"
  local history="$2"  # JSON array of previous messages
  
  # Build messages array: system + history + current prompt
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
      '{model: $model, messages: $msgs, temperature: 0.7, stream: false}')"
}
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

#### 1.4 Define Action Protocol (Unchanged from General Plan)
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

#### 1.5 Action Execution Engine (Unchanged)
Add function to interpret and execute actions (same as general plan).

#### 1.6 Prompt Construction (NVIDIA-tailored)
Instead of feeding full prompt.md/CLAUDE.md, construct LLM prompt that includes:
- Task context (from prd.json, progress.txt)
- Instructions for action format
- Conversation history
- Current objective (highest priority story)

Example system prompt (optimized for NVIDIA models):
```
You are an expert autonomous coding agent. Your goal is to implement the highest priority user story where passes: false.

You MUST respond with ONLY a valid JSON object containing:
- "thoughts": string - Your concise reasoning
- "actions": array - Action objects to execute NOW

Supported action types:
1. read_file: { "type": "read_file", "path": "relative/path/to/file" }
2. edit_file: { 
     "type": "edit_file", 
     "path": "relative/path/to/file", 
     "changes": [ 
       { "oldString": "exact text to replace", 
         "newString": "replacement text" } 
     ] 
   }
3. run_command: { 
     "type": "run_command", 
     "command": "shell command to execute", 
     "expects_success": true/false 
   }
4. finish: { "type": "finish" }  // Signals task completion

RULES:
- Only output valid JSON - no additional text before or after
- Keep thoughts brief but informative
- Make minimal, focused changes per action
- If uncertain, use read_file actions to gather information first
- ALWAYS finish with a finish action when the story is complete

Current context:
- PRD: [content from prd.json]
- Progress: [relevant progress.txt entries]
- Story to implement: [highest priority story where passes=false]
```

### Phase 2: Robustness & .env Support
**Goal:** Add environment file support and improve reliability

#### 2.1 .env File Integration
**Approach:** Optional .env loading (not required, but convenient)

Add to ralph.sh initialization (before argument parsing):
```bash
# Load .env file if it exists (for local development convenience)
if [ -f ".env" ]; then
  # shellcheck disable=SC1091
  . .env
  echo "Loaded configuration from .env file"
fi
```

**Important security considerations to document:**
- .env files MUST NOT be committed to git (already covered by .gitignore? Need to add)
- Recommend using .env.example for template
- Production systems should use environment variables directly, not .env
- Never log API keys or sensitive values

Update .gitignore to explicitly ignore .env:
```
# Ralph working files (generated during runs)
prd.json
progress.txt
.last-branch

# Environment files (containing secrets)
.env
.env.local

# Archive is optional to commit
# archive/

# OS files
.DS_Store

#Claude
.claude/
```

#### 2.2 Environment Variable Naming Convention
For clarity and to avoid conflicts:
- `NVIDIA_API_KEY` - Primary API key for NVIDIA services
- Alternative: Allow `LLM_API_KEY` to override for flexibility
- Document both options in README

In ralph.sh, prioritize:
```bash
# API Key precedence: command line > NVIDIA_API_KEY > LLM_API_KEY > .env
LLM_API_KEY="${LLM_API_KEY:-${NVIDIA_API_KEY}}"
```

#### 2.3 Enhanced NVIDIA-Specific Features
Leverage NVIDIA strengths:
- **Model flexibility**: Easy switching between NVIDIA models via --llm-model
- **Performance tuning**: Expose useful parameters like temperature, top_p
- **Streaming option**: For future enhancement (not needed for MVP)
- **Embedding models**: Potential for RAG enhancements later

Add optional parameters:
```bash
--llm-temperature) LLM_TEMPERATURE="$2"; shift 2 ;;
--llm-top-p) LLM_TOP_P="$2"; shift 2 ;;
```

Pass to API call:
```bash
... \
  --argjson temp "${LLM_TEMPERATURE:-0.7}" \
  --argjson top_p "${LLM_TOP_P:-1.0}" \
  '{model: $model, messages: $msgs, temperature: $temp, top_p: $top_p, stream: false}'
```

#### 2.4 Context Management Strategies
To prevent exceeding NVIDIA model context limits:
- **Sliding window**: Keep last K exchanges (configurable)
- **Summarization**: Every N exchanges, ask LLM to summarize history
- **Selective inclusion**: For large files, only include relevant sections
- **Token counting**: Estimate and truncate if needed (using simple heuristics)

### Phase 3: Polish & Enterprise Considerations
**Goal:** Production-ready for NVIDIA AI Enterprise environments

#### 3.1 Secure Credential Handling
- Document use of environment variables: `NVIDIA_API_KEY`, `LLM_API_KEY`
- Explicit warning: Never commit .env files containing keys
- Suggest Docker/Kubernetes secret injection for production
- Optional: Support for AWS Secrets Manager, HashiCorp Vault (advanced)

#### 3.2 NVIDIA-Specific Optimizations
- Connection pooling where supported (via curl reuse)
- Retry logic with exponential backoff for rate limits (429 errors)
- Specific error handling for NVIDIA API error codes
- Logging of request IDs for debugging with NVIDIA support

#### 3.3 Documentation & Examples
Update README.md with NVIDIA-specific usage:
```
# Using NVIDIA NIM or API Catalog
export NVIDIA_API_KEY="your-nvidia-api-key-here"
./ralph.sh --llm-provider nvidia --llm-model mistralai/mixtral-8x22b-instruct

# Using .env file (development only)
echo "NVIDIA_API_KEY=your-key" > .env
./ralph.sh --llm-provider nvidia

# Override specific parameters
./ralph.sh --llm-provider nvidia --llm-temperature 0.2 --llm-model nvidia/nemotron-4-340b-instruct
```

Create NVIDIA-specific examples in docs/
Update skills documentation if prompting conventions change significantly

#### 3.4 Integration with NVIDIA Ecosystem
Highlight compatibility with:
- NVIDIA AI Enterprise software stack
- Triton Inference Server (if self-hosting NIM)
- NeMo framework for custom model fine-tuning
- RAPIDS for data processing components

## Files to Modify
- **Primary:** `ralph.sh` - Contains all core logic changes
- **Secondary:** 
  - `.gitignore` - Add .env exclusion
  - `README.md` - Add NVIDIA usage examples
  - Possibly `AGENTS.md` template if default prompts change

## Dependencies (Verify Availability)
- **bash**: Standard shell (already required)
- **jq**: JSON parsing (already required by Ralph for prd.json handling)
- **curl**: HTTP client for API calls (needs verification - standard on most Linux/macOS, may need Windows alternative)
- **Basic text utilities**: sed, grep (standard)

## NVIDIA-Specific Advantages Over General Approach
1. **Performance**: NVIDIA-optimized models and serving stack
2. **Enterprise readiness**: NVIDIA AI Enterprise support and security
3. **Model access**: State-of-the-art models like Nemotron series
4. **Ecosystem**: Integration with broader NVIDIA AI software
5. **Potential cost efficiency**: For specific workloads on accelerated infrastructure

## Open Questions for User Input
1. **Default Model**: Which NVIDIA model should be default? 
   - Options: `mistralai/mixtral-8x22b-instruct`, `nvidia/nemotron-4-340b-instruct`, `google/gemma-2-27b-it`
2. **.env Handling**: 
   - Should we provide .env.example template?
   - Should we warn if .env is detected but seems to contain secrets?
3. **NVIDIA Features**: 
   - Should we expose NVIDIA-specific parameters like `top_k`, `min_p`?
   - Should we add support for NVIDIA embedding models for future RAG?
4. **Error Handling**: 
   - How verbose should NVIDIA-specific error messages be?
   - Should we include help links to NVIDIA documentation on common errors?

## Implementation Recommendation
Start with **Phase 1 (MVP)** for NVIDIA only:
1. Get the basic loop working with NVIDIA API
2. Implement minimal action set (read_file, edit_file, run_command, finish)
3. Verify with a simple PRD using a NVIDIA model
4. Once working, add .env support and NVIDIA-specific optimizations

This approach delivers value quickly while minimizing risk. The existing amp/claude code paths remain untouched during development, ensuring backward compatibility.

## Security Note
When using NVIDIA API:
- Treat API keys as seriously as passwords
- Use environment variables or secure secret stores in CI/CD
- Monitor API usage for unexpected charges
- Consider setting up API key restrictions in NVIDIA dashboard if available