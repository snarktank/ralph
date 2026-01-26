#!/bin/bash
# Lerty Setup for Ralph
# Configures push notifications, Live Activities, and approval workflows

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$SCRIPT_DIR/.lerty"
CONFIG_FILE="$CONFIG_DIR/config.json"

echo "======================================"
echo "  Lerty Setup for Ralph"
echo "======================================"
echo ""
echo "This will configure:"
echo "  - Push notifications for task updates"
echo "  - Live Activities for progress tracking"
echo "  - Human-in-the-loop approval workflows"
echo ""

# Check for existing config
if [ -f "$CONFIG_FILE" ]; then
  echo "Existing configuration found."
  read -p "Overwrite? (y/N): " overwrite
  if [ "$overwrite" != "y" ] && [ "$overwrite" != "Y" ]; then
    echo "Setup cancelled."
    exit 0
  fi
fi

# Get API URL
echo ""
read -p "Lerty API URL [https://lerty.ai]: " api_url
api_url="${api_url:-https://lerty.ai}"
api_url="${api_url%/}"  # Remove trailing slash

# Get API Key
echo ""
echo "Enter your Lerty API key"
echo "(Get this from Settings > API Keys at https://lerty.ai)"
read -p "API Key: " api_key

if [ -z "$api_key" ]; then
  echo "Error: API key is required"
  exit 1
fi

# Get Agent ID
echo ""
echo "Enter the Agent ID for Ralph communications"
echo "(Approval requests will appear in this agent's conversation)"
read -p "Agent ID: " agent_id

if [ -z "$agent_id" ]; then
  echo "Error: Agent ID is required"
  exit 1
fi

# Get user email
echo ""
echo "Enter your email address (for push notifications)"
read -p "Email: " user_email

if [ -z "$user_email" ]; then
  echo "Error: Email is required"
  exit 1
fi

# Verify connection
echo ""
echo "Verifying connection to Lerty API..."

response=$(curl -s -w "\n%{http_code}" "$api_url/api/v1/agents/$agent_id" \
  -H "Authorization: Bearer $api_key" 2>/dev/null)

http_code=$(echo "$response" | tail -1)
body=$(echo "$response" | head -n -1)

if [ "$http_code" != "200" ]; then
  echo "Error: Could not connect to Lerty API"
  echo "HTTP Status: $http_code"
  echo "Response: $body"
  echo ""
  echo "Please verify:"
  echo "  - API URL is correct"
  echo "  - API key is valid"
  echo "  - Agent ID exists"
  exit 1
fi

agent_name=$(echo "$body" | jq -r '.name // "Unknown"')
echo "Connected! Agent: $agent_name"

# Create a conversation for Ralph
echo ""
echo "Creating Ralph conversation..."

conv_response=$(curl -s -X POST "$api_url/api/v1/agents/$agent_id/conversations/create" \
  -H "Authorization: Bearer $api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Ralph",
    "metadata": {"type": "ralph", "created_by": "lerty-setup"}
  }' 2>/dev/null)

conversation_id=$(echo "$conv_response" | jq -r '.conversation_id // .id // empty')

if [ -z "$conversation_id" ]; then
  echo "Warning: Could not create conversation automatically"
  echo "Response: $conv_response"
  echo ""
  read -p "Enter an existing conversation ID: " conversation_id

  if [ -z "$conversation_id" ]; then
    echo "Error: Conversation ID is required"
    exit 1
  fi
fi

echo "Conversation ID: $conversation_id"

# Create config directory and file
mkdir -p "$CONFIG_DIR"

cat > "$CONFIG_FILE" << EOF
{
  "enabled": true,
  "apiUrl": "$api_url",
  "apiKey": "$api_key",
  "conversationId": "$conversation_id",
  "agentId": "$agent_id",
  "userEmail": "$user_email",
  "pollInterval": 5
}
EOF

chmod 600 "$CONFIG_FILE"

# Test push notification
echo ""
read -p "Send a test push notification? (Y/n): " send_test

if [ "$send_test" != "n" ] && [ "$send_test" != "N" ]; then
  echo "Sending test notification..."

  test_response=$(curl -s -X POST "$api_url/api/push/send" \
    -H "Authorization: Bearer $api_key" \
    -H "Content-Type: application/json" \
    -d '{
      "email": "'"$user_email"'",
      "title": "Ralph + Lerty",
      "body": "Setup complete! You will receive notifications here.",
      "priority": "high"
    }' 2>/dev/null)

  echo "Test notification sent! Check your Lerty app."
fi

echo ""
echo "======================================"
echo "  Setup Complete!"
echo "======================================"
echo ""
echo "Configuration saved to: $CONFIG_FILE"
echo ""
echo "To enable approval workflows for a story, add to prd.json:"
echo ""
cat << 'EOF'
  {
    "id": "US-001",
    "title": "Your story",
    "hitl": {
      "approvalRequired": ["commit"]
    }
  }
EOF
echo ""
echo "Available triggers:"
echo "  - start    : Before starting work on a story"
echo "  - commit   : Before committing changes"
echo "  - stuck    : When encountering errors"
echo "  - complete : Before marking story complete"
echo ""
