# Logic Stage Tasks

Use this guidance for tasks focused on business rules, server actions, API endpoints, and backend logic.

## Before Starting

1. Read existing server actions or API routes to understand patterns
2. Check error handling patterns used elsewhere
3. Review validation logic to match existing approach

## Focus Areas

- **Business Rules** - Implement logic correctly, handle edge cases
- **Error Handling** - Use consistent error patterns
- **Validation** - Validate inputs using existing patterns
- **Types** - Ensure all inputs/outputs are properly typed

## Common Patterns to Preserve

- Error responses: Match existing error format
- Validation: Use same validation library/pattern
- Database queries: Follow existing query patterns
- Logging: Use existing logging approach (if any)

## Verification

- Logic handles all cases described in acceptance criteria
- Errors are handled gracefully
- Input validation works correctly
- Typecheck passes
- Tests pass (if tests exist)

## Notes for progress.txt

Document:
- Business rule gotchas discovered
- Error handling patterns used
- Dependencies between functions
- Performance considerations

