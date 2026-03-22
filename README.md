# Ralph 🐛

**A self-improving autonomous AI coding agent in bash.**

![Ralph](ralph.webp)

Ralph reads a PRD (Product Requirements Document), implements tasks autonomously, verifies results through TDD, learns from mistakes, and commits to git — all in a loop, without human intervention.

Built on the [Ralph Loop](https://www.aihero.dev/getting-started-with-ralph) pattern popularized by Geoffrey Huntley.

---

## ✨ Key Features

| Feature            | Ralph                                            | Typical AI Agents         |
| ------------------ | ------------------------------------------------ | ------------------------- |
| **Runtime**        | Pure bash (~1000 lines)                          | Python/Node.js frameworks |
| **LLM Provider**   | NVIDIA NIM (free tier)                           | OpenAI/Anthropic (paid)   |
| **Self-Improving** | Agent modifies its own code                      | Static scaffolding        |
| **TDD Built-in**   | `verificationCommand` per story                  | Manual testing            |
| **Memory**         | 4 channels (git, progress, learnings, knowledge) | Context window only       |
| **Error Recovery** | 4-level (format → logic → stuck → catastrophic)  | Retry and fail            |
| **Cost**           | $0 (free tier API)                               | $5-50+ per session        |

---

## 🚀 Quick Start

### Prerequisites

- **WSL2** (Ubuntu 22.04+) or Linux
- `jq`, `curl`, `git`, `python3`
- NVIDIA NIM API key ([get free key](https://build.nvidia.com/))

### 1. Clone & Setup

```bash
git clone https://github.com/snarktank/ralph.git
cd ralph
chmod +x ralph-v3.sh

# Install dependencies (Ubuntu/WSL)
sudo apt install jq curl python3 -y
```

### 2. Configure API Key

```bash
cat > .env << 'EOF'
NVIDIA_API_KEY=nvapi-your-key-here
LLM_MODEL=moonshotai/kimi-k2-instruct
LLM_BASE_URL=https://integrate.api.nvidia.com/v1
LLM_TEMPERATURE=0.3
EOF
```

### 3. Create Your PRD

```bash
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
```

### 4. Run

```bash
./ralph-v3.sh --llm-provider nvidia --max-iterations 6
```

Ralph will automatically:

1. 📖 Read the PRD and find the first unfinished story
2. 📋 Plan its approach
3. 📝 Create/edit files to implement the story
4. 🏃 Run commands to verify
5. ✅ Execute verificationCommand (TDD gate)
6. 📦 Git commit the result
7. ➡️ Move to the next story
8. 🎉 Exit when all stories pass

---

## 📊 Project Structure

```
ralph/
├── ralph-v3.sh          # Main agent script (~1000 lines of bash)
├── ralph-v2.sh          # Previous version with more features
├── .env                 # API credentials (never committed)
├── prd.json             # Product Requirements Document (input)
├── progress.txt          # Progress log (auto-generated)
├── learnings.md          # Lessons learned (persists across runs)
├── AGENTS.md            # Knowledge base (persists across runs)
├── STEERING.md          # Human override file (optional)
├── ralph-debug.log      # Timestamped action log
├── flowchart/            # Interactive visualization
├── archive/             # Archived completed runs
└── .staging/            # Sandbox for self-modification
```

---

## 🧠 Architecture

```
┌─────────────────────────────────────────────────────┐
│ Outer Loop (max_iterations)                         │
│  ┌───────────────────────────────────────────────┐  │
│  │ Inner Agent Loop (max 40 iterations)          │  │
│  │                                               │  │
│  │  ┌─────────┐  ┌──────────┐  ┌─────────────┐   │  │
│  │  │ Prompt  │→│ LLM Call │→│ Execute       │   │  │
│  │  │ Builder │  │ (NVIDIA) │  │ Actions     │   │  │
│  │  └─────────┘  └──────────┘  └─────────────┘   │  │
│  │       ↑                           │           │  │
│  │       └───── conversation ────────┘           │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  Memory: [git] [progress.txt] [learnings.md]        │
│          [AGENTS.md] [STEERING.md]                  │
└─────────────────────────────────────────────────────┘
```

### How It Works

1. Ralph picks the **highest priority** story where `passes: false`
2. Spawns a **fresh AI instance** with clean context
3. The AI implements the story, runs checks, commits
4. Marks story as `passes: true`, appends learnings to `progress.txt`
5. Repeats until all stories pass or max iterations reached
6. Outputs `<promise>COMPLETE</promise>` when done

### Agent Actions

| Action        | Description                                            |
| ------------- | ------------------------------------------------------ |
| `plan`        | Records a todo list (context engineering trick)        |
| `read_file`   | Reads file with smart chunking (>200 lines auto-split) |
| `list_files`  | Lists directory contents                               |
| `edit_file`   | Find-and-replace edit                                  |
| `create_file` | Creates new file with content                          |
| `run_command` | Executes shell command (isolated via `bash -c`)        |
| `finish`      | Triggers verification + marks story as passed          |

---

## ✅ TDD Verification

Each story can have a `verificationCommand` — a shell command that validates the implementation:

```json
{
  "id": "FEAT-001",
  "title": "Create greeting script",
  "verificationCommand": "bash greet.sh World | grep -qxF 'Hello, World!'",
  "passes": false
}
```

| Result                    | Action                                 |
| ------------------------- | -------------------------------------- |
| ✅ Command exits 0        | Story marked as PASSED                 |
| ❌ Command exits non-zero | Story REJECTED, agent gets full output |
| 🚨 5 rejections           | Critical hint injected                 |
| No `verificationCommand`  | Agent is trusted                       |

### Tips for Good PRDs

- Keep stories **small** — one task per story
- Write **specific** `verificationCommand` — this is your TDD gate
- Use `grep -qxF` for exact match verification
- Chain multiple checks with `&&`

---

## 🛡️ Safety Features

### Sandbox for Self-Modification

Ralph can modify its own code. When it edits `.sh` files:

1. Changes go to `.staging/` copy
2. `bash -n` syntax validation runs
3. ✅ Valid → applied to live code
4. ❌ Invalid → rejected, agent gets error feedback

### Protected Files

| File            | Protection                            |
| --------------- | ------------------------------------- |
| `.env`, `.git/` | Hard blocked (cannot edit)            |
| `prd.json`      | Warning (use `finish` action instead) |

### Error Recovery (4 Levels)

| Level           | Trigger         | Response                             |
| --------------- | --------------- | ------------------------------------ |
| 1. Format       | Invalid JSON ×3 | Recovery prompt with example         |
| 2. Logic        | Tests fail      | Full test output as feedback         |
| 3. Stuck        | Same error ×3   | Self-reflection prompt injected      |
| 4. Catastrophic | API failure ×3  | Fallback model + exponential backoff |

### Guardrails

- Max 40 inner iterations per story
- Sliding context window (10 messages, auto-shrink at 80%)
- Command isolation (`bash -c` instead of `eval`)
- Checkpoint file for crash recovery

---

## 🔄 Memory System (4 Channels)

Ralph maintains memory across iterations through files, not model context:

| Channel     | File           | Purpose                        | Persistence |
| ----------- | -------------- | ------------------------------ | ----------- |
| Git History | `.git/`        | Code changes, diffs            | Permanent   |
| Progress    | `progress.txt` | What was done each iteration   | Per run     |
| Learnings   | `learnings.md` | What worked, what failed       | Across runs |
| Knowledge   | `AGENTS.md`    | Patterns, gotchas, conventions | Across runs |

### Live Human Override

Create `STEERING.md` while Ralph is running:

```bash
echo "PRIORITY: Fix the bug in auth.sh before continuing" > STEERING.md
```

Ralph reads this file every iteration and prioritizes its content over normal tasks.

---

## ⚙️ Configuration

### CLI Arguments

```bash
./ralph-v3.sh \
  --llm-provider nvidia \                           # Required for NVIDIA NIM
  --max-iterations 10 \                             # Outer loop limit (default: 10)
  --llm-model moonshotai/kimi-k2-instruct \         # Override model
  --llm-temperature 0.3 \                          # LLM temperature
  --debug                                           # Enable verbose debug output
```

### Environment Variables

```bash
NVIDIA_API_KEY=nvapi-...                             # Required
LLM_MODEL=moonshotai/kimi-k2-instruct              # Default model
LLM_BASE_URL=https://integrate.api.nvidia.com/v1
LLM_TEMPERATURE=0.3
MAX_CONTEXT_TOKENS=200000                           # Context window limit
```

### Tested Models (NVIDIA NIM Free Tier)

| Model                                      | Context | Notes                             |
| ------------------------------------------ | ------- | --------------------------------- |
| `moonshotai/kimi-k2-instruct`              | 256K    | ✅ Recommended — clean JSON, fast |
| `qwen/qwen2.5-coder-32b-instruct`          | 32K     | Good coding, small context        |
| `meta/llama-3.1-70b-instruct`              | 128K    | Wraps JSON in markdown            |
| `abacusai/dracarys-llama-3.1-70b-instruct` | 128K    | Fallback model                    |
| `microsoft/phi-3-medium-128k-instruct`     | 128K    | Fallback model                    |

---

## 🏗️ Self-Improvement History

Ralph has improved itself across multiple iterations:

| Version | Stories          | Key Additions                                   |
| ------- | ---------------- | ----------------------------------------------- |
| v1.0    | SI-001 to SI-004 | Logging, list_files, auto git commit            |
| v1.5    | SI-005 to SI-009 | Status display, timer, COMPLETE detection       |
| v2.0    | Full rewrite     | 4-channel memory, error recovery, plan action   |
| v3.0    | Refactor         | Cleaner architecture, better temp file handling |

### What Ralph Built for Itself

- `log_action()` — timestamped debug logging
- `list_files` action type
- Auto git commit before `.sh` edits
- Smart file chunking for large files
- Structured error messages

---

## 🔬 Benchmarks

| Test                            | Stories | Pass Rate | Time | LLM Calls |
| ------------------------------- | ------- | --------- | ---- | --------- |
| Basic (greet + word count)      | 2/2     | 100%      | 36s  | 9         |
| TDD (parse_stats + file_report) | 2/2     | 100%      | ~60s | ~12       |
| Stress (fizzbuzz + csv + tree)  | 3/3     | 100%      | 78s  | 14        |

---

## 🧪 Development

### Running Tests

```bash
# Syntax check
bash -n ralph-v3.sh

# Quick validation (2 simple stories)
cp prd.json.example prd.json
./ralph-v3.sh --llm-provider nvidia --max-iterations 6 --debug

# Check results
jq '.userStories[] | {id, passes}' prd.json
cat learnings.md
```

### Debug Mode

```bash
# Full debug output
./ralph-v3.sh --llm-provider nvidia --debug 2>&1 | tee debug.log

# Check debug log
tail -20 ralph-debug.log

# Check last LLM response
cat /tmp/llm_response_debug.txt
```

---

## 📚 Research & References

Ralph's architecture is informed by:

- **ICLR 2025** — [A Self-Improving Coding Agent](https://arxiv.org/abs/2501.12527) (+17-53% on SWE-bench)
- **ICLR 2025** — [EvoMAC](https://arxiv.org/abs/2501.09632) — objective verification > LLM-as-judge
- **Princeton** — [mini-swe-agent](https://github.com/PrincetonML/SWE-agent) — 100 lines, >74% SWE-bench
- **LangChain** — [Harness Engineering](https://blog.langchain.dev/improving-langgraph-with-harnesses/) — +13.7 points
- **Addy Osmani** — [Self-Improving Agents](https://addyosmani.com/blog/self-improving-agents/)
- **Geoffrey Huntley** — [Ralph Loop](https://ghuntley.com/ralph/)

### Key Design Decisions

| Decision                | Why                                             |
| ----------------------- | ----------------------------------------------- |
| Pure bash (no Python)   | Minimal dependencies, runs anywhere             |
| `bash -c` over `eval`   | Subprocess isolation prevents state leaks       |
| `plan` as no-op action  | Forces LLM to articulate strategy               |
| Tests over LLM-as-judge | Objective verification eliminates hallucination |
| 4 memory channels       | Minimum for stable cross-iteration learning     |
| File-based state        | Survives crashes, restarts, model switches      |

---

## 🗺️ Roadmap

```
v1.0 ─── Basic agent loop with NVIDIA NIM
  │
v1.5 ─── Self-improvement (agent modifies its own code)
  │
v2.0 ─── Memory, error recovery, harness engineering
  │
v2.1 ─── Prompt self-evolution (GEPA pattern)
  │
v2.2 ─── Multi-agent (Coder + Reviewer + Tester)
  │
v3.0 ─── Cleaner architecture, optimized for production
```

---

## 📄 License

MIT

---

## 🙏 Acknowledgments

- **Geoffrey Huntley** — for the Ralph Loop concept
- **NVIDIA** — for the free NIM API tier
- **Moonshot AI** — for Kimi K2, the model that makes this work
- **The open-source AI agent community**

---

Ralph is named after Ralph Wiggum from The Simpsons — persistent, cheerful, and surprisingly effective.
