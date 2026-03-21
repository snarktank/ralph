# Testing Ralph with NVIDIA LLM Integration

## Prerequisites
1. Obtain an NVIDIA API key from https://api.nvidia.com/
2. Install required dependencies:
   - jq (for JSON parsing)
   - curl (for HTTP requests)
   - Git (already required for Ralph)

## Setup

### Option 1: Using .env file (development only)
```bash
# Create .env file in Ralph project root
echo "NVIDIA_API_KEY=your_actual_api_key_here" > .env

# Verify .env is ignored by git (should be in .gitignore)
cat .gitignore | grep -E '\.env'
```

### Option 2: Using environment variable
```bash
export NVIDIA_API_KEY="your_actual_api_key_here"
```

### Option 3: Command line argument (most secure)
```bash
./ralph.sh --llm-provider nvidia --llm-api-key your_actual_api_key_here
```

## Running a Test

We have prepared a simple test PRD that creates a verification file:

```bash
# Check the test PRD
cat test-simple-prd.md

# Load it as the active PRD
cp test-simple-prd.md tasks/prd-test-simple-file.md
# Note: Ralph expects prd.json, so we need to convert or directly use
# For quick testing, we'll directly use the prepared prd.json

# Run Ralph with NVIDIA LLM
source .env  # loads NVIDIA_API_KEY if using .env method
./ralph.sh --llm-provider nvidia --max-iterations 10
```

## Expected Behavior
1. Ralph will load the PRD from `prd.json`
2. It will identify the highest priority story where `passes: false`
3. In each iteration, it will:
   - Construct a prompt for the NVIDIA LLM including the story context
   - Call the NVIDIA API
   - Parse the LLM's response as JSON actions
   - Execute those actions (creating/editing files, running commands)
   - Check for completion signal (`<promise>COMPLETE</promise>`)
   - Update progress.txt and prd.json if successful
4. The process continues until:
   - The story is marked as complete (`passes: true`)
   - Max iterations is reached
   - An unrecoverable error occurs

## Verification
After Ralph completes (either successfully or by reaching max iterations), check:

```bash
# See if the verification file was created
ls -la verification.txt

# Check its content
cat verification.txt

# Review the progress log
cat progress.txt

# Check the updated PRD
cat prd.json | jq '.userStories[] | {id, title, passes}'
```

## Troubleshooting

### Common Issues
1. **"jq is required but not installed"** - Install jq from https://github.com/jqlang/jq/releases
2. **API authentication errors** - Verify your NVIDIA API key is correct and has sufficient permissions
3. **Model not found errors** - Try different model identifiers (we use `abacusai/dracarys-llama-3.1-70b-instruct` by default)
4. **LLM not returning valid JSON** - The system includes fallback parsing, but you may need to adjust the system prompt in ralph.sh

### Debugging
To see detailed debug information, run:
```bash
source .env && bash -x ralph.sh --llm-provider nvidia --max-iterations 2 2>&1 | head -30
```

To test the NVIDIA API connection directly:
```bash
source .env && curl -s -X POST "https://integrate.api.nvidia.com/v1/chat/completions" \
  -H "Authorization: Bearer $NVIDIA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"abacusai/dracarys-llama-3.1-70b-instruct","messages":[{"role":"user","content":"Say hello in JSON: {\"message\":\"test\"}"}],"max_tokens":20}' | jq .
```

## Notes
- The .env method is for convenience during development ONLY. Never commit .env files containing API keys.
- For production use, prefer environment variables or secret management systems.
- Ralph maintains backward compatibility with `--tool amp` and `--tool claude` options.
- The NVIDIA integration uses an agent loop that expects the LLM to return structured JSON actions.