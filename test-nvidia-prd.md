# Test NVIDIA Integration

As a developer, I want to verify that Ralph can be invoked with NVIDIA LLM provider arguments.

## Acceptance Criteria
- Ralph accepts --llm-provider nvidia argument
- Ralph accepts --llm-api-key argument
- Ralph shows appropriate error when API key is missing
- Ralph works with existing --tool amp and --tool claude arguments