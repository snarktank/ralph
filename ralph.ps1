#Requires -Version 5.1
<#
.SYNOPSIS
    Ralph Wiggum - Long-running AI agent loop (Windows PowerShell version)

.DESCRIPTION
    Ralph is an autonomous AI agent loop that runs AI coding tools (Amp or Claude Code)
    repeatedly until all PRD items are complete.

.PARAMETER Tool
    The AI tool to use: 'amp' or 'claude'. Default: amp

.PARAMETER MaxIterations
    Maximum number of iterations to run. Default: 10

.EXAMPLE
    .\ralph.ps1
    Runs with default settings (amp, 10 iterations)

.EXAMPLE
    .\ralph.ps1 -Tool claude -MaxIterations 5
    Runs with Claude Code for 5 iterations

.EXAMPLE
    .\ralph.ps1 --tool claude 5
    Alternative bash-style syntax
#>

[CmdletBinding()]
param(
    [Parameter()]
    [ValidateSet('amp', 'claude')]
    [string]$Tool = 'amp',

    [Parameter()]
    [int]$MaxIterations = 10
)

# Support bash-style arguments for compatibility
$scriptArgs = $args
for ($i = 0; $i -lt $scriptArgs.Count; $i++) {
    switch ($scriptArgs[$i]) {
        '--tool' {
            $Tool = $scriptArgs[$i + 1]
            $i++
        }
        { $_ -match '^--tool=' } {
            $Tool = $_ -replace '^--tool=', ''
        }
        { $_ -match '^\d+$' } {
            $MaxIterations = [int]$_
        }
    }
}

# Validate tool choice
if ($Tool -notin @('amp', 'claude')) {
    Write-Error "Error: Invalid tool '$Tool'. Must be 'amp' or 'claude'."
    exit 1
}

# Set strict mode
$ErrorActionPreference = 'Stop'

# Script paths
$ScriptDir = $PSScriptRoot
$PrdFile = Join-Path $ScriptDir 'prd.json'
$ProgressFile = Join-Path $ScriptDir 'progress.txt'
$ArchiveDir = Join-Path $ScriptDir 'archive'
$LastBranchFile = Join-Path $ScriptDir '.last-branch'

# Helper function to read JSON safely
function Get-JsonProperty {
    param(
        [string]$FilePath,
        [string]$PropertyPath
    )

    try {
        if (Test-Path $FilePath) {
            $json = Get-Content $FilePath -Raw | ConvertFrom-Json
            $value = $json
            foreach ($prop in $PropertyPath.Split('.')) {
                if ($null -ne $value -and $value.PSObject.Properties.Name -contains $prop) {
                    $value = $value.$prop
                } else {
                    return $null
                }
            }
            return $value
        }
    } catch {
        return $null
    }
    return $null
}

# Archive previous run if branch changed
if ((Test-Path $PrdFile) -and (Test-Path $LastBranchFile)) {
    $CurrentBranch = Get-JsonProperty -FilePath $PrdFile -PropertyPath 'branchName'
    $LastBranch = Get-Content $LastBranchFile -Raw -ErrorAction SilentlyContinue
    if ($LastBranch) { $LastBranch = $LastBranch.Trim() }

    if ($CurrentBranch -and $LastBranch -and ($CurrentBranch -ne $LastBranch)) {
        # Archive the previous run
        $Date = Get-Date -Format 'yyyy-MM-dd'
        # Strip "ralph/" prefix from branch name for folder
        $FolderName = $LastBranch -replace '^ralph/', ''
        $ArchiveFolder = Join-Path $ArchiveDir "$Date-$FolderName"

        Write-Host "Archiving previous run: $LastBranch"
        if (-not (Test-Path $ArchiveDir)) {
            New-Item -ItemType Directory -Path $ArchiveDir -Force | Out-Null
        }
        New-Item -ItemType Directory -Path $ArchiveFolder -Force | Out-Null

        if (Test-Path $PrdFile) {
            Copy-Item $PrdFile -Destination $ArchiveFolder
        }
        if (Test-Path $ProgressFile) {
            Copy-Item $ProgressFile -Destination $ArchiveFolder
        }
        Write-Host "   Archived to: $ArchiveFolder"

        # Reset progress file for new run
        $progressHeader = @"
# Ralph Progress Log
Started: $(Get-Date)
---
"@
        Set-Content -Path $ProgressFile -Value $progressHeader
    }
}

# Track current branch
if (Test-Path $PrdFile) {
    $CurrentBranch = Get-JsonProperty -FilePath $PrdFile -PropertyPath 'branchName'
    if ($CurrentBranch) {
        Set-Content -Path $LastBranchFile -Value $CurrentBranch
    }
}

# Initialize progress file if it doesn't exist
if (-not (Test-Path $ProgressFile)) {
    $progressHeader = @"
# Ralph Progress Log
Started: $(Get-Date)
---
"@
    Set-Content -Path $ProgressFile -Value $progressHeader
}

Write-Host "Starting Ralph - Tool: $Tool - Max iterations: $MaxIterations"

for ($i = 1; $i -le $MaxIterations; $i++) {
    Write-Host ""
    Write-Host "==============================================================="
    Write-Host "  Ralph Iteration $i of $MaxIterations ($Tool)"
    Write-Host "==============================================================="

    # Run the selected tool with the ralph prompt
    $Output = ""
    try {
        if ($Tool -eq 'amp') {
            $PromptFile = Join-Path $ScriptDir 'prompt.md'
            $PromptContent = Get-Content $PromptFile -Raw

            # Run amp and capture output while displaying it
            $Output = $PromptContent | & amp --dangerously-allow-all 2>&1 | ForEach-Object {
                Write-Host $_
                $_
            } | Out-String
        } else {
            # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
            $ClaudeFile = Join-Path $ScriptDir 'CLAUDE.md'

            # Run claude and capture output while displaying it
            $Output = Get-Content $ClaudeFile -Raw | & claude --dangerously-skip-permissions --print 2>&1 | ForEach-Object {
                Write-Host $_
                $_
            } | Out-String
        }
    } catch {
        Write-Host "Warning: Tool execution had errors: $_"
        $Output = $_.ToString()
    }

    # Check for completion signal
    if ($Output -match '<promise>COMPLETE</promise>') {
        Write-Host ""
        Write-Host "Ralph completed all tasks!"
        Write-Host "Completed at iteration $i of $MaxIterations"
        exit 0
    }

    Write-Host "Iteration $i complete. Continuing..."
    Start-Sleep -Seconds 2
}

Write-Host ""
Write-Host "Ralph reached max iterations ($MaxIterations) without completing all tasks."
Write-Host "Check $ProgressFile for status."
exit 1
