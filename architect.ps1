# ============================================================
# Architect Deep Planning Loop
# ============================================================
# This script runs Claude Code in a planning loop, deeply thinking
# through each section of a project before any code is written.
#
# Unlike the execution loop (ralph.ps1), this loop:
# - Focuses on PLANNING, not implementation
# - Creates detailed task specifications
# - Validates dependencies and feasibility
# - Pauses for human review at checkpoints
#
# Usage: .\architect.ps1 [max_iterations]
# Example: .\architect.ps1 30
# ============================================================

param(
    [Parameter(Mandatory=$false)]
    [int]$MaxIterations = 30
)

$ErrorActionPreference = "Stop"

# ============================================================
# Setup: Ensure required folders and files exist
# ============================================================

$requiredFolders = @(
    "architect/sections",
    "architect/tasks",
    "architect/validation",
    "architect/templates"
)

foreach ($folder in $requiredFolders) {
    if (-not (Test-Path $folder)) {
        New-Item -ItemType Directory -Path $folder -Force | Out-Null
        Write-Host "Created folder: $folder" -ForegroundColor Gray
    }
}

# Check for required files
if (-not (Test-Path "architect/prompt.md")) {
    Write-Host "Error: architect/prompt.md not found" -ForegroundColor Red
    Write-Host "Please create the architect prompt file first."
    exit 1
}

if (-not (Test-Path "architect/idea.md")) {
    Write-Host "Error: architect/idea.md not found" -ForegroundColor Red
    Write-Host "Please copy your project idea to architect/idea.md"
    Write-Host ""
    Write-Host "Example:"
    Write-Host "  Copy-Item 'path/to/myidea.md' 'architect/idea.md'"
    exit 1
}

# Initialize status file if not exists
if (-not (Test-Path "architect/validation/status.md")) {
    @"
# Planning Status

## Overview
- **Started:** $(Get-Date -Format "yyyy-MM-dd HH:mm")
- **Project:** (pending)
- **Status:** In Progress

## Completed Sections
(none yet)

## In Progress
(starting...)

## Pending
(to be discovered)

## Validation Checks
- [ ] All sections analyzed
- [ ] All dependencies resolved
- [ ] No circular dependencies
- [ ] All tasks have verification
- [ ] Time estimates reasonable
- [ ] Ready for execution

## Notes
Planning loop started.
"@ | Out-File -FilePath "architect/validation/status.md" -Encoding utf8
}

# ============================================================
# Display Header
# ============================================================

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "          ARCHITECT - Deep Planning Loop                    " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Purpose: Think deeply about each section before coding    " -ForegroundColor White
Write-Host "  Output:  Detailed task specifications in architect/tasks/ " -ForegroundColor White
Write-Host ""
Write-Host "  Max iterations: " -NoNewline
Write-Host "$MaxIterations" -ForegroundColor Green
Write-Host "  Section complete signal: " -NoNewline
Write-Host "<section>COMPLETE</section>" -ForegroundColor Yellow
Write-Host "  Planning complete signal: " -NoNewline
Write-Host "<architect>READY_FOR_REVIEW</architect>" -ForegroundColor Green
Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Starting in 3 seconds... Press Ctrl+C to abort" -ForegroundColor Yellow
Start-Sleep -Seconds 3
Write-Host ""

# ============================================================
# Main Planning Loop
# ============================================================

$sectionsCompleted = 0
$startTime = Get-Date

for ($i = 1; $i -le $MaxIterations; $i++) {

    # Iteration Header
    Write-Host ""
    Write-Host "------------------------------------------------------------" -ForegroundColor Blue
    Write-Host "  Planning Iteration $i of $MaxIterations" -ForegroundColor Blue
    Write-Host "  Sections completed so far: $sectionsCompleted" -ForegroundColor Gray
    Write-Host "------------------------------------------------------------" -ForegroundColor Blue
    Write-Host ""

    # Read the architect prompt
    $prompt = Get-Content -Path "architect/prompt.md" -Raw

    # Run Claude with the prompt
    # Using --dangerously-skip-permissions with explicit allowed tools
    # The settings.json has allowDangerouslySkipPermissions: true
    try {
        $claudeArgs = @(
            "-p", $prompt,
            "--output-format", "text",
            "--dangerously-skip-permissions",
            "--allowedTools", "Write,Edit,Read,Bash(mkdir:*),Bash(ls:*)"
        )

        $result = & claude @claudeArgs 2>&1 | Out-String
        Write-Host $result
    }
    catch {
        Write-Host "Error running Claude: $_" -ForegroundColor Red
        $result = ""
    }

    Write-Host ""

    # ========================================================
    # Check for Planning Complete Signal
    # ========================================================
    if ($result -match "<architect>READY_FOR_REVIEW</architect>") {
        $endTime = Get-Date
        $duration = $endTime - $startTime

        Write-Host ""
        Write-Host "============================================================" -ForegroundColor Green
        Write-Host "          PLANNING COMPLETE!                                " -ForegroundColor Green
        Write-Host "============================================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "  Iterations used: $i" -ForegroundColor White
        Write-Host "  Sections planned: $sectionsCompleted" -ForegroundColor White
        Write-Host "  Time elapsed: $($duration.ToString('hh\:mm\:ss'))" -ForegroundColor White
        Write-Host ""
        Write-Host "  Review your plan:" -ForegroundColor Yellow
        Write-Host "    - Sections:   architect/sections/" -ForegroundColor Gray
        Write-Host "    - Tasks:      architect/tasks/" -ForegroundColor Gray
        Write-Host "    - Validation: architect/validation/status.md" -ForegroundColor Gray
        Write-Host ""
        Write-Host "  Next steps:" -ForegroundColor Yellow
        Write-Host "    1. Review the generated sections and tasks" -ForegroundColor White
        Write-Host "    2. Make any adjustments needed" -ForegroundColor White
        Write-Host "    3. Run: .\generate-prd.ps1" -ForegroundColor Cyan
        Write-Host "    4. Then run: .\ralph.ps1 50" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "============================================================" -ForegroundColor Green
        exit 0
    }

    # ========================================================
    # Check for Section Complete Signal
    # ========================================================
    if ($result -match "<section>COMPLETE</section>") {
        $sectionsCompleted++
        Write-Host ""
        Write-Host "  [OK] Section $sectionsCompleted completed!" -ForegroundColor Cyan
        Write-Host "       Continuing to next section..." -ForegroundColor Gray
        Write-Host ""

        # Brief pause between sections
        Start-Sleep -Seconds 2
    }

    # ========================================================
    # Checkpoint: Pause for human review every 5 sections
    # ========================================================
    if ($sectionsCompleted -gt 0 -and $sectionsCompleted % 5 -eq 0) {
        Write-Host ""
        Write-Host "============================================================" -ForegroundColor Yellow
        Write-Host "          CHECKPOINT: $sectionsCompleted sections completed " -ForegroundColor Yellow
        Write-Host "============================================================" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "  Take a moment to review progress in architect/tasks/" -ForegroundColor White
        Write-Host ""
        Write-Host "  Press Enter to continue, or Ctrl+C to stop and review..." -ForegroundColor Yellow
        Read-Host
    }

    # Small delay between iterations
    Start-Sleep -Seconds 2
}

# ============================================================
# Max Iterations Reached
# ============================================================

$endTime = Get-Date
$duration = $endTime - $startTime

Write-Host ""
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host "          MAX ITERATIONS REACHED                            " -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Iterations used: $MaxIterations" -ForegroundColor White
Write-Host "  Sections completed: $sectionsCompleted" -ForegroundColor White
Write-Host "  Time elapsed: $($duration.ToString('hh\:mm\:ss'))" -ForegroundColor White
Write-Host ""
Write-Host "  The planning loop reached its limit." -ForegroundColor Gray
Write-Host "  This could mean:" -ForegroundColor Gray
Write-Host "    - The project is very large (try more iterations)" -ForegroundColor Gray
Write-Host "    - Planning is stuck on a complex section" -ForegroundColor Gray
Write-Host ""
Write-Host "  Options:" -ForegroundColor Yellow
Write-Host "    1. Review progress: architect/validation/status.md" -ForegroundColor White
Write-Host "    2. Run more iterations: .\architect.ps1 50" -ForegroundColor Cyan
Write-Host "    3. Continue with partial plan: .\generate-prd.ps1" -ForegroundColor Cyan
Write-Host ""
exit 1
