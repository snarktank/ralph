#!/bin/bash

# Test rejection counter and critical hint functionality

SCRIPT_DIR="$(pwd)"
REJECTION_FILE="$SCRIPT_DIR/.rejection_count"

echo "=== Testing Rejection Counter ==="

# Initialize
echo "0" > "$REJECTION_FILE"
echo "Initialized rejection counter to: $(cat "$REJECTION_FILE")"

# Simulate 5 verification failures
for i in {1..5}; do
  echo -e "\n--- Failure $i ---"
  
  # Simulate a failed verification (we'll just increment the counter)
  _current=$(cat "$REJECTION_FILE")
  _new=$((_current + 1))
  echo "$_new" > "$REJECTION_FILE"
  
  echo "Rejection count after increment: $_new"
  
  if [[ $_new -ge 5 ]]; then
    echo "🚨 CRITICAL HINT: 5 consecutive verification failures"
    echo "   Consider re-examining your approach or breaking down the problem further"
  fi
done

echo -e "\n=== Test Complete ==="
echo "Final rejection count: $(cat "$REJECTION_FILE")"

# Cleanup
rm -f "$REJECTION_FILE"