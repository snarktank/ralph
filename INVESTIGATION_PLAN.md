# Investigation Plan: Integrating Alternative LLM Providers with Ralph

## Objective
Investigate whether and how Ralph can be configured to use API keys from alternative LLM providers (other than those used by Amp or Claude Code).

## Current Architecture Review

### Ralph's Tool Invocation Mechanism
From `ralph.sh`:
- Supports only two tools: `amp` (default) and `claude`
- Amp invocation: `amp --dangerously-allow-all`
- Claude Code invocation: `claude --dangerously-skip-permissions --print`
- Tool selection is hardcoded with validation (lines 32-35)

### Amp Investigation
Based on webfetch results, Amp is:
- A coding agent from AmpCode
- Appears to have its own model/backend (references to "GPT-5.4, The New Oracle")
- Marketed as having deep agentic capabilities
- Unknown if it supports plugging in external API keys

## Investigation Steps

### 1. Amp Capabilities Research
**Goal:** Determine if Amp supports external LLM providers via API keys or configuration

**Methods:**
- Review Amp's documentation for LLM configuration options
- Check if Amp uses environment variables for API keys
- Investigate Amp's model selection capabilities
- Determine if Amp has an "open" or "bring-your-own-model" mode

### 2. Ralph Extension Possibilities
**Goal:** Identify how Ralph could be modified to support alternative LLM providers

**Approaches to investigate:**
A. **Through Amp configuration** (if Amp supports external providers)
   - Modify Ralph to set appropriate environment variables/flags for Amp
   - Leverage Amp's existing multi-provider support

B. **Direct Ralph modification** 
   - Add new tool options to ralph.sh (e.g., --tool openai, --tool ollama)
   - Create wrapper scripts that translate Ralph's tool invocation to target LLM APIs
   - Modify the core loop to invoke different LLM APIs directly

C. **Adapter pattern**
   - Create a universal adapter script that mimics Amp/Claude Code interfaces
   - Route calls through the adapter to various LLM providers
   - Keep Ralph unchanged, only change the adapter

### 3. Requirements Analysis
**What would be needed for each approach:**

For Amp-based approach:
- Amp must support external API keys
- Ralph may need to set environment variables before invoking Amp
- Potential need to pass model selection parameters

For direct Ralph modification:
- Understanding of each target LLM provider's API format
- Implementation of completion/chat APIs equivalent to what Amp/Claude Code provide
- Handling of tool use capabilities (since Ralph expects agent-like behavior)
- Authentication mechanisms for each provider

### 4. Risk and Tradeoff Assessment
**Consider each approach:**

**Amp Extension (if possible):**
- Pros: Minimal changes to Ralph, leverages existing Amp capabilities
- Cons: Depends on Amp's capabilities, less control over provider choice

**Direct Ralph Modification:**
- Pros: Full control over provider integration
- Cons: Significant development effort, need to replicate agent capabilities
- Risk: May not achieve same level of autonomy as Amp/Claude Code

**Adapter Pattern:**
- Pros: Zero changes to Ralph, flexible provider switching
- Cons: Complexity of maintaining adapter, potential latency/faithfulness issues

## Recommendation Framework

### Decision Criteria
1. **Feasibility:** Can it actually be done with reasonable effort?
2. **Compatibility:** Will it work with Ralph's expectations (agent behavior, tool use, etc.)?
3. **Maintenance:** How much ongoing work will be required?
4. **Ralph Principles:** Does it align with Ralph's design (simple, reliable, autonomous)?

### Suggested Investigation Path
1. First investigate Amp's capabilities for external providers
2. If Amp supports it, recommend that approach
3. If not, investigate creating a minimal adapter for a popular provider (e.g., OpenAI compatible API)
4. Prototype with a simple proof-of-concept
5. Document limitations and required Ralph modifications

## Next Steps for Investigation

Since I cannot execute commands or modify non-markdown files in PLAN mode, the user would need to:

1. **Research Amp's documentation** for:
   - Environment variables (AMP_API_KEY, etc.)
   - Model configuration options
   - Alternative provider support in Amp's settings
   - Whether Amp has a "proxy" or "custom endpoint" mode

2. **Experiment with Amp directly** to see:
   - What happens when you set different API keys
   - If Amp accepts OPENAI_API_KEY or similar variables
   - Whether Amp can be pointed at custom endpoints

3. **Check Ralph's current tool invocation** to understand what capabilities it expects from the tool:
   - Does it need streaming responses?
   - Does it need specific exit codes?
   - What input format does it provide (prompt via stdin)?
   - What output format does it expect?

## Potential Target Providers to Investigate
- OpenAI API (most common, likely easiest)
- Ollama (for local models)
- Other providers with OpenAI-compatible APIs
- Anthropic API (if wanting different Claude models)

## Conclusion
The investigation should start with Amp's capabilities since modifying Ralph to work through Amp would require the least changes to the Ralph system itself. If Amp doesn't support external providers, then we would need to consider modifying Ralph's tool invocation mechanism.

The key insight is that Ralph doesn't care which LLM is used - it only cares that the invoked tool behaves like an autonomous coding agent capable of:
- Reading prompt.md or CLAUDE.md from stdin
- Editing files based on instructions
- Running commands
- Producing output that Ralph can parse for completion signals

Any LLM provider that can be wrapped to provide this interface could potentially work with Ralph.