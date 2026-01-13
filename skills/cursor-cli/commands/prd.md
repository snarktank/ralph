# PRD Command

Generate a Product Requirements Document (PRD) for a new feature.

## Usage

Use `/prd` or say "create a PRD for [feature]" to start the PRD generation workflow.

## What This Does

1. Asks 3-5 clarifying questions with lettered options
2. Generates a structured PRD based on your answers
3. Saves to `tasks/prd-[feature-name].md`

## Example

```
/prd create a PRD for adding task priority levels
```

Or use natural language:
```
create a PRD for adding task priority levels
```

## Next Steps

After creating the PRD, use `/ralph` to convert it to `prd.json` format and begin autonomous execution.

## Full Documentation

See `.cursor/rules/prd.md` for complete PRD generation guidelines and examples.
