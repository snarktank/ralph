# Ralph Verification Feature - User Guide

This guide explains how to test the Test-Driven Ralph verification feature that was recently implemented.

## Overview

The verification feature adds test-driven development capabilities to Ralph. When an agent calls the `finish` action on a story that has a `verificationCommand` field in the PRD, Ralph will:

1. Execute the verification command
2. If it passes (exit code 0): mark the story as passed
3. If it fails (non-zero exit code): 
   - Increment a rejection counter
   - Provide feedback
   - After 5 consecutive failures, show a critical hint
   - Continue the loop (don't mark story as passed)

If no verificationCommand exists, Ralph behaves as before.

## Prerequisites

1. WSL2 installed with a Linux distribution (Ubuntu recommended)
2. Ralph project cloned or copied to WSL filesystem
3. Basic bash utilities (jq, etc.) - these should already be present

## Setup Instructions

### 1. Copy Ralph Project to WSL

If you haven't already, copy the Ralph project to your WSL filesystem:

```bash
# From Windows, copy to WSL (example for Ubuntu)
cp -r /mnt/g/ralph ~/ralph
# Or if already in WSL:
cd ~/ralph
```

### 2. Verify Dependencies

Ensure required tools are available:

```bash
# Check for jq (required for PRD parsing)
which jq

# Check for bash
which bash

# Both should return paths, not "not found"
```

## Testing the Verification Feature

### Step 1: Prepare a Test PRD

Create or modify a PRD file (`prd.json`) with a story that includes a verification command:

```json
{
  "project": "Test Project",
  "branchName": "test",
  "description": "Testing verification feature",
  "userStories": [
    {
      "id": "TEST-001",
      "title": "Create working hello script",
      "verificationCommand": "bash hello.sh | grep -qF 'Hello from Ralph TDD!'",
      "passes": false
    }
  ]
}
```

### Step 2: Create Test Script

Create the script that will be verified (`hello.sh`):

```bash
#!/bin/bash
echo "Hello from Ralph TDD!"
```

Make it executable:

```bash
chmod +x hello.sh
```

### Step 3: Initialize Rejection Counter (Optional)

Ralph will automatically create this file when needed, but you can initialize it:

```bash
echo 0 > .rejection_count
```

### Step 4: Run Ralph

Execute Ralph with a limited number of iterations for testing:

```bash
# Using default tool (amp)
./ralph.sh 3

# Or specify max iterations
./ralph.sh --tool amp 5
```

### Step 5: Observe Results

#### Case A: Verification Passes
If `hello.sh` outputs "Hello from Ralph TDD!":
- You should see: `VERIFICATION: ✅ PASSED - Story TEST-001 verified successfully`
- The story will be marked as passed in PRD
- Ralph will continue or complete depending on other stories

#### Case B: Verification Fails
If `hello.sh` outputs something else:
- You should see: `VERIFICATION: ❌ FAILED - Story TEST-001 verification failed (exit 1)`
- Rejection counter increments: `Rejection count: 1`
- Ralph continues to next iteration

#### Case C: Multiple Failures → Critical Hint
After 5 consecutive verification failures:
- You'll see: `VERIFICATION: 🚨 CRITICAL HINT - 5 consecutive verification failures`
- Suggestion: `Consider re-examining your approach or breaking down the problem further`

### Step 6: Reset and Retry

Fix your script and try again:

```bash
# Update hello.sh to pass verification
echo '#!/bin/bash' > hello.sh
echo 'echo "Hello from Ralph TDD!"' >> hello.sh
chmod +x hello.sh

# Reset rejection counter (happens automatically on success)
echo 0 > .rejection_count

# Run Ralph again
./ralph.sh 2
```

## Sample Test Scenarios

### Scenario 1: Basic Pass/Fail Cycle

1. Create failing verification command:
   ```json
   {
     "id": "TEST-001",
     "title": "Test verification",
     "verificationCommand": "bash test.sh | grep -qF 'SUCCESS'",
     "passes": false
   }
   ```
   
2. Create failing test.sh:
   ```bash
   #!/bin/bash
   echo "FAIL"
   ```
   
3. Run Ralph → Observe failure → Rejection counter increments
   
4. Fix test.sh:
   ```bash
   #!/bin/bash
   echo "SUCCESS"
   ```
   
5. Run Ralph again → Observe success → Story marked as passed

### Scenario 2: Critical Hint Trigger

1. Keep a consistently failing verification command
2. Run Ralph 5 times in succession
3. On the 5th failure, observe the critical hint message
4. Fix the command and see the counter reset

## Expected File States

After successful verification:
- `hello.sh` contains the expected output
- `.rejection_count` contains `0` (reset on success)
- `prd.json` has `"passes": true` for the story
- Progress log shows completion

After failed verification:
- `.rejection_count` contains incremented value
- Story remains `"passes": false`
- No changes applied to live ralph.sh (due to sandbox validation)

## Troubleshooting

### "command not found" errors
Ensure you're running commands from within the Ralph project directory where `ralph.sh` is located.

### Permission denied on scripts
Make scripts executable: `chmod +x hello.sh`

### PRD parsing errors
Validate your JSON: `jq . prd.json` should return formatted JSON without errors

### No verification output
Check that:
1. Story actually has `passes: false`
2. Story has a `verificationCommand` field
3. You're looking at the correct PRD file path

## Safety Notes

- The verification feature works WITH the existing staging sandbox
- Syntax validation of ralph.sh changes happens BEFORE verification
- Live ralph.sh is only updated if BOTH sandbox validation AND verification pass
- Your original ralph.sh is protected by the staging mechanism

## Extending for Your Own Use

To use this feature in your own stories:
1. Add a `verificationCommand` field to any story in your PRD
2. Make the command return exit 0 on success, non-zero on failure
3. Keep commands idempotent and safe to run multiple times
4. Consider using command substitution for complex checks:
   ```json
   "verificationCommand": "[[ $(wc -l < output.txt) -gt 5 ]] && [[ $(grep -c 'ERROR' output.txt) -eq 0 ]]"
   ```