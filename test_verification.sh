#!/bin/bash
source ralph.sh

# Set up test variables
SCRIPT_DIR="/g/ralph"
PRD_FILE="$SCRIPT_DIR/prd.json"

# Create a test PRD with a failing verification command
cat > "$PRD_FILE" << 'PRD_EOF'
{
  "project": "Test",
  "branchName": "test",
  "description": "Test verification",
  "userStories": [
    {
      "id": "TEST-001",
      "title": "Create failing script",
      "verificationCommand": "bash /g/ralph/hello.sh | grep -qF 'Hello from Ralph TDD!'",
      "passes": false
    }
  ]
}
PRD_EOF

# Create a hello.sh that fails
echo '#!/bin/bash' > /g/ralph/hello.sh
echo 'echo "Wrong output"' >> /g/ralph/hello.sh
chmod +x /g/ralph/hello.sh

# Initialize rejection counter
echo 0 > "$SCRIPT_DIR/.rejection_count"

# Simulate the finish action logic
_sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$SCRIPT_DIR/prd.json" 2>/dev/null)
echo "Story ID: $_sid"

if [[ -n "$_sid" && "$_sid" != "null" ]]; then
  _verification_cmd=$(jq -r --arg id "$_sid" '.userStories[] | select(.id == $id) | .verificationCommand // empty' "$SCRIPT_DIR/prd.json" 2>/dev/null)
  echo "Verification command: $_verification_cmd"
  
  if [[ -n "$_verification_cmd" && "$_verification_cmd" != "null" ]]; then
    echo "Running verification..."
    if eval "$_verification_cmd"; then
      echo "Verification PASSED"
    else
      _exit_code=$?
      echo "Verification FAILED with exit code $_exit_code"
      _current_rejections=$(cat "$SCRIPT_DIR/.rejection_count")
      _new_rejections=$((_current_rejections + 1))
      echo $_new_rejections > "$SCRIPT_DIR/.rejection_count"
      echo "Rejection count: $_new_rejections"
      if [[ $_new_rejections -ge 5 ]]; then
        echo "CRITICAL HINT: 5 consecutive failures"
      fi
    fi
  fi
fi

# Show final state
echo "Final PRD:"
cat "$SCRIPT_DIR/prd.json"
echo "Rejection count: $(cat "$SCRIPT_DIR/.rejection_count")"
