---
name: ralph-init
description: "Initialize Ralph in the current repository. Sets up scripts/ralph/ with all necessary files. Use when starting a new Ralph project. Triggers on: setup ralph, init ralph, initialize ralph, ralph init."
user-invocable: true
---

# Ralph Setup

Initialize the Ralph autonomous agent system in the current repository.

---

## What This Does

1. Creates `scripts/ralph/` directory in the current working directory
2. Downloads the following files from GitHub:
   - `ralph.sh` - Main execution script
   - `CLAUDE.md` - Instructions for Claude Code
3. Makes `ralph.sh` executable
4. Creates a starter `progress.txt`

---

## Instructions

Execute these commands to set up Ralph:

```bash
# Create the ralph directory
mkdir -p scripts/ralph

# Download Ralph files from GitHub
curl -fsSL https://raw.githubusercontent.com/foxytanuki/ralph/main/ralph.sh -o scripts/ralph/ralph.sh
curl -fsSL https://raw.githubusercontent.com/foxytanuki/ralph/main/CLAUDE.md -o scripts/ralph/CLAUDE.md

# Make executable
chmod +x scripts/ralph/ralph.sh

# Create initial progress file
echo "# Ralph Progress Log" > scripts/ralph/progress.txt
echo "Initialized: $(date)" >> scripts/ralph/progress.txt
echo "---" >> scripts/ralph/progress.txt
```

---

## After Setup

1. Create a PRD for your feature (use `/prd` skill)
2. Convert it to `prd.json` (use `/ralph` skill) - save to `scripts/ralph/prd.json`
3. Run Ralph: `./scripts/ralph/ralph.sh 10`

---

## Directory Structure

After running this skill, you'll have:

```
your-project/
├── scripts/
│   └── ralph/
│       ├── ralph.sh        # Run this to start Ralph
│       ├── CLAUDE.md       # Claude Code instructions
│       ├── prd.json        # Your PRD (create with /ralph skill)
│       └── progress.txt    # Progress log (auto-managed)
```

---

## Notes

- The `prd.json` file is NOT downloaded - you need to create it for your specific feature
- Use `/prd` to generate a PRD, then `/ralph` to convert it to `prd.json`
- Discord webhook notifications can be configured via `RALPH_WEBHOOK_URL` environment variable (add to `.bashrc`/`.zshrc` for global config)
