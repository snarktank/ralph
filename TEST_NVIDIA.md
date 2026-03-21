# Quick Test: Ralph with NVIDIA LLM

This guide will walk you through testing Ralph with your NVIDIA API key using a simple example.

## Prerequisites
- You have an NVIDIA API key (you mentioned you have one)
- You have installed `jq` and `curl` (required dependencies)

## Step-by-Step Instructions

### 1. Set up your API key
Choose one of these methods:

**Method A: Using .env file (recommended for testing)**
```bash
# In the Ralph project directory, create a .env file:
echo "NVIDIA_API_KEY=your_actual_api_key_here" > .env

# Verify the file was created correctly:
cat .env
# Should show: NVIDIA_API_KEY=your_actual_api_key_here
```

**Method B: Export environment variable**
```bash
export NVIDIA_API_KEY="your_actual_api_key_here"
```

**Method C: Pass as command line argument (most secure)**
```bash
# You'll add this to the ralph.sh command below
--llm-api-key your_actual_api_key_here
```

### 2. Use the prepared test PRD
We have already created a simple test PRD that asks Ralph to create a verification file.

Check what we're trying to accomplish:
```bash
cat test-simple-prd.md
```

The PRD has been converted to `prd.json` and is ready to use.

### 3. Run Ralph with NVIDIA LLM
Now run Ralph using your NVIDIA API key:

```bash
# If you used Method A (.env file):
source .env  # This loads your API key from .env
./ralph.sh --llm-provider nvidia --max-iterations 5

# If you used Method B (exported variable):
./ralph.sh --llm-provider nvidia --max-iterations 5

# If you prefer Method C (command line argument):
./ralph.sh --llm-provider nvidia --llm-api-key your_actual_api_key_here --max-iterations 5
```

### 4. What to expect
Ralph will run for up to 5 iterations (you can adjust this number). In each iteration, it will:
1. Read the PRD to see what needs to be done
2. Ask the NVIDIA LLM what action to take next
3. Execute that action (creating/editing files, etc.)
4. Check if the task is complete
5. Continue until done or max iterations reached

### 5. Verify the results
After Ralph finishes, check:

```bash
# Did it create our verification file?
ls -l verification.txt

# What's in the file?
cat verification.txt

# Did it update the PRD to mark the story as complete?
cat prd.json | jq '.userStories[] | {id, title, passes}'

# What did Ralph learn during the process?
cat progress.txt
```

### 6. Expected outcome
For our test PRD, Ralph should:
1. Create a file named `verification.txt` in the project root
2. The file should contain exactly: "Ralph NVIDIA integration is working!"
3. Mark the story as complete in `prd.json`
4. Append learning notes to `progress.txt`

## Troubleshooting

If you see "parse error: Unfinished JSON term at EOF":
- This means Ralph is trying to read the progress.txt file before it has content
- This is normal in early iterations and should resolve as the progress log gets populated
- If it persists, check that your API key is valid and the NVIDIA API is accessible

If you see authentication errors:
- Double-check your API key is correct
- Ensure you haven't accidentally committed your .env file (it should be in .gitignore)

If the model doesn't work:
- We're using "abacusai/dracarys-llama-3.1-70b-instruct" by default
- You can try a different model by adding: `--llm-model "nemotron-4-340b-instruct"`

## Notes
- The .env method is for convenience during testing. Never commit .env files containing real API keys.
- Ralph maintains full backward compatibility with `--tool amp` and `--tool claude`.
- For longer-running tasks, increase `--max-iterations` (default is 10).