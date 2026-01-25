# {{PROJECT_NAME}} - WordPress SSH Project

## Server Access
```bash
ssh -p 722 {{USER}}@niobium.cloudhosting.uk
```

## Paths
- Web root: /home/{{USER}}/public_html
- Plugins: /home/{{USER}}/public_html/wp-content/plugins
- Themes: /home/{{USER}}/public_html/wp-content/themes
- Logs: /home/{{USER}}/logs/error.log

## Common Commands

### Check site status
```bash
# Homepage loads
curl -sI 'https://{{DOMAIN}}/' | head -5

# Check PHP errors
ssh -p 722 {{USER}}@niobium.cloudhosting.uk "tail -100 /home/{{USER}}/logs/error.log"

# WordPress CLI
ssh -p 722 {{USER}}@niobium.cloudhosting.uk "cd /home/{{USER}}/public_html && wp option list --search='{{SEARCH}}'"
```

### File operations
```bash
# List plugins
ssh -p 722 {{USER}}@niobium.cloudhosting.uk "ls -la /home/{{USER}}/public_html/wp-content/plugins/"

# Check permissions
ssh -p 722 {{USER}}@niobium.cloudhosting.uk "ls -la /home/{{USER}}/public_html/wp-content/plugins/{{PLUGIN}}/"

# Fix permissions
ssh -p 722 {{USER}}@niobium.cloudhosting.uk "find /home/{{USER}}/public_html/wp-content/plugins/{{PLUGIN}} -type f -exec chmod 644 {} \; && find /home/{{USER}}/public_html/wp-content/plugins/{{PLUGIN}} -type d -exec chmod 755 {} \;"
```

## Quality Checks

Before marking any story as `passes: true`:
1. Site loads without errors
2. No PHP errors in logs related to changes
3. All assets return 200 status
4. Functionality tested and working

## Progress Report Format

APPEND to progress.txt:
```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- Commands used
- **Learnings:**
  - Patterns discovered
  - Gotchas encountered
---
```

## Stop Condition

If ALL stories in prd.json have `passes: true`:
<promise>COMPLETE</promise>

Otherwise end normally for next iteration.

## Important

- ONE story per iteration
- ALWAYS backup before changes: `cp file.php file.php.bak`
- Document everything in progress.txt
- Use WP-CLI where possible
