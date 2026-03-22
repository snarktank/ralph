# Ralph 🐛

**A self-improving autonomous AI coding agent in bash.**

Ralph reads a PRD, implements tasks, verifies results through TDD, learns from mistakes,
and commits to git — all in a loop, without human intervention.

Built on the [Ralph Loop](https://www.aihero.dev/getting-started-with-ralph) pattern
popularized by Geoffrey Huntley: _"Ralph is a bash loop."_
PRD → Agent reads story → Plans → Implements → Verifies → Commits → Next story → Done

text

---

## ✨ What Makes Ralph Different

| Feature            | Ralph                                                 | Typical AI Agents         |
| ------------------ | ----------------------------------------------------- | ------------------------- |
| **Runtime**        | Pure bash (~500 lines)                                | Python/Node.js frameworks |
| **LLM Provider**   | NVIDIA NIM (free tier)                                | OpenAI/Anthropic (paid)   |
| **Self-Improving** | Agent modifies its own code                           | Static scaffolding        |
| **TDD Built-in**   | `verificationCommand` per story                       | Manual testing            |
| **Memory**         | 4 channels (git, progress, learnings, knowledge base) | Context window only       |
| **Error Recovery** | 4-level (format → logic → stuck → catastrophic)       | Retry and fail            |
| **Cost**           | $0 (free tier API)                                    | $5-50+ per session        |

---

## 🚀 Quick Start

### Prerequisites

- **WSL2** (Ubuntu 22.04+) or Linux
- `jq`, `curl`, `git`, `python3`
- NVIDIA NIM API key ([free tier](https://build.nvidia.com/))

### Setup

```bash
git clone https://github.com/yourusername/ralph.git
cd ralph
chmod +x ralph-v3.sh

# Install dependencies (Ubuntu/WSL)
sudo apt install jq curl python3 -y

# Create .env with your API key
cat > .env << 'EOF'
NVIDIA_API_KEY=nvapi-your-key-here
LLM_MODEL=moonshotai/kimi-k2-instruct
LLM_BASE_URL=https://integrate.api.nvidia.com/v1
LLM_TEMPERATURE=0.3
EOF
Create a PRD
Bash

cat > prd.json << 'EOF'
{
  "branchName": "ralph/my-feature",
  "userStories": [
    {
      "id": "FEAT-001",
      "title": "Create greeting script",
      "description": "Create greet.sh that takes a name and prints 'Hello, NAME!'",
      "acceptanceCriteria": ["bash greet.sh World outputs 'Hello, World!'"],
      "verificationCommand": "bash greet.sh World | grep -qxF 'Hello, World!'",
      "passes": false
    }
  ]
}
EOF
Run
Bash

./ralph-v3.sh --llm-provider nvidia --max-iterations 6
Ralph will:

📖 Read the PRD and find the first unfinished story
📋 Plan its approach
📝 Create/edit files to implement the story
🏃 Run commands to verify
✅ Execute verificationCommand (TDD gate)
📦 Git commit the result
➡️ Move to the next story
🎉 Exit when all stories pass
📁 Project Structure
text

ralph/
├── ralph-v3.sh          # Main agent script (~520 lines of bash)
├── .env                 # API credentials (never committed)
├── prd.json             # Product Requirements Document (input)
├── progress.txt         # Progress log (auto-generated)
├── learnings.md         # Lessons learned (persists across runs)
├── AGENTS.md            # Knowledge base (persists across runs)
├── STEERING.md          # Human override file (optional, create during run)
├── ralph-debug.log      # Timestamped action log
├── archive/             # Archived completed runs
└── .staging/            # Sandbox for self-modification
🧠 Architecture
text

┌─────────────────────────────────────────────────────┐
│ Outer Loop (max_iterations)                         │
│  ┌───────────────────────────────────────────────┐  │
│  │ Inner Agent Loop (max 40 iterations)          │  │
│  │                                                │  │
│  │  ┌─────────┐  ┌──────────┐  ┌─────────────┐  │  │
│  │  │ Prompt  │→│ LLM Call │→│ Execute     │  │  │
│  │  │ Builder │  │ (NVIDIA) │  │ Actions     │  │  │
│  │  └─────────┘  └──────────┘  └─────────────┘  │  │
│  │       ↑                           │           │  │
│  │       └───── conversation ────────┘           │  │
│  │              history                          │  │
│  └───────────────────────────────────────────────┘  │
│                                                      │
│  Memory: [git] [progress.txt] [learnings.md]        │
│          [AGENTS.md] [STEERING.md]                   │
└─────────────────────────────────────────────────────┘
Agent Actions
Action	Description
plan	Records a todo list (context engineering trick)
read_file	Reads file with smart chunking (>200 lines auto-split)
list_files	Lists directory contents
edit_file	Find-and-replace edit
create_file	Creates new file with content
run_command	Executes shell command (isolated via bash -c)
finish	Triggers verification + marks story as passed
TDD Verification
Each story can have a verificationCommand:

JSON

{
  "id": "FEAT-001",
  "verificationCommand": "bash my_script.sh | grep -qF 'expected output'",
  "passes": false
}
✅ Command exits 0 → story marked as PASSED
❌ Command exits non-zero → story REJECTED, agent gets feedback with full output
🚨 After 5 rejections → critical hint injected
No verificationCommand → agent is trusted (default behavior)
🛡️ Safety Features
Sandbox for Self-Modification
Ralph can modify its own code. When it edits .sh files:

Changes go to .staging/ copy
bash -n syntax validation runs
✅ Valid → applied to live code
❌ Invalid → rejected, agent gets error feedback
Protected Files
.env, .git/ — hard blocked (cannot edit)
prd.json — warning (use finish action instead)
Error Recovery (4 Levels)
Level	Trigger	Response
1. Format	Invalid JSON ×3	Recovery prompt with example
2. Logic	Tests fail	Full test output as feedback
3. Stuck	Same error ×3	Self-reflection prompt injected
4. Catastrophic	API failure ×3	Fallback model + exponential backoff
Guardrails
Max 40 inner iterations per story
Sliding context window (10 messages, auto-shrink at 80%)
Command isolation (bash -c instead of eval)
Checkpoint file for crash recovery
🔄 Memory System (4 Channels)
Ralph maintains memory across iterations through files, not model context:

Channel	File	Purpose	Persistence
Git History	.git/	Code changes, diffs	Permanent
Progress	progress.txt	What was done each iteration	Per run
Learnings	learnings.md	What worked, what failed	Across runs
Knowledge	AGENTS.md	Patterns, gotchas, conventions	Across runs
Live Human Override
Create STEERING.md while Ralph is running:

Bash

echo "PRIORITY: Fix the bug in auth.sh before continuing with other tasks" > STEERING.md
Ralph reads this file every iteration and prioritizes its content over normal tasks.

⚙️ Configuration
CLI Arguments
Bash

./ralph-v3.sh \
  --llm-provider nvidia \       # Required for NVIDIA NIM
  --max-iterations 10 \         # Outer loop limit (default: 10)
  --llm-model moonshotai/kimi-k2-instruct \  # Override model
  --llm-temperature 0.3 \       # LLM temperature
  --debug                       # Enable verbose debug output
Environment Variables (.env)
Bash

NVIDIA_API_KEY=nvapi-...        # Required
LLM_MODEL=moonshotai/kimi-k2-instruct  # Default model
LLM_BASE_URL=https://integrate.api.nvidia.com/v1
LLM_TEMPERATURE=0.3
MAX_CONTEXT_TOKENS=200000       # Context window limit
Tested Models (NVIDIA NIM Free Tier)
Model	Context	Notes
moonshotai/kimi-k2-instruct	256K	✅ Recommended — clean JSON, fast
qwen/qwen2.5-coder-32b-instruct	32K	Good coding, small context
meta/llama-3.1-70b-instruct	128K	Wraps JSON in markdown
abacusai/dracarys-llama-3.1-70b-instruct	128K	Fallback model
microsoft/phi-3-medium-128k-instruct	128K	Fallback model
📊 PRD Format
JSON

{
  "branchName": "ralph/feature-name",
  "userStories": [
    {
      "id": "FEAT-001",
      "title": "Short title",
      "description": "Detailed description of what to build",
      "acceptanceCriteria": [
        "Criterion 1",
        "Criterion 2"
      ],
      "verificationCommand": "bash test.sh && echo pass",
      "passes": false
    }
  ]
}
Tips for good PRDs:

Keep stories small — one task per story
Write specific verificationCommand — this is your TDD gate
Use grep -qxF for exact match verification
Chain multiple checks with &&
🏗️ Self-Improvement History
Ralph has improved itself across multiple iterations:

Version	Stories Completed	Key Additions
v1.0	SI-001 to SI-004	Logging, list_files, auto git commit, error reporting
v1.5	SI-005 to SI-009	Status display, timer, COMPLETE detection, chunking, TDD
v2.0	Full rewrite	4-channel memory, error recovery, plan action, guardrails
What Ralph Built for Itself
log_action() — timestamped debug logging
list_files action type
Auto git commit before .sh edits
Structured error messages
Smart file chunking for large files
🔬 Benchmarks
Tested on internal task suites:

Test	Stories	Pass Rate	Time	LLM Calls
Basic (greet + word count)	2/2	100%	36s	9
TDD (parse_stats + file_report)	2/2	100%	~60s	~12
Stress (fizzbuzz + csv + tree)	TBD	TBD	TBD	TBD
🧪 Development
Running Tests
Bash

# Syntax check
bash -n ralph-v3.sh

# Quick validation (2 simple stories)
cp prd.json.example prd.json
./ralph-v3.sh --llm-provider nvidia --max-iterations 6 --debug

# Check results
jq '.userStories[] | {id, passes}' prd.json
cat learnings.md
Debug Mode
Bash

# Full debug output
./ralph-v3.sh --llm-provider nvidia --debug 2>&1 | tee debug.log

# Check debug log
tail -20 ralph-debug.log

# Check last LLM response
cat /tmp/llm_response_debug.txt
📚 Research & References
Ralph's architecture is informed by:

A Self-Improving Coding Agent (ICLR 2025) — scaffolding self-improvement yields +17-53% on SWE-bench
EvoMAC (ICLR 2025) — objective verification (tests) > LLM-as-judge
mini-swe-agent (Princeton) — 100 lines, >74% SWE-bench; subprocess isolation
Deep Agents (LangChain) — write_todos no-op trick, 4 pillars of deep agents
LangChain Harness Engineering — +13.7 points from harness optimization alone
Addy Osmani: Self-Improving Agents — 4 memory channels, Ralph Loop patterns
Awesome Ralph — ecosystem of Ralph Loop implementations
Key Design Decisions
Decision	Why
Pure bash (no Python framework)	Minimal dependencies, runs anywhere
bash -c over eval	Subprocess isolation prevents state leaks
plan as no-op action	Forces LLM to articulate strategy (context engineering)
Tests over LLM-as-judge	Objective verification eliminates hallucination bias
4 memory channels	Minimum for stable cross-iteration learning
File-based state	Survives crashes, restarts, model switches
🗺️ Roadmap
 v1.0 — Basic agent loop with NVIDIA NIM
 v1.5 — Self-improvement (agent modifies its own code)
 v2.0 — Memory, error recovery, harness engineering
 v2.1 — Prompt self-evolution (GEPA pattern)
 v2.2 — Multi-agent (Coder + Reviewer + Tester)
 v3.0 — LangGraph migration for advanced orchestration
📄 License
MIT

🙏 Acknowledgments
Geoffrey Huntley — for the Ralph Loop concept
NVIDIA — for the free NIM API tier
Moonshot AI — for Kimi K2, the model that makes this work
The open-source AI agent community
Ralph is named after Ralph Wiggum from The Simpsons — persistent, cheerful, and surprisingly effective.

text


---
```
