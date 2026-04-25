#!/bin/bash

RATE_LIMIT_FALLBACK_WAIT_SECONDS=$((5 * 60 * 60))

is_rate_limit_output() {
  local output_lower
  output_lower=$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')

  [[ "$output_lower" == *"usage limit reached"* ]] \
    || [[ "$output_lower" == *"rate limit"* ]] \
    || [[ "$output_lower" == *"quota exceeded"* ]] \
    || [[ "$output_lower" == *"hit your"* ]]
}

extract_rate_limit_reset_details() {
  local output="$1"
  local reset_time=""
  local reset_timezone=""
  local reset_regex='[Rr]esets?([[:space:]]+at)?[[:space:]]+([^()[:space:]][^()]*)[[:space:]]\(([^)]+)\)'

  if [[ "$output" =~ $reset_regex ]]; then
    reset_time="${BASH_REMATCH[2]}"
    reset_timezone="${BASH_REMATCH[3]}"
  fi

  if [[ -n "$reset_time" && -n "$reset_timezone" ]]; then
    printf '%s|%s\n' "$reset_time" "$reset_timezone"
    return 0
  fi

  return 1
}

calculate_rate_limit_wait_seconds() {
  local reset_time="$1"
  local reset_timezone="$2"
  local seconds_until_reset=""

  if ! command -v python3 >/dev/null 2>&1; then
    return 1
  fi

  seconds_until_reset=$(python3 - "$reset_time" "$reset_timezone" <<'PY'
from datetime import datetime, timedelta
from zoneinfo import ZoneInfo
import sys

reset_time = sys.argv[1].strip().lower().replace(".", "")
reset_timezone = sys.argv[2].strip()

formats = ("%I%p", "%I:%M%p", "%I %p", "%I:%M %p")
now = datetime.now(ZoneInfo(reset_timezone))

parsed_time = None
for fmt in formats:
    try:
        parsed_time = datetime.strptime(reset_time.upper(), fmt)
        break
    except ValueError:
        continue

if parsed_time is None:
    raise SystemExit(1)

reset_at = now.replace(
    hour=parsed_time.hour,
    minute=parsed_time.minute,
    second=0,
    microsecond=0,
)

if reset_at <= now:
    reset_at += timedelta(days=1)

print(max(1, int((reset_at - now).total_seconds())))
PY
  ) || return 1

  if [[ "$seconds_until_reset" =~ ^[0-9]+$ ]]; then
    printf '%s\n' "$seconds_until_reset"
    return 0
  fi

  return 1
}

format_wait_duration() {
  local total_seconds="$1"
  local hours=$((total_seconds / 3600))
  local minutes=$(((total_seconds % 3600) / 60))
  local seconds=$((total_seconds % 60))

  if (( hours > 0 )); then
    printf '%dh %dm %ds' "$hours" "$minutes" "$seconds"
  elif (( minutes > 0 )); then
    printf '%dm %ds' "$minutes" "$seconds"
  else
    printf '%ds' "$seconds"
  fi
}

handle_rate_limit() {
  local output="$1"
  local wait_seconds="$RATE_LIMIT_FALLBACK_WAIT_SECONDS"
  local reset_details=""
  local reset_time=""
  local reset_timezone=""
  local wait_duration=""

  if ! is_rate_limit_output "$output"; then
    return 1
  fi

  echo "Claude hit a rate limit. Waiting for quota reset before retrying this iteration..."

  if reset_details=$(extract_rate_limit_reset_details "$output"); then
    reset_time="${reset_details%%|*}"
    reset_timezone="${reset_details#*|}"

    if wait_seconds=$(calculate_rate_limit_wait_seconds "$reset_time" "$reset_timezone"); then
      wait_duration=$(format_wait_duration "$wait_seconds")
      echo "Detected reset time: $reset_time ($reset_timezone). Sleeping for $wait_duration."
    else
      wait_duration=$(format_wait_duration "$wait_seconds")
      echo "Couldn't calculate reset time from Claude output. Falling back to $wait_duration."
    fi
  else
    wait_duration=$(format_wait_duration "$wait_seconds")
    echo "Couldn't parse reset details from Claude output. Falling back to $wait_duration."
  fi

  sleep "$wait_seconds"
  return 0
}
