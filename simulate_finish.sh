#!/bin/bash

# Source necessary functions from ralph.sh
source ./ralph.sh

# Set up the test exactly as Ralph would see it
SCRIPT_DIR="$(pwd)"
PRD_FILE="$SCRIPT_DIR/prd.json"

echo "=== SIMULATING FINISH ACTION ==="
echo "PRD file: $PRD_FILE"
echo "PRD contents:"
cat "$PRD_FILE"
echo

# Test the finish action logic
echo "Executing finish action logic..."

# This is copied from the finish block in ralph.sh
_sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$PRD_FILE" 2>/dev/null)
echo "Current failing story ID: '$_sid'"

if [[ -n "$_sid" && "$_sid" != "null" ]]; then
  echo "Found failing story: $_sid"
  
  _verification_cmd=$(jq -r --arg id "$_sid" '.userStories[] | select(.id == $id) | .verificationCommand // empty' "$PRD_FILE" 2>/dev/null)
  echo "Verification command: '$_verification_cmd'"
  
  if [[ -n "$_verification_cmd" && "$_verification_cmd" != "null" ]]; then
    echo "Running verification command..."
    if eval "$_verification_cmd"; then
      echo "VERIFICATION RESULT: PASSED"
      # In the real finish block, this would mark the story as passed
    else
      _exit_code=$?
      echo "VERIFICATION RESULT: FAILED (exit code $_exit_code)"
      # In the real finish block, this would increment rejection counter and continue (not return 0)
    fi
  else
    echo "No verification command - would proceed as before"
  fi
else
  echo "No failing stories found"
fi

echo
echo "=== TEST COMPLETE ==="