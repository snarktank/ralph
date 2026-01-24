# Polish Stage Tasks

Use this guidance for tasks focused on error handling, edge cases, refinement, and final touches.

## Before Starting

1. Review the feature as implemented so far
2. Check existing error handling patterns
3. Look for similar edge case handling elsewhere
4. Review user feedback or requirements for edge cases

## Focus Areas

- **Edge Cases** - Handle boundary conditions and unusual inputs
- **Error States** - Provide clear error messages and recovery paths
- **User Experience** - Ensure smooth experience even when things go wrong
- **Performance** - Optimize for large datasets or slow networks

## Common Patterns to Preserve

- Error messages: Match existing error message style
- Empty states: Use existing empty state components
- Loading states: Match existing loading patterns
- Validation: Use existing validation error display
- Retry logic: Follow existing retry patterns (if any)

## Verification

- All edge cases from acceptance criteria are handled
- Error states are user-friendly
- Performance is acceptable (no obvious slowdowns)
- No regressions in existing functionality
- Typecheck passes
- **Verify in browser using dev-browser skill** (required for UI polish)

## Notes for progress.txt

Document:
- Edge cases discovered during implementation
- Error handling patterns used
- Performance optimizations made
- Gotchas about edge case handling

