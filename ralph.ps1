# Ralph Wiggum - Long-running AI agent loop (PowerShell)
# Usage: .\ralph.ps1 [max_iterations]

param(
    [int]$MaxIterations = 10
)

$ErrorActionPreference = "Stop"

# Check for required commands
if (-not (Get-Command agent -ErrorAction SilentlyContinue)) {
    Write-Host "Error: 'agent' command not found. Please install Cursor CLI: https://cursor.com/docs/cli" -ForegroundColor Red
    exit 1
}

if (-not (Get-Command jq -ErrorAction SilentlyContinue)) {
    Write-Host "Error: 'jq' command not found. Please install jq: choco install jq" -ForegroundColor Red
    exit 1
}

# Find project root (where prd.json should be located)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = $ScriptDir

for ($i = 0; $i -le 3; $i++) {
    if (Test-Path "$ProjectRoot\prd.json") {
        break
    }
    $ParentDir = Split-Path -Parent $ProjectRoot
    if ($ParentDir -eq $ProjectRoot) {
        # Reached root, fallback to script directory
        $ProjectRoot = $ScriptDir
        break
    }
    $ProjectRoot = $ParentDir
}

$PrdFile = Join-Path $ProjectRoot "prd.json"
$ProgressFile = Join-Path $ProjectRoot "progress.txt"
$ArchiveDir = Join-Path $ProjectRoot "archive"
$LastBranchFile = Join-Path $ProjectRoot ".last-branch"
$PromptFile = Join-Path $ScriptDir "prompt.md"

# Archive previous run if branch changed
if ((Test-Path $PrdFile) -and (Test-Path $LastBranchFile)) {
    try {
        $CurrentBranch = (& jq -r '.branchName // empty' $PrdFile 2>$null) -replace "`r`n|`n", ""
        $LastBranch = (Get-Content $LastBranchFile -Raw -ErrorAction SilentlyContinue) -replace "`r`n|`n", ""
        
        if ($CurrentBranch -and $LastBranch -and $CurrentBranch -ne $LastBranch) {
            $Date = Get-Date -Format "yyyy-MM-dd"
            $FolderName = $LastBranch -replace "^ralph/", ""
            $ArchiveFolder = Join-Path $ArchiveDir "$Date-$FolderName"
            
            Write-Host "Archiving previous run: $LastBranch"
            New-Item -ItemType Directory -Force -Path $ArchiveFolder | Out-Null
            if (Test-Path $PrdFile) { Copy-Item $PrdFile $ArchiveFolder\ }
            if (Test-Path $ProgressFile) { Copy-Item $ProgressFile $ArchiveFolder\ }
            Write-Host "   Archived to: $ArchiveFolder"
            
            # Reset progress file for new run
            @"
# Ralph Progress Log
Started: $(Get-Date)
---
"@ | Set-Content $ProgressFile -Encoding UTF8
        }
    } catch {
        Write-Host "Warning: Could not check branch for archiving: $_" -ForegroundColor Yellow
    }
}

# Track current branch
if (Test-Path $PrdFile) {
    try {
        $CurrentBranch = (& jq -r '.branchName // empty' $PrdFile 2>$null) -replace "`r`n|`n", ""
        if ($CurrentBranch) {
            $CurrentBranch | Set-Content $LastBranchFile -Encoding UTF8
        }
    } catch {
        # Ignore errors
    }
}

# Initialize progress file if it doesn't exist
if (-not (Test-Path $ProgressFile)) {
    @"
# Ralph Progress Log
Started: $(Get-Date)
---
"@ | Set-Content $ProgressFile -Encoding UTF8
}

Write-Host "Starting Ralph - Max iterations: $MaxIterations"

$ConsecutiveErrors = 0
$MaxRetries = 3
$RetryDelay = 10
$Iteration = 1

while ($Iteration -le $MaxIterations) {
    Write-Host ""
    Write-Host "======================================================="
    Write-Host "  Ralph Iteration $Iteration of $MaxIterations"
    Write-Host "======================================================="
    
    # Read prompt file
    if (-not (Test-Path $PromptFile)) {
        Write-Host "Error: prompt.md not found at $PromptFile" -ForegroundColor Red
        exit 1
    }
    $Prompt = Get-Content $PromptFile -Raw -Encoding UTF8
    
    # Run Cursor CLI agent with the ralph prompt
    # --print flag is required for non-interactive mode and enables shell execution
    # --force flag forces allow commands unless explicitly denied
    # --workspace sets the working directory (where prd.json is located)
    $Output = ""
    try {
        $Output = & agent --print --force --workspace $ProjectRoot --output-format text $Prompt 2>&1 | Tee-Object -Variable CapturedOutput
        $Output = $CapturedOutput -join "`n"
    } catch {
        $Output = $_.Exception.Message
    }
    
    # Check for connection errors - these mean the iteration didn't actually run
    if ($Output -match "ConnectError|ETIMEDOUT|ECONNRESET|ENOTFOUND") {
        $ConsecutiveErrors++
        Write-Host ""
        Write-Host "Warning: Connection error detected ($ConsecutiveErrors consecutive)" -ForegroundColor Yellow
        
        if ($ConsecutiveErrors -ge $MaxRetries) {
            Write-Host "Error: Too many consecutive connection errors. Stopping." -ForegroundColor Red
            Write-Host "   Check your network connection and Cursor CLI status."
            exit 1
        }
        
        # Exponential backoff: 10s, 20s, 30s...
        $WaitTime = $RetryDelay * $ConsecutiveErrors
        Write-Host "   Waiting ${WaitTime}s before retry..."
        Start-Sleep -Seconds $WaitTime
        
        # Don't increment iteration - retry this one
        continue
    }
    
    # Reset error counter on successful connection
    $ConsecutiveErrors = 0
    
    # Check for completion signal
    if ($Output -match "<promise>COMPLETE</promise>") {
        Write-Host ""
        Write-Host "Ralph completed all tasks!" -ForegroundColor Green
        Write-Host "Completed at iteration $Iteration of $MaxIterations"
        exit 0
    }
    
    Write-Host "Iteration $Iteration complete. Continuing..."
    $Iteration++
    Start-Sleep -Seconds 2
}

Write-Host ""
Write-Host "Ralph reached max iterations ($MaxIterations) without completing all tasks."
Write-Host "Check $ProgressFile for status."
exit 1
