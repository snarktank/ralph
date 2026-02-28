#!/usr/bin/env bash
set -euo pipefail

# Skip entirely if requested
if [ "${SKIP_AIDER_REVIEW:-0}" = "1" ]; then
  echo "⏭️  SKIP_AIDER_REVIEW=1 — skipping aider review step"
  exit 0
fi

# Fallback defaults for safety
: "${LLM_API_BASE:=http://localhost:8080/v1}"
: "${LLM_API_KEY:=dummy}"

# Verify server is alive before blocking the loop
if ! curl -sf --max-time 5 "$LLM_API_BASE/models" > /dev/null; then
  echo -e "\033[33m⚠️  LLM server not responding at $LLM_API_BASE — skipping aider review\033[0m"
  echo "   Set SKIP_AIDER_REVIEW=1 to suppress this warning"
  exit 0
fi

echo "🔍 Running Aider review via $LLM_API_BASE"

aider \
  --model qwen3-coder-next \
  --openai-api-base "$LLM_API_BASE" \
  --openai-api-key "$LLM_API_KEY" \
  --yes \
  --no-auto-commit \
  --message "Review the git diff since the last commit. Focus on:
- Correctness against PRD
- Test coverage & quality
- Type safety & anti-patterns
- Consistency with patterns in AGENTS.md

Return:
1. Critical issues (must fix)
2. Optional improvements
3. New patterns to add to AGENTS.md"
