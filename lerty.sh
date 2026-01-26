#!/bin/bash
# Ralph + Lerty Integration
# Push notifications, Live Activities, and human-in-the-loop approvals via Lerty API

LERTY_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LERTY_CONFIG_DIR="$LERTY_SCRIPT_DIR/.lerty"
LERTY_CONFIG_FILE="$LERTY_CONFIG_DIR/config.json"
LERTY_SESSION_FILE="$LERTY_CONFIG_DIR/session.json"

# Check if Lerty integration is enabled and configured
lerty_enabled() {
  if [ -f "$LERTY_CONFIG_FILE" ]; then
    local enabled
    enabled=$(jq -r '.enabled // false' "$LERTY_CONFIG_FILE" 2>/dev/null)
    [ "$enabled" = "true" ]
  else
    return 1
  fi
}

# Load configuration
lerty_load_config() {
  if [ ! -f "$LERTY_CONFIG_FILE" ]; then
    echo "Lerty config not found. Run lerty-setup.sh first." >&2
    return 1
  fi

  LERTY_API_URL=$(jq -r '.apiUrl' "$LERTY_CONFIG_FILE")
  LERTY_API_KEY=$(jq -r '.apiKey' "$LERTY_CONFIG_FILE")
  LERTY_CONVERSATION_ID=$(jq -r '.conversationId' "$LERTY_CONFIG_FILE")
  LERTY_AGENT_ID=$(jq -r '.agentId' "$LERTY_CONFIG_FILE")
  LERTY_USER_EMAIL=$(jq -r '.userEmail' "$LERTY_CONFIG_FILE")
  LERTY_POLL_INTERVAL=$(jq -r '.pollInterval // 5' "$LERTY_CONFIG_FILE")

  export LERTY_API_URL LERTY_API_KEY LERTY_CONVERSATION_ID LERTY_AGENT_ID LERTY_USER_EMAIL LERTY_POLL_INTERVAL
}

# ============================================================
# Push Notifications
# ============================================================

# Send a push notification (non-blocking)
# Usage: lerty_notify "Title" "Body" "priority"
# Priority: low, medium, high, critical
lerty_notify() {
  local title="$1"
  local body="$2"
  local priority="${3:-medium}"

  lerty_load_config || return 1

  curl -s -X POST "$LERTY_API_URL/api/push/send" \
    -H "Authorization: Bearer $LERTY_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{
      "email": "'"$LERTY_USER_EMAIL"'",
      "title": "'"$(echo "$title" | sed 's/"/\\"/g')"'",
      "body": "'"$(echo "$body" | sed 's/"/\\"/g')"'",
      "priority": "'"$priority"'"
    }' > /dev/null 2>&1
}

# ============================================================
# Live Activities
# ============================================================

# Start a Live Activity for a story
# Usage: lerty_start_activity "story_id" "story_title"
lerty_start_activity() {
  local story_id="$1"
  local story_title="$2"
  local session_id="ralph-$(date +%s)-$$"

  lerty_load_config || return 1

  # Save session ID
  mkdir -p "$LERTY_CONFIG_DIR"
  echo "{\"live_activity_id\": \"$session_id\", \"story_id\": \"$story_id\"}" > "$LERTY_SESSION_FILE"

  curl -s -X POST "$LERTY_API_URL/api/push/send" \
    -H "Authorization: Bearer $LERTY_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{
      "email": "'"$LERTY_USER_EMAIL"'",
      "title": "Ralph: '"$story_id"'",
      "body": "'"$(echo "$story_title" | sed 's/"/\\"/g')"'",
      "priority": "high",
      "data": {
        "id": "'"$session_id"'",
        "notification_type": "workflow_tracking",
        "plugin_name": "Ralph",
        "category": "coding",
        "item_title": "'"$(echo "$story_title" | sed 's/"/\\"/g')"'",
        "item_subtitle": "Story '"$story_id"'",
        "status": "processing",
        "approval_required": false,
        "workflow_state": {
          "current_step": 1,
          "total_steps": 5,
          "step_name": "Starting",
          "progress_percent": 0
        }
      }
    }' > /dev/null 2>&1

  echo "Lerty: Started Live Activity for $story_id"
}

# Update Live Activity progress
# Usage: lerty_update_progress "current_step" "total_steps" "step_name" "progress_percent" "story_title"
lerty_update_progress() {
  local current_step="$1"
  local total_steps="$2"
  local step_name="$3"
  local progress_percent="$4"
  local story_title="${5:-Ralph Task}"

  lerty_load_config || return 1

  # Get session ID
  local session_id
  if [ -f "$LERTY_SESSION_FILE" ]; then
    session_id=$(jq -r '.live_activity_id // empty' "$LERTY_SESSION_FILE")
  fi

  if [ -z "$session_id" ]; then
    session_id="ralph-$(date +%s)-$$"
    mkdir -p "$LERTY_CONFIG_DIR"
    echo "{\"live_activity_id\": \"$session_id\"}" > "$LERTY_SESSION_FILE"
  fi

  curl -s -X POST "$LERTY_API_URL/api/push/live-activity/update" \
    -H "Authorization: Bearer $LERTY_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{
      "email": "'"$LERTY_USER_EMAIL"'",
      "activity_id": "'"$session_id"'",
      "content_state": {
        "itemTitle": "'"$(echo "$story_title" | sed 's/"/\\"/g')"'",
        "status": "'"$(echo "$step_name" | sed 's/"/\\"/g')"'",
        "currentStep": '"$current_step"',
        "totalSteps": '"$total_steps"',
        "progressPercent": '"$progress_percent"'
      }
    }' > /dev/null 2>&1
}

# End a Live Activity
# Usage: lerty_end_activity "status" (completed, cancelled, failed)
lerty_end_activity() {
  local status="${1:-completed}"

  lerty_load_config || return 1

  if [ ! -f "$LERTY_SESSION_FILE" ]; then
    return 0
  fi

  local session_id
  session_id=$(jq -r '.live_activity_id // empty' "$LERTY_SESSION_FILE")

  if [ -z "$session_id" ]; then
    return 0
  fi

  # Send final update
  lerty_update_progress 5 5 "$status" 100

  # Clean up session
  rm -f "$LERTY_SESSION_FILE"

  echo "Lerty: Ended Live Activity ($status)"
}

# ============================================================
# Human-in-the-Loop Approvals
# ============================================================

# Send an interactive approval request and wait for response
# Usage: lerty_request_approval "trigger_type" "title" "details"
# Returns: 0 if approved, 1 if rejected, 2 on error
lerty_request_approval() {
  local trigger_type="$1"
  local title="$2"
  local details="$3"
  local callback_id="ralph-$(date +%s)-$$"

  lerty_load_config || return 2

  echo "Lerty: Requesting approval - $title"
  echo "Lerty: Waiting for response in Lerty app..."

  # Send interactive message
  local response
  response=$(curl -s -X POST "$LERTY_API_URL/api/v1/conversations/$LERTY_CONVERSATION_ID/interactive" \
    -H "Authorization: Bearer $LERTY_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{
      "callback_url": "https://httpbin.org/post",
      "callback_id": "'"$callback_id"'",
      "elements": [
        {"type": "header", "text": "'"$(echo "$title" | sed 's/"/\\"/g')"'"},
        {"type": "text", "text": "'"$(echo "$details" | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')"'"},
        {"type": "divider"},
        {"type": "boolean", "approve_label": "Approve", "reject_label": "Reject", "with_message": true}
      ]
    }')

  local message_id
  message_id=$(echo "$response" | jq -r '.message_id // empty')

  if [ -z "$message_id" ]; then
    echo "Lerty: Error sending approval request"
    echo "Lerty: Response: $response"
    return 2
  fi

  echo "Lerty: Approval request sent (message: $message_id)"

  # Save pending approval to session
  mkdir -p "$LERTY_CONFIG_DIR"
  local existing_session="{}"
  [ -f "$LERTY_SESSION_FILE" ] && existing_session=$(cat "$LERTY_SESSION_FILE")
  echo "$existing_session" | jq '. + {"pending_approval": {"message_id": "'"$message_id"'", "callback_id": "'"$callback_id"'", "trigger": "'"$trigger_type"'"}}' > "$LERTY_SESSION_FILE"

  # Poll for response (wait indefinitely)
  local status="pending"
  local poll_count=0

  while [ "$status" = "pending" ]; do
    sleep "$LERTY_POLL_INTERVAL"
    poll_count=$((poll_count + 1))

    # Show waiting indicator every 6 polls (30 seconds with 5s interval)
    if [ $((poll_count % 6)) -eq 0 ]; then
      echo "Lerty: Still waiting for approval... ($(( poll_count * LERTY_POLL_INTERVAL ))s)"
    fi

    local msg_response
    msg_response=$(curl -s "$LERTY_API_URL/api/v1/messages/$message_id" \
      -H "Authorization: Bearer $LERTY_API_KEY")

    status=$(echo "$msg_response" | jq -r '.metadata.interactive.status // "pending"')

    if [ "$status" = "responded" ] || [ "$status" = "callback_failed" ]; then
      local approved
      approved=$(echo "$msg_response" | jq -r '.metadata.interactive.response.approved // false')
      local message
      message=$(echo "$msg_response" | jq -r '.metadata.interactive.response.message // empty')

      # Clear pending approval from session
      [ -f "$LERTY_SESSION_FILE" ] && \
        jq 'del(.pending_approval)' "$LERTY_SESSION_FILE" > "$LERTY_SESSION_FILE.tmp" && \
        mv "$LERTY_SESSION_FILE.tmp" "$LERTY_SESSION_FILE"

      if [ "$approved" = "true" ]; then
        echo "Lerty: APPROVED"
        [ -n "$message" ] && echo "Lerty: Note: $message"
        return 0
      else
        echo "Lerty: REJECTED"
        [ -n "$message" ] && echo "Lerty: Note: $message"
        return 1
      fi
    fi
  done
}

# Check if story requires approval for a trigger
# Usage: lerty_story_requires_approval "story_json" "trigger_type"
lerty_story_requires_approval() {
  local story_json="$1"
  local trigger_type="$2"

  local story_triggers
  story_triggers=$(echo "$story_json" | jq -r '.hitl.approvalRequired // [] | .[]' 2>/dev/null)

  if echo "$story_triggers" | grep -q "^${trigger_type}$"; then
    return 0
  fi

  return 1
}

# ============================================================
# Output Parsing
# ============================================================

# Parse Lerty markers from AI output
# Markers: LERTY:APPROVAL_NEEDED:type, LERTY:NOTIFY:priority:message
lerty_parse_output() {
  local output="$1"

  # Check for approval request
  if echo "$output" | grep -q "LERTY:APPROVAL_NEEDED"; then
    echo "approval"
    return 0
  fi

  # Check for notification request
  if echo "$output" | grep -q "LERTY:NOTIFY"; then
    echo "notify"
    return 0
  fi

  echo "none"
}
