# Architect - Deep Planning System for Ralph

The Architect system adds an intelligent planning phase BEFORE the Ralph execution loop. Instead of generating all tasks in one prompt, it iteratively thinks through each section of your project, creating detailed specifications with verification criteria.

## Why Architect?

The standard Ralph approach generates all tasks at once, which can lead to:
- Shallow task specifications
- Missing edge cases
- Incorrect assumptions
- Tasks that compile but don't work

Architect solves this by:
- Taking ONE section at a time
- Asking 5 critical questions for each section
- Creating detailed subtask specifications
- Identifying risks before implementation
- Mapping dependencies explicitly

## Quick Start

### 1. Set Up Your Project Idea

Copy your project description to `architect/idea.md`:

```powershell
# Your idea file should describe what you're building
# See architect/idea.md for an example
```

### 2. Run the Planning Loop

```powershell
.\architect.ps1 30
```

This will:
- Read your project idea
- Analyze each section deeply
- Generate task specifications in `architect/tasks/`
- Pause for human review every 5 sections

### 3. Review the Output

Check the generated files:
- `architect/sections/` - Deep analysis of each section
- `architect/tasks/` - JSON task specifications
- `architect/validation/status.md` - Planning progress

### 4. Generate PRD for Ralph

When planning is complete:

```powershell
.\generate-prd.ps1 "MyProject" "ralph/my-feature"
```

This converts architect output to `prd.json` for Ralph.

### 5. Run Ralph Execution Loop

```powershell
.\ralph.ps1 50
```

Now Ralph has detailed, validated tasks to execute.

## The 5 Critical Questions

For each section, Architect answers:

1. **What are we ACTUALLY building?**
   - Specific components, functions, data structures
   - Visual/functional outcomes
   - Inputs and outputs

2. **What are the EXACT steps?**
   - 15-30 minute atomic tasks
   - Files to create/modify
   - Order of operations

3. **How will we PROVE it works?**
   - Unit tests to run
   - Visual verification steps
   - Build commands
   - Expected outcomes

4. **What could go WRONG?**
   - Technical risks
   - Logic errors (e.g., Math.sqrt vs Math.ceil)
   - Integration issues
   - Edge cases

5. **What must be done FIRST?**
   - Prerequisites from other sections
   - Required state/data
   - Dependency mapping

## File Structure

```
architect/
├── prompt.md           # Instructions for the planning agent
├── idea.md             # Your project description
├── sections/           # Deep analysis files per section
│   ├── section-001-setup.md
│   ├── section-002-types.md
│   └── ...
├── tasks/              # JSON task specifications
│   ├── task-001-setup.json
│   ├── task-002-types.json
│   └── ...
├── validation/         # Status and reports
│   └── status.md
└── templates/          # Reference templates
    ├── section-template.md
    └── task-template.json
```

## Verification System

Each task includes verification criteria:

```json
{
  "verification": {
    "type": "browser-check",
    "url": "http://localhost:5173",
    "steps": ["Open page", "Click button", "Verify result"],
    "screenshotRequired": true
  }
}
```

Types:
- `build-check` - TypeScript/lint/build must pass
- `unit-test` - Specific test command
- `browser-check` - Visual verification
- `api-test` - API endpoint testing

## Human Checkpoints

The system pauses for human review:
- Every 5 sections during planning
- Before generating final PRD
- After major execution milestones (optional)

This ensures you maintain control and can adjust the plan before wasted effort.

## Tips

1. **Start with a clear idea** - The better your `idea.md`, the better the planning
2. **Review early sections carefully** - They set patterns for later sections
3. **Don't skip validation** - The 5 questions exist for a reason
4. **Iterate if needed** - Run architect again if you find issues

## Troubleshooting

**Planning stuck in a loop:**
- Check if the section is too complex
- Try breaking it into smaller sections manually
- Review `validation/status.md` for progress

**Tasks too large:**
- Each subtask should be 15-30 minutes
- If longer, it needs more breakdown

**Missing dependencies:**
- Review the dependency graph in task files
- Ensure all prerequisites are defined
