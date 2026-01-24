# Foundation Stage Tasks

Use this guidance for tasks focused on schema, migrations, data models, and core types.

## Before Starting

1. Read existing schema files to understand naming conventions
2. Check migration history to see the pattern used
3. Review type definitions to match existing patterns

## Focus Areas

- **Consistency** - Follow existing naming conventions exactly
- **Types** - Ensure types are properly exported and reusable
- **Migrations** - Use the same migration pattern as existing ones
- **Constraints** - Add appropriate foreign keys, indexes, defaults

## Common Patterns to Preserve

- Migration file naming: `YYYYMMDDHHMMSS-description.sql`
- Type exports: Export types from schema files for reuse
- Default values: Match existing patterns (e.g., timestamps, enums)
- Nullability: Follow existing patterns for optional vs required fields

## Verification

- Migration runs without errors
- Types compile correctly
- No breaking changes to existing queries (unless intentional)
- Typecheck passes

## Notes for progress.txt

Document:
- Any new patterns established
- Naming conventions discovered
- Gotchas about the schema (e.g., "field X must be set before Y")

