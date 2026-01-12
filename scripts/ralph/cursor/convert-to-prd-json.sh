#!/bin/bash
# Convert PRD markdown -> prd.json using Cursor CLI (template version).
#
# Usage:
#   ./scripts/ralph/cursor/convert-to-prd-json.sh <path-to-prd-markdown> [--model MODEL] [--out OUT_JSON]
#
# Defaults:
# - MODEL: "auto"
# - OUT_JSON: ../prd.json (in scripts/ralph/ directory, same level as prd.json.example)
#
# Notes:
# - This is a convenience helper to streamline PRD->prd.json conversion.
# - It is intentionally separate from the Ralph iteration loop.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PRD_MD_FILE=""
MODEL="auto"
OUT_JSON="$SCRIPT_DIR/../prd.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      MODEL="${2:-}"
      shift 2
      ;;
    --out)
      OUT_JSON="${2:-}"
      shift 2
      ;;
    -*)
      echo "Unknown flag: $1" >&2
      exit 2
      ;;
    *)
      if [[ -z "$PRD_MD_FILE" ]]; then
        PRD_MD_FILE="$1"
      else
        echo "Unexpected argument: $1" >&2
        exit 2
      fi
      shift
      ;;
  esac
done

if [[ -z "$PRD_MD_FILE" ]]; then
  echo "Usage: $0 <path-to-prd-markdown> [--model MODEL] [--out OUT_JSON]" >&2
  exit 2
fi

PROMPT_TEMPLATE_FILE="$SCRIPT_DIR/prompt.convert-to-prd-json.md"
if [[ ! -f "$PROMPT_TEMPLATE_FILE" ]]; then
  echo "Error: missing prompt template: $PROMPT_TEMPLATE_FILE" >&2
  exit 1
fi

EXAMPLE_FILE="$SCRIPT_DIR/../prd.json.example"
if [[ ! -f "$EXAMPLE_FILE" ]]; then
  echo "Error: missing example file: $EXAMPLE_FILE" >&2
  exit 1
fi

CURSOR_BIN="${RALPH_CURSOR_BIN:-cursor}"

PROMPT_TEXT="$(
  cat "$PROMPT_TEMPLATE_FILE"
  printf "\n\n---\n\n"
  printf "## Inputs\n"
  printf "Read the PRD markdown file at: %s\n" "$PRD_MD_FILE"
  printf "Read the format reference at: %s\n" "$EXAMPLE_FILE"
  printf "\n"
  printf "## Output\n"
  printf "Write/overwrite prd.json at: %s\n" "$OUT_JSON"
)"

exec "$CURSOR_BIN" --model "$MODEL" --print --force --approve-mcps "$PROMPT_TEXT" </dev/null

