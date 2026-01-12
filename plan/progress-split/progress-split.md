### "progress.txt" bloat prevention (recommended adjustment)

#### Problem
Risk: Cursor may read large files; telling it "don't" isn't always enough.

#### Solution
Adopt a split:
- **`patterns.md` (or `patterns.txt`)**: small, curated, always read first
- **`progress.log`**: append-only human audit trail, usually not read by the agent

#### Implementation
Update the prompt contract:
- Always read `patterns.*`
- Only read recent tail of `progress.log` if needed (or never, unless debugging)
