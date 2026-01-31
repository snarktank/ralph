#Requires -Version 5.1
<#
.SYNOPSIS
    Ralph Wiggum - Long-running AI agent loop (Windows PowerShell version)

.DESCRIPTION
    Ralph is an autonomous AI agent loop that runs AI coding tools (Amp or Claude Code)
    repeatedly until all PRD items are complete.

    Supports multiple argument styles for flexibility:
    - PowerShell style: -Tool claude -MaxIterations 5
    - Bash style: --tool claude 5
    - Positional: claude 5

.EXAMPLE
    .\ralph.ps1
    Runs with default settings (amp, 10 iterations)

.EXAMPLE
    .\ralph.ps1 -Tool claude -MaxIterations 5
    Runs with Claude Code for 5 iterations (PowerShell style)

.EXAMPLE
    .\ralph.ps1 --tool claude 5
    Bash-style syntax for compatibility

.EXAMPLE
    .\ralph.ps1 claude 5
    Positional arguments (tool then iterations)
#>

# Note: We use manual argument parsing to support both PowerShell-style (-Tool)
# and bash-style (--tool) arguments. PowerShell 5.1 doesn't support -- prefix.

# Initialize defaults
$Tool = 'amp'
$MaxIterations = 10

# Parse all arguments manually to support both styles
$i = 0
while ($i -lt $args.Count) {
    $arg = $args[$i]

    switch -Regex ($arg) {
        '^(-Tool|--tool)$' {
            if ($i + 1 -lt $args.Count) {
                $Tool = $args[$i + 1]
                $i++
            }
        }
        '^--tool=(.+)$' {
            $Tool = $Matches[1]
        }
        '^(-MaxIterations|--max-iterations)$' {
            if ($i + 1 -lt $args.Count) {
                $MaxIterations = [int]$args[$i + 1]
                $i++
            }
        }
        '^--max-iterations=(.+)$' {
            $MaxIterations = [int]$Matches[1]
        }
        '^\d+$' {
            # Bare number is max iterations (bash compatibility)
            $MaxIterations = [int]$arg
        }
        '^(amp|claude)$' {
            # Bare tool name (positional)
            $Tool = $arg
        }
        '^(-h|--help|-\?)$' {
            Write-Host @"
Ralph Wiggum - Long-running AI agent loop (Windows PowerShell version)

USAGE:
    .\ralph.ps1 [OPTIONS]

OPTIONS:
    -Tool, --tool <amp|claude>       AI tool to use (default: amp)
    -MaxIterations, --max-iterations Number of iterations (default: 10)
    -h, --help                       Show this help message

EXAMPLES:
    .\ralph.ps1                          # Use amp with 10 iterations
    .\ralph.ps1 -Tool claude             # Use Claude Code
    .\ralph.ps1 --tool claude 5          # Bash-style: Claude with 5 iterations
    .\ralph.ps1 claude 5                 # Positional: Claude with 5 iterations
"@
            exit 0
        }
    }
    $i++
}

# Validate tool choice
if ($Tool -notin @('amp', 'claude')) {
    Write-Error "Error: Invalid tool '$Tool'. Must be 'amp' or 'claude'."
    exit 1
}

# Check if the selected tool is available
$toolCommand = if ($Tool -eq 'amp') { 'amp' } else { 'claude' }
$toolPath = Get-Command $toolCommand -ErrorAction SilentlyContinue
if (-not $toolPath) {
    Write-Error "Error: '$toolCommand' command not found. Please ensure it is installed and in your PATH."
    Write-Host "For Amp: Visit https://ampcode.com"
    Write-Host "For Claude Code: Run 'npm install -g @anthropic-ai/claude-code'"
    exit 1
}

# Set strict mode (but we'll handle errors in the main loop with try/catch)
$ErrorActionPreference = 'Stop'

# Set UTF-8 encoding without BOM for cross-platform compatibility
$Utf8NoBom = New-Object System.Text.UTF8Encoding $false

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

        # Reset progress file for new run (UTF-8 without BOM for cross-platform compatibility)
        $progressHeader = @"
# Ralph Progress Log
Started: $(Get-Date)
---
"@
        [System.IO.File]::WriteAllText($ProgressFile, $progressHeader, $Utf8NoBom)
    }
}

# Track current branch
if (Test-Path $PrdFile) {
    $CurrentBranch = Get-JsonProperty -FilePath $PrdFile -PropertyPath 'branchName'
    if ($CurrentBranch) {
        [System.IO.File]::WriteAllText($LastBranchFile, $CurrentBranch, $Utf8NoBom)
    }
}

# Initialize progress file if it doesn't exist
if (-not (Test-Path $ProgressFile)) {
    $progressHeader = @"
# Ralph Progress Log
Started: $(Get-Date)
---
"@
    [System.IO.File]::WriteAllText($ProgressFile, $progressHeader, $Utf8NoBom)
}

# Verify prompt file exists
$PromptFile = if ($Tool -eq 'amp') { Join-Path $ScriptDir 'prompt.md' } else { Join-Path $ScriptDir 'CLAUDE.md' }
if (-not (Test-Path $PromptFile)) {
    Write-Error "Error: Prompt file not found: $PromptFile"
    Write-Host "Please ensure the prompt file exists in the same directory as ralph.ps1"
    exit 1
}

Write-Host "Starting Ralph - Tool: $Tool - Max iterations: $MaxIterations"

for ($i = 1; $i -le $MaxIterations; $i++) {
    Write-Host ""
    Write-Host "==============================================================="
    Write-Host "  Ralph Iteration $i of $MaxIterations ($Tool)"
    Write-Host "==============================================================="

    # Run the selected tool with the ralph prompt
    # Use $ErrorActionPreference = 'Continue' locally to match bash's `|| true` behavior
    $Output = ""
    $previousErrorAction = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'

    try {
        if ($Tool -eq 'amp') {
            $PromptContent = Get-Content $PromptFile -Raw -Encoding UTF8

            # Run amp and capture output while displaying it
            $Output = $PromptContent | & amp --dangerously-allow-all 2>&1 | ForEach-Object {
                Write-Host $_
                $_
            } | Out-String
        } else {
            # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
            $PromptContent = Get-Content $PromptFile -Raw -Encoding UTF8

            # Run claude and capture output while displaying it
            $Output = $PromptContent | & claude --dangerously-skip-permissions --print 2>&1 | ForEach-Object {
                Write-Host $_
                $_
            } | Out-String
        }
    } catch {
        Write-Host "Warning: Tool execution had errors: $_"
        $Output = $_.ToString()
    } finally {
        $ErrorActionPreference = $previousErrorAction
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
