# PRD Generator for Ralph

Use this prompt template to create PRDs suitable for Ralph's autonomous execution.

## Usage

Copy this prompt and give it to Claude (in chat or CLI) along with your feature description:

---

## PRD Generator Prompt

```
I need you to help me create a Product Requirements Document (PRD) for Ralph, an autonomous coding agent.

First, ask me 3-5 clarifying questions about my feature. Format questions with lettered options like:

1. What is the primary goal?
   A. Option 1
   B. Option 2
   C. Other: [specify]

After I answer, generate a PRD in this exact JSON format:

{
  "project": "[Project Name]",
  "branchName": "ralph/[feature-name-kebab-case]",
  "description": "[Feature description]",
  "userStories": [
    {
      "id": "US-001",
      "title": "[Story title - verb phrase]",
      "description": "As a [user], I want [feature] so that [benefit]",
      "acceptanceCriteria": [
        "Specific testable criterion 1",
        "Specific testable criterion 2",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}

CRITICAL: Each story must be small enough to complete in ONE context window (roughly 15-30 minutes of focused work). If a story is too big, break it into multiple stories.

Good story sizes:
- Add a database column with migration
- Create one UI component
- Add one API endpoint
- Write tests for one module

Too big:
- "Build the auth system" (break into: add user table, add login endpoint, add session handling, add login UI, etc.)

My feature is: [DESCRIBE YOUR FEATURE HERE]
```

---

## Example Output

For "Add task priorities to my todo app":

```json
{
  "project": "TodoApp",
  "branchName": "ralph/task-priority",
  "description": "Task Priority System - Add priority levels to tasks",
  "userStories": [
    {
      "id": "US-001",
      "title": "Add priority field to database",
      "description": "As a developer, I need to store task priority so it persists across sessions.",
      "acceptanceCriteria": [
        "Add priority column to tasks table: 'high' | 'medium' | 'low' (default 'medium')",
        "Generate and run migration successfully",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    },
    {
      "id": "US-002",
      "title": "Display priority indicator on task cards",
      "description": "As a user, I want to see task priority at a glance.",
      "acceptanceCriteria": [
        "Each task card shows colored priority badge",
        "Priority visible without hovering or clicking",
        "Typecheck passes"
      ],
      "priority": 2,
      "passes": false,
      "notes": ""
    }
  ]
}
```

## Tips

1. **Start with database/backend** - Priority 1 should be data model changes
2. **Build up the stack** - Backend → API → UI → Polish
3. **Include quality checks** - Always add "Typecheck passes" or equivalent
4. **Browser verification** - For UI stories, add "Verify in browser" criteria
5. **One thing per story** - If you use "and" in the title, it might be two stories
