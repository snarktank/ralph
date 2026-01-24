# Integration Stage Tasks

Use this guidance for tasks focused on connecting pieces, data fetching, event handling, and end-to-end flows.

## Before Starting

1. Read how data fetching is done elsewhere in the codebase
2. Check existing event handling patterns
3. Review how different layers connect (UI → API → Database)
4. Understand state management approach

## Focus Areas

- **Data Flow** - Ensure data flows correctly through layers
- **State Synchronization** - Keep UI and backend in sync
- **Error Propagation** - Errors surface correctly to users
- **Performance** - Avoid unnecessary re-renders or requests

## Common Patterns to Preserve

- Data fetching: Use existing fetch/cache patterns
- State updates: Follow existing state update patterns
- Optimistic updates: Match existing optimistic update approach
- Error handling: Use existing error handling in UI
- Loading states: Match existing loading patterns

## Verification

- Data appears correctly in UI
- Updates persist correctly
- Errors are handled and displayed
- Loading states work properly
- No unnecessary network requests
- Typecheck passes
- **Verify in browser using dev-browser skill** (required for UI integration)

## Notes for progress.txt

Document:
- Data flow patterns discovered
- Gotchas about state synchronization
- Performance considerations
- Dependencies between layers

