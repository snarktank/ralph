# Ralph Scripts

This folder contains Ralph execution scripts managed by the Ralph installation process.

**⚠️ DO NOT MANUALLY EDIT** these files. They are managed by:
- Installation: `ralph_install.sh`
- Version tracking: `.ralph-version`

## Updating Ralph

To update Ralph to the latest version:

1. Navigate to your Ralph repository:
   ```bash
   cd /path/to/ralph
   ```

2. Pull latest changes:
   ```bash
   git pull
   ```

3. Re-run install script from your project root:
   ```bash
   cd /path/to/your/project
   ./ralph_install.sh /path/to/ralph
   ```

Or if `ralph_install.sh` is already in your project:
```bash
./ralph_install.sh
```

## Files in this Directory

| File | Purpose | Editable? |
|------|---------|-----------|
| `ralph.sh` | Main Ralph execution script | ❌ No - managed by install |
| `prompt.md` | Agent instructions for each iteration | ❌ No - managed by install |
| `.ralph-version` | Installed version marker (git commit hash) | ❌ No - auto-generated |
| `.gitattributes` | Git merge protection settings | ❌ No - managed by install |
| `README.md` | This file | ✅ Yes - can customize |

## Version Information

The installed version is tracked in `.ralph-version`. This helps identify which version of Ralph you're using and whether updates are available.

## Customization

If you need to customize Ralph behavior:
- **Don't edit** `ralph.sh` or `prompt.md` directly
- Consider using environment variables or configuration files
- Or fork the Ralph repository and modify the source files there

## Troubleshooting

**Q: Can I commit these files to git?**
A: Yes! These files are meant to be committed. The `.gitattributes` file helps prevent merge conflicts when updating.

**Q: The files look outdated, how do I update?**
A: Re-run `ralph_install.sh` from your project root with the path to your updated Ralph repository.

**Q: What if I accidentally edited ralph.sh?**
A: Re-run the install script to restore the original files. Your customizations will be lost, so make sure you have them backed up if needed.
