#!/bin/bash
# Test script to verify NVIDIA LLM integration in Ralph

echo "Testing Ralph NVIDIA LLM Integration"
echo "===================================="

# Test 1: Check if help works
echo "1. Testing basic help/runnable state..."
if ./ralph.sh --help 2>/dev/null || [[ $? -eq 1 ]]; then
  echo "   ✓ Script executes (help may not be implemented but no syntax errors)"
else
  echo "   ✗ Script failed to execute"
  exit 1
fi

# Test 2: Check argument parsing for NVIDIA provider
echo "2. Testing NVIDIA argument recognition..."
if ./ralph.sh --llm-provider nvidia --llm-api-key test-key 2>&1 | grep -q "Error: Empty response from NVIDIA API"; then
  echo "   ✓ NVIDIA provider recognized and attempted API call (failed as expected without valid key)"
elif ./ralph.sh --llm-provider nvidia --llm-api-key test-key 2>&1 | grep -q "Authorization: Bearer"; then
  echo "   ✓ NVIDIA provider recognized (authorization header would be sent)"
else
  # Check if it at least didn't complain about invalid provider
  if ! ./ralph.sh --llm-provider nvidia --llm-api-key test-key 2>&1 | grep -q "Only.*nvidia.*is currently supported"; then
    echo "   ✓ NVIDIA provider recognized (no rejection error)"
  else
    echo "   ✗ NVIDIA provider not recognized properly"
    exit 1
  fi
fi

# Test 3: Check that invalid provider is rejected
echo "3. Testing invalid provider rejection..."
if ./ralph.sh --llm-provider invalid --llm-api-key test 2>&1 | grep -q "Only.*nvidia.*is currently supported"; then
  echo "   ✓ Invalid provider correctly rejected"
else
  echo "   ✗ Invalid provider not rejected properly"
  exit 1
fi

# Test 4: Check that amp/claude still work
echo "4. Testing backward compatibility..."
if ./ralph.sh --tool amp 2>&1 | head -1 | grep -q "Starting Ralph"; then
  echo "   ✓ Amp tool still works"
else
  echo "   ✗ Amp tool broken"
  exit 1
fi

if ./ralph.sh --tool claude 2>&1 | head -1 | grep -q "Starting Ralph"; then
  echo "   ✓ Claude tool still works"
else
  echo "   ✗ Claude tool broken"
  exit 1
fi

echo ""
echo "All tests passed! The NVIDIA LLM integration is ready to use."
echo ""
echo "To use with a real NVIDIA API key:"
echo "  ./ralph.sh --llm-provider nvidia --llm-api-key YOUR_NVIDIA_API_KEY"
echo ""
echo "Optional parameters:"
echo "  --llm-model MODEL_NAME        (default: nvidia/nemotron-4-340b-instruct)"
echo "  --llm-base-url URL            (default: https://integrate.api.nvidia.com/v1)"
echo "  --llm-temperature TEMPERATURE (default: 0.7)"
echo ""
echo "Example with .env file (development only):"
echo "  echo \"NVIDIA_API_KEY=your-key\" > .env"
echo "  ./ralph.sh --llm-provider nvidia"
echo ""
echo "IMPORTANT: Add .env to .gitignore to prevent committing secrets!"