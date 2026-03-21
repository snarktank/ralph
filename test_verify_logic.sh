#!/bin/bash

# Test the verification logic directly without invoking Ralph

# Set up test variables
SCRIPT_DIR="$(pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"

echo "=== Testing Verification Logic ==="
echo "PRD file: $PRD_FILE"
echo

# Show current PRD
echo "Current PRD contents:"
cat "$PRD_FILE"
echo

# Get current failing story (this is what Ralph does)
_sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$PRD_FILE" 2>/dev/null)
echo "Current failing story ID: '$_sid'"
echo

if [[ -n "$_sid" && "$_sid" != "null" ]]; then
  echo "Found failing story: $_sid"
  
  # Check if story has verificationCommand
  _verification_cmd=$(jq -r --arg id "$_sid" '.userStories[] | select(.id == $id) | .verificationCommand // empty' "$PRD_FILE" 2>/dev/null)
  echo "Verification command: '$_verification_cmd'"
  echo
  
  if [[ -n "$_verification_cmd" && "$_verification_cmd" != "null" ]]; then
    echo "Running verification command..."
    echo "Command: $_verification_cmd"
    
    if eval "$_verification_cmd"; then
      echo "RESULT: VERIFICATION PASSED"
      echo "In Ralph, this would mark the story as passed and return 0"
    else
      _exit_code=$?
      echo "RESULT: VERIFICATION FAILED (exit code $_exit_code)"
      echo "In Ralph, this would:"
      echo "  1. Increment rejection counter"
      echo "  2. Log action end with success=false"
      echo "  3. Continue (not return 0)"
      
      # Show what the rejection counter would do
      REJECTION_FILE="$SCRIPT_DIR/.rejection_count"
      if [[ ! -f "$REJECTION_FILE" ]]; then
        echo "0" > "$REJECTION_FILE"
      fi
      _current=$(cat "$REJECTION_FILE")
      _new=$((_current + 1))
      echo "$_new" > "$REJECTION_FILE"
      echo "  4. New rejection count: $_new"
      if [[ $_new -ge 5 ]]; then
        echo "  5. 🚨 CRITICAL HINT would be shown"
      fi
    fi
  else
    echo "No verification command - would proceed as before"
  fi
else
  echo "No failing stories found"
fi

echo
echo "=== Test Complete ==="
