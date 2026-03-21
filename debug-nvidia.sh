#!/bin/bash
# Debug script to test NVIDIA API connection

echo "Testing NVIDIA API Connection"
echo "============================"

# Load .env if exists
if [ -f ".env" ]; then
  source .env
fi

if [ -z "$NVIDIA_API_KEY" ]; then
  echo "ERROR: NVIDIA_API_KEY not set"
  echo "Please set it in .env file or export NVIDIA_API_KEY"
  exit 1
fi

echo "API Key found: ${NVIDIA_API_KEY:0:10}...${NVIDIA_API_KEY: -10}"

# Test 1: Basic connectivity with minimal payload
echo ""
echo "1. Testing basic connectivity..."
RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "https://integrate.api.nvidia.com/v1/chat/completions" \
  -H "Authorization: Bearer $NVIDIA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"nvidia/nemotron-4-340b-instruct","messages":[{"role":"user","content":"test"}],"max_tokens":5}' 2>/dev/null) || echo "CURL FAILED"

HTTP_STATUS=$(echo "$RESPONSE" | sed -n 's/.*HTTP_STATUS://p')
RESPONSE_BODY=$(echo "$RESPONSE" | sed -n '/HTTP_STATUS:/!p')

echo "HTTP Status: $HTTP_STATUS"
echo "Response: $RESPONSE_BODY"

# Test 2: Try different model formats
echo ""
echo "2. Trying different model identifiers..."
MODELS_TO_TRY=(
  "nvidia/nemotron-4-340b-instruct"
  "nemotron-4-340b-instruct" 
  "mistralai/mixtral-8x22b-instruct"
  "mixtral-8x22b-instruct"
)

for model in "${MODELS_TO_TRY[@]}"; do
  echo "Trying model: $model"
  RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "https://integrate.api.nvidia.com/v1/chat/completions" \
    -H "Authorization: Bearer $NVIDIA_API_KEY" \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"$model\",\"messages\":[{\"role\":\"user\",\"content\":\"say hello\"}],\"max_tokens\":10}" 2>/dev/null) || echo "CURL FAILED"
  
  HTTP_STATUS=$(echo "$RESPONSE" | sed -n 's/.*HTTP_STATUS://p')
  RESPONSE_BODY=$(echo "$RESPONSE" | sed -n '/HTTP_STATUS:/!p')
  
  echo "  HTTP Status: $HTTP_STATUS"
  if [[ "$HTTP_STATUS" == "200" ]]; then
    echo "  ✓ SUCCESS with model: $model"
    echo "  Response preview: $(echo "$RESPONSE_BODY" | head -c 100)"
    break
  else
    echo "  ✗ FAILED: $RESPONSE_BODY"
  fi
done

# Test 3: Check if we need a different base URL
echo ""
echo "3. Checking alternative endpoints..."
ALT_URLS=(
  "https://api.nvidia.com/v1/chat/completions"
  "https://integrate.api.nvidia.com/v1"
  "https://health.api.nvidia.com/v1/chat/completions"
)

for url in "${ALT_URLS[@]}"; do
  echo "Trying URL: $url"
  RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$url" \
    -H "Authorization: Bearer $NVIDIA_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"model":"nvidia/nemotron-4-340b-instruct","messages":[{"role":"user","content":"test"}],"max_tokens":5}' 2>/dev/null) || echo "CURL FAILED"
  
  HTTP_STATUS=$(echo "$RESPONSE" | sed -n 's/.*HTTP_STATUS://p')
  RESPONSE_BODY=$(echo "$RESPONSE" | sed -n '/HTTP_STATUS:/!p')
  
  echo "  HTTP Status: $HTTP_STATUS"
  if [[ "$HTTP_STATUS" == "200" ]]; then
    echo "  ✓ SUCCESS with URL: $url"
    break
  else
    echo "  ✗ FAILED: $(echo "$RESPONSE_BODY" | head -c 50)"
  fi
done

echo ""
echo "Debug script completed."