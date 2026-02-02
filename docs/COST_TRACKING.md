# Cost Tracking Guide

Running autonomous agents like Ralph can incur significant API costs. This guide helps you set budgets, track usage, and prevent runaway costs.

## Budget Recommendations

Set these budget limits before starting any autonomous session:

| Feature Size | Estimated Stories | Recommended Budget | Max Iterations |
|--------------|-------------------|-------------------|----------------|
| Small        | 1-3 stories       | $5-10             | 10             |
| Medium       | 4-8 stories       | $15-30            | 25             |
| Large        | 9-15 stories      | $40-75            | 50             |
| XL           | 16+ stories       | $100+             | 100            |

**Note:** These are estimates. Actual costs depend on story complexity, codebase size, and retry frequency.

## Feature Size to Budget Mapping

Use this guide to estimate budget before starting:

### Small Features ($5-10)
- Bug fixes with clear reproduction steps
- Adding a single new field or column
- Documentation updates
- Simple configuration changes
- 1-3 acceptance criteria per story

### Medium Features ($15-30)
- New API endpoint with tests
- Adding a new UI component
- Integration with existing service
- Refactoring a single module
- 3-5 acceptance criteria per story

### Large Features ($40-75)
- New feature spanning multiple files
- Database migration with data transformation
- Multi-step workflow implementation
- Cross-cutting concerns (auth, logging)
- 5+ acceptance criteria per story

## Claude Code Usage Tracking

Claude Code provides built-in usage tracking. Use these commands to monitor costs:

### Check Current Usage
```bash
# View usage summary for current session
claude usage

# View detailed usage breakdown
claude usage --detailed
```

### Set Budget Limits
```bash
# Set a budget limit before starting (prevents overspend)
claude config set budget_limit 25.00

# Check remaining budget
claude usage --remaining
```

### Monitor During Session
```bash
# Watch usage in real-time (run in separate terminal)
watch -n 30 'claude usage'
```

### Post-Session Analysis
```bash
# Export usage report
claude usage --export > usage-report-$(date +%Y%m%d).json

# Parse costs from report
cat usage-report-*.json | jq '.total_cost'
```

## Amp Usage Tracking

Amp (Sourcegraph's AI assistant) tracks usage through Sourcegraph's dashboard:

### Web Dashboard
1. Navigate to your Sourcegraph instance
2. Go to **Settings** → **Usage & Billing**
3. View Amp usage by time period

### CLI Tracking
```bash
# Check Amp usage via API (requires auth token)
curl -H "Authorization: token $SRC_ACCESS_TOKEN" \
  https://sourcegraph.com/.api/user/usage | jq '.'

# Filter for Amp-specific usage
curl -H "Authorization: token $SRC_ACCESS_TOKEN" \
  https://sourcegraph.com/.api/user/usage | jq '.amp'
```

### Amp Budget Controls
- Set organization-wide limits in Sourcegraph admin
- Per-user limits available in enterprise plans
- Monitor usage alerts via email or Slack integration

## Cost Prevention Strategies

### Before Starting
1. **Estimate scope** - Map features to budget sizes above
2. **Set hard limits** - Configure budget caps that stop execution
3. **Use circuit breakers** - Limit retries per story (see ralph.sh)
4. **Start small** - Run a pilot with 1-2 stories before full batch

### During Execution
1. **Monitor actively** - Watch `claude usage` during runs
2. **Check progress.txt** - Stories with many retries indicate problems
3. **Stop early** - Kill the session if costs are tracking above budget
4. **Review prd.json** - Check for stories repeatedly failing

### After Completion
1. **Export usage** - Save detailed reports for analysis
2. **Calculate cost per story** - Total cost / stories completed
3. **Adjust estimates** - Update budget recommendations based on actuals
4. **Identify expensive patterns** - Stories with many retries cost more

## Red Flags: Cost Warning Signs

Watch for these patterns that indicate escalating costs:

| Red Flag | Likely Cause | Action |
|----------|--------------|--------|
| Same story retrying 3+ times | Unclear acceptance criteria | Stop and clarify requirements |
| Many small commits | Agent thrashing on solution | Review approach |
| No progress for 5+ iterations | Blocking issue | Stop and investigate |
| Budget 50% spent, < 25% done | Scope underestimated | Re-evaluate or pause |

## Cost Tracking Checklist

Before each Ralph session:

- [ ] Estimated feature size and set budget
- [ ] Configured hard budget limit in Claude/Amp
- [ ] Set max iterations in ralph.sh
- [ ] Have monitoring terminal ready
- [ ] Know how to emergency stop

After each Ralph session:

- [ ] Exported usage report
- [ ] Calculated actual vs estimated cost
- [ ] Updated budget estimates if needed
- [ ] Documented expensive stories for future reference
