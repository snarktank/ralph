# ============================================================
# Generate PRD from Architect Output
# ============================================================
# This script converts the detailed task specifications from
# the Architect planning loop into a prd.json file that
# Ralph can execute.
#
# Usage: .\generate-prd.ps1 [project-name] [branch-name]
# Example: .\generate-prd.ps1 "TerraNest" "ralph/terranest-v2"
# ============================================================

param(
    [Parameter(Mandatory=$false)]
    [string]$ProjectName = "MyProject",

    [Parameter(Mandatory=$false)]
    [string]$BranchName = "ralph/main"
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "          Generate PRD from Architect Tasks                 " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# ============================================================
# Validate Input
# ============================================================

$tasksFolder = "architect/tasks"

if (-not (Test-Path $tasksFolder)) {
    Write-Host "Error: No tasks folder found at $tasksFolder" -ForegroundColor Red
    Write-Host "Run .\architect.ps1 first to generate task specifications."
    exit 1
}

$taskFiles = Get-ChildItem -Path $tasksFolder -Filter "*.json" | Sort-Object Name

if ($taskFiles.Count -eq 0) {
    Write-Host "Error: No task files found in $tasksFolder" -ForegroundColor Red
    Write-Host "Run .\architect.ps1 first to generate task specifications."
    exit 1
}

Write-Host "Found $($taskFiles.Count) task files to process..." -ForegroundColor Gray
Write-Host ""

# ============================================================
# Process Task Files
# ============================================================

$userStories = @()
$priority = 1
$errors = @()

foreach ($file in $taskFiles) {
    Write-Host "Processing: $($file.Name)" -ForegroundColor Gray

    try {
        $taskContent = Get-Content $file.FullName -Raw | ConvertFrom-Json

        # Validate task structure
        if (-not $taskContent.subtasks) {
            $errors += "Warning: $($file.Name) has no subtasks"
            continue
        }

        # Process each subtask
        foreach ($subtask in $taskContent.subtasks) {

            # Build acceptance criteria from verification
            $criteria = @()

            if ($subtask.verification) {
                # Add expected outcome
                if ($subtask.verification.expected) {
                    $criteria += $subtask.verification.expected
                }

                # Add specific checks
                if ($subtask.verification.specificChecks) {
                    $criteria += $subtask.verification.specificChecks
                }

                # Add verification steps for browser checks
                if ($subtask.verification.type -eq "browser-check" -and $subtask.verification.steps) {
                    $criteria += "Visual verification: $($subtask.verification.steps -join ' -> ')"
                }
            }

            # Always add typecheck
            if ($criteria -notcontains "Typecheck passes") {
                $criteria += "Typecheck passes"
            }

            # Create user story
            $story = @{
                id = "US-$('{0:D3}' -f $priority)"
                title = $subtask.description
                description = "Section: $($taskContent.section) | Task: $($taskContent.title)"
                acceptanceCriteria = $criteria
                priority = $priority
                passes = $false
                notes = ""
                verification = $subtask.verification
                files = $subtask.files
                timeEstimate = $subtask.timeEstimate
            }

            $userStories += $story
            $priority++
        }

    } catch {
        $errors += "Error processing $($file.Name): $_"
    }
}

# ============================================================
# Report Errors
# ============================================================

if ($errors.Count -gt 0) {
    Write-Host ""
    Write-Host "Warnings/Errors encountered:" -ForegroundColor Yellow
    foreach ($error in $errors) {
        Write-Host "  - $error" -ForegroundColor Yellow
    }
    Write-Host ""
}

# ============================================================
# Generate PRD JSON
# ============================================================

# Try to extract project info from idea.md
$description = "Generated from Architect planning loop"
if (Test-Path "architect/idea.md") {
    $ideaContent = Get-Content "architect/idea.md" -Raw -ErrorAction SilentlyContinue
    if ($ideaContent -and $ideaContent.Length -gt 100) {
        $description = $ideaContent.Substring(0, [Math]::Min(200, $ideaContent.Length)) -replace "`n", " " -replace "`r", ""
        $description = $description.Trim() + "..."
    }
}

$prd = @{
    project = $ProjectName
    branchName = $BranchName
    description = $description
    generatedAt = (Get-Date -Format "yyyy-MM-ddTHH:mm:ss")
    generatedBy = "architect-loop"
    totalTasks = $userStories.Count
    userStories = $userStories
}

# ============================================================
# Write Output
# ============================================================

$outputPath = "prd.json"
$prd | ConvertTo-Json -Depth 10 | Out-File -FilePath $outputPath -Encoding utf8

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "          PRD Generated Successfully!                       " -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Output file: $outputPath" -ForegroundColor White
Write-Host "  Project: $ProjectName" -ForegroundColor White
Write-Host "  Branch: $BranchName" -ForegroundColor White
Write-Host "  Total tasks: $($userStories.Count)" -ForegroundColor White
Write-Host ""
Write-Host "  Task breakdown:" -ForegroundColor Gray

# Show task summary
$tasksBySection = $userStories | Group-Object { $_.description.Split('|')[0].Trim() }
foreach ($section in $tasksBySection) {
    Write-Host "    - $($section.Name): $($section.Count) tasks" -ForegroundColor Gray
}

Write-Host ""
Write-Host "  Next steps:" -ForegroundColor Yellow
Write-Host "    1. Review prd.json" -ForegroundColor White
Write-Host "    2. Initialize git branch: git checkout -b $BranchName" -ForegroundColor Cyan
Write-Host "    3. Run Ralph: .\ralph.ps1 $($userStories.Count + 10)" -ForegroundColor Cyan
Write-Host ""
Write-Host "============================================================" -ForegroundColor Green

# ============================================================
# Generate Summary Report
# ============================================================

$summaryPath = "architect/validation/prd-summary.md"
@"
# PRD Generation Summary

**Generated:** $(Get-Date -Format "yyyy-MM-dd HH:mm")
**Project:** $ProjectName
**Branch:** $BranchName
**Total Tasks:** $($userStories.Count)

## Tasks by Section

$($tasksBySection | ForEach-Object { "- **$($_.Name.Trim())**: $($_.Count) tasks" } | Out-String)

## Task List

$($userStories | ForEach-Object { "- [ ] $($_.id): $($_.title)" } | Out-String)

## Verification Types

$($userStories | Where-Object { $_.verification } | Group-Object { $_.verification.type } | ForEach-Object { "- **$($_.Name)**: $($_.Count) tasks" } | Out-String)

## Notes

Ready for execution with Ralph loop.
"@ | Out-File -FilePath $summaryPath -Encoding utf8

Write-Host "Summary saved to: $summaryPath" -ForegroundColor Gray
Write-Host ""
