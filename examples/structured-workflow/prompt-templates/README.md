# Prompt Templates

Optional prompt snippets you can reference when customizing `prompt.md` for different task types.

**These are suggestions, not requirements.** Ralph works fine with the default `prompt.md`. Use these if you want to add task-type-specific guidance.

## How to Use

1. Read the template that matches your task's `stage` or `focus` (if you're using those metadata fields)
2. Copy relevant sections into your `prompt.md`
3. Adapt to your project's conventions

Or ignore these entirely and stick with the default prompt.

## Templates

- **`foundation.md`** - For schema, migrations, data model tasks
- **`logic.md`** - For business rules, server actions, API endpoints
- **`interface.md`** - For UI components, forms, displays
- **`integration.md`** - For connecting pieces, data fetching, event handling
- **`polish.md`** - For error handling, edge cases, refinement

## Philosophy

These templates emphasize:
- **Clear scope** - What exactly needs to be done
- **Verification** - How to know it's complete
- **Context** - What the agent should read first
- **Patterns** - What to preserve from existing code

They don't change Ralph's execution model - they just help the agent understand the task better.

