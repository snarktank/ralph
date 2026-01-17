#!/usr/bin/env bash

set -euo pipefail

# usage: ./ralph.sh <prd-file-or-dir> <max-iterations>
if [ $# -lt 2 ]; then
  echo "Usage: $0 <prd-file-or-dir> <max-iterations>"
  exit 1
fi

PRD_INPUT="$1"
MAX_ITERATIONS="$2"

# Resolve PRD JSON file
if [ -d "$PRD_INPUT" ]; then
  PRD_JSON="$PRD_INPUT/prd.json"
else
  PRD_JSON="$PRD_INPUT"
fi

if [ ! -f "$PRD_JSON" ]; then
  echo "❌ PRD file not found at: $PRD_JSON"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROGRESS_DIR="$SCRIPT_DIR/progress"

# jq filters (more robust for Claude Code stream-json)
stream_text='
  (select(.type == "assistant").message.content[]?.text? // empty),
  (.event.delta.text? // empty)
'

final_result='select(.type == "result").result // empty'

echo "🚀 Starting Ralph with streaming output"
echo "📋 PRD: $PRD_JSON"
echo "📂 Progress dir: $PROGRESS_DIR"
echo "🔁 Max iterations: $MAX_ITERATIONS"

mkdir -p "$PROGRESS_DIR"

for i in $(seq 1 "$MAX_ITERATIONS"); do
  echo "========================================"
  echo "Iteration $i / $MAX_ITERATIONS"
  echo "========================================"

  TMPFILE=$(mktemp)
  trap 'rm -f "$TMPFILE"' EXIT

  echo "⬇️ Streaming Claude Code output…"

  claude \
    --verbose \
    --print \
    --output-format stream-json \
    "@$SCRIPT_DIR/prompt.md @$PRD_JSON $PROGRESS_DIR/*.md" \
  | grep --line-buffered '^{' \
  | tee "$TMPFILE" \
  | jq --unbuffered -rj "$stream_text"

  RESULT=$(jq -r "$final_result" "$TMPFILE")

  echo
  echo "➤ Claude result (final): $RESULT"
  echo

  if [[ "$RESULT" == *"<promise>COMPLETE</promise>"* ]]; then
    echo "🎉 Ralph complete after $i iterations."
    exit 0
  fi

  rm -f "$TMPFILE"
done

echo "⚠️ Max iterations reached without completing all tasks."
exit 1