# Ralph Execution Loop
# Usage: .\ralph.ps1 [max_iterations]
# Example: .\ralph.ps1 98

param([int]$MaxIterations = 50)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Info { param($msg) Write-Host $msg -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host $msg -ForegroundColor Green }
function Write-Warning { param($msg) Write-Host $msg -ForegroundColor Yellow }
function Write-Step { param($msg) Write-Host $msg -ForegroundColor Blue }

# Create necessary folders
$folders = @("screenshots", "logs")
foreach ($folder in $folders) {
    if (-not (Test-Path $folder)) {
        New-Item -ItemType Directory -Path $folder -Force | Out-Null
    }
}

# Create progress.txt if it doesn't exist
if (-not (Test-Path "progress.txt")) {
    @"
# Ralph Progress Log

## Codebase Patterns
(Add reusable patterns here as you discover them)

---

"@ | Out-File -FilePath "progress.txt" -Encoding utf8
    Write-Info "Created progress.txt"
}

Write-Host ""
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "   Ralph Execution Loop" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "Max iterations: $MaxIterations" -ForegroundColor Yellow
Write-Host ""

$startTime = Get-Date
$completedStories = 0

for ($i = 1; $i -le $MaxIterations; $i++) {
    Write-Step "`n--- Execution Iteration $i of $MaxIterations ---"
    $iterationStart = Get-Date

    # Read the prompt
    if (-not (Test-Path "prompt.md")) {
        Write-Warning "ERROR: prompt.md not found!"
        exit 1
    }
    $prompt = Get-Content -Path "prompt.md" -Raw

    # Run Claude with permission bypass
    $claudeArgs = @(
        "-p", $prompt,
        "--output-format", "text",
        "--dangerously-skip-permissions",
        "--allowedTools", "Write,Edit,Read,Bash(npm:*),Bash(npx:*),Bash(node:*),Bash(git:*),Bash(vercel:*),Bash(agent-browser:*),Bash(mkdir:*),Bash(ls:*),Bash(cat:*),Bash(echo:*),Bash(curl:*),Bash(tail:*)"
    )

    Write-Info "Running Claude..."
    $result = & claude @claudeArgs 2>&1 | Out-String

    # Display output
    Write-Host $result

    # Log iteration to file
    $logEntry = @"

=== Iteration $i - $(Get-Date -Format "yyyy-MM-dd HH:mm:ss") ===
$result
"@
    Add-Content -Path "logs/ralph-execution.log" -Value $logEntry

    # Check for completion signal
    if ($result -match "<promise>COMPLETE</promise>") {
        $elapsed = (Get-Date) - $startTime
        Write-Host ""
        Write-Success "====================================="
        Write-Success "   ALL STORIES COMPLETE!"
        Write-Success "====================================="
        Write-Host ""
        Write-Info "Total iterations: $i"
        Write-Info "Total time: $($elapsed.ToString('hh\:mm\:ss'))"
        Write-Host ""
        Write-Info "Next steps:"
        Write-Host "  1. Review progress.txt for learnings"
        Write-Host "  2. Deploy to Vercel: vercel --prod"
        Write-Host "  3. Check screenshots/ for visual verification"
        exit 0
    }

    # Check for story completion
    if ($result -match "passes.*true|feat:|Committed|story.*complete") {
        $completedStories++
        Write-Success "Story completed! Total: $completedStories"
    }

    # Check for errors
    if ($result -match "error|Error|ERROR|failed|Failed|FAILED") {
        Write-Warning "Potential issue detected in iteration $i - check logs/ralph-execution.log"
    }

    $iterationDuration = (Get-Date) - $iterationStart
    Write-Info "Iteration $i completed in $($iterationDuration.ToString('mm\:ss'))"

    # Brief pause between iterations
    Start-Sleep -Seconds 2
}

$elapsed = (Get-Date) - $startTime
Write-Host ""
Write-Warning "====================================="
Write-Warning "   MAX ITERATIONS REACHED"
Write-Warning "====================================="
Write-Host ""
Write-Info "Completed iterations: $MaxIterations"
Write-Info "Stories completed this run: $completedStories"
Write-Info "Total time: $($elapsed.ToString('hh\:mm\:ss'))"
Write-Host ""
Write-Warning "Check prd.json for remaining stories with passes: false"
Write-Warning "Run again with: .\ralph.ps1 $MaxIterations"
