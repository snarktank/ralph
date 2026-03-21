#!/bin/bash
source .env

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"

# Get the context as the agent would
context=$(jq -r '.userStories[] | select(.passes == false) | .' "$PRD_FILE" 2>/dev/null | head -n 1)
if [[ -z "$context" ]]; then
  context="{}"
fi

echo "Context JSON:"
echo "$context" | jq .
echo ""

# Build the prompt as the agent would
instructions=$(cat <<EOF
You are an autonomous coding agent implementing user stories from a PRD. You MUST respond with ONLY a valid JSON object and nothing else. The JSON must have exactly these two fields:
1. 'thoughts': a string explaining your reasoning
2. 'actions': an array of action objects to execute

Each action object MUST have a 'type' field. Supported types:
- read_file: { "type": "read_file", "path": "relative/path/file.ext" }
- edit_file: { "type": "edit_file", "path": "relative/path/file.ext", "changes": [ {"oldString": "exact text to find", "newString": "replacement text"} ] }
  (Alternative format also accepted: { "type": "create_file", "path": "relative/path/file.ext", "content": "file content" } )
- run_command: { "type": "run_command", "command": "shell command to execute", "expects_success": true/false }
- finish: { "type": "finish" }  // Use when the story is completely implemented

IMPORTANT: The PRD is located at "prd.json" and the progress log is located at "progress.txt" in the project root. Use these exact file names when referring to these files.

DO NOT include any text outside the JSON object. DO NOT include explanations, markdown formatting, or additional fields. Your entire response must be parseable as JSON.

Current story context:
EOF
)

prompt="$instructions
$context

Recent progress (last 5 lines):
$(tail -n 5 "$SCRIPT_DIR/progress.txt" 2>/dev/null || echo "(no progress yet)")"

echo "=== PROMPT SENT TO LLM ==="
echo "$prompt"
echo "=========================="
echo ""

# Call the LLM
LLM_MODEL="abacusai/dracarys-llama-3.1-70b-instruct"
LLM_BASE_URL="https://integrate.api.nvidia.com/v1"
LLM_TEMPERATURE="0.7"

response=$(curl -s -X POST "${LLM_BASE_URL}/chat/completions" \
  -H "Authorization: Bearer $NVIDIA_API_KEY" \
  -H "Content-Type: application/json" \
  -d "$(jq -n \
    --arg model "$LLM_MODEL" \
    --arg sys "$instructions" \
    --arg prompt "$context

Recent progress (last 5 lines):
$(tail -n 5 "$SCRIPT_DIR/progress.txt" 2>/dev/null || echo "(no progress yet)")" \
    '{model: $model, messages: [ {role: "system", content: $sys}, {role: "user", content: $prompt} ], temperature: 0.7, stream: false}')")

echo "=== RAW LLM RESPONSE ==="
echo "$response" | jq .
echo "========================"
echo ""

# Extract content
content=$(echo "$response" | jq -r '.choices[0].message.content // empty')
echo "=== EXTRACTED CONTENT ==="
echo "$content"
echo "========================"
echo ""

# Try to parse as JSON
if [[ -n "$content" && "$content" != "null" ]]; then
  echo "Attempting to parse as JSON:"
  parsed=$(echo "$content" | jq . 2>/dev/null) || echo "FAILED TO PARSE AS JSON"
  echo "$parsed"
else
  echo "Content is empty or null"
fi