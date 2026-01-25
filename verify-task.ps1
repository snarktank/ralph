# ============================================================
# Verify Task Completion
# ============================================================
# Runs all verification checks for a completed task and
# generates a debug report.
#
# Usage: .\verify-task.ps1 [task-id]
# Example: .\verify-task.ps1 US-005
# ============================================================

param(
    [Parameter(Mandatory=$false)]
    [string]$TaskId = "current",

    [Parameter(Mandatory=$false)]
    [switch]$DeployPreview,

    [Parameter(Mandatory=$false)]
    [switch]$TakeScreenshot
)

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "          Task Verification System                          " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Task: $TaskId" -ForegroundColor White
Write-Host "  Time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Gray
Write-Host ""

# ============================================================
# Create output directory
# ============================================================

$reportDir = "verification-reports"
if (-not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
}

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$reportFile = "$reportDir/verify-$TaskId-$timestamp.md"

# Initialize report
$report = @"
# Verification Report: $TaskId

**Generated:** $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
**Status:** In Progress

---

"@

# ============================================================
# 1. TypeScript Check
# ============================================================

Write-Host "1. Running TypeScript check..." -ForegroundColor Yellow

$typecheckStart = Get-Date
$typecheckResult = & npm run typecheck 2>&1 | Out-String
$typecheckEnd = Get-Date
$typecheckDuration = ($typecheckEnd - $typecheckStart).TotalSeconds

if ($LASTEXITCODE -eq 0) {
    Write-Host "   [PASS] TypeScript check passed ($([math]::Round($typecheckDuration, 1))s)" -ForegroundColor Green
    $typecheckStatus = "PASS"
} else {
    Write-Host "   [FAIL] TypeScript errors found" -ForegroundColor Red
    $typecheckStatus = "FAIL"
}

$report += @"
## 1. TypeScript Check

**Status:** $typecheckStatus
**Duration:** $([math]::Round($typecheckDuration, 1))s

``````
$typecheckResult
``````

---

"@

# ============================================================
# 2. Lint Check
# ============================================================

Write-Host "2. Running lint check..." -ForegroundColor Yellow

$lintStart = Get-Date
$lintResult = & npm run lint 2>&1 | Out-String
$lintEnd = Get-Date
$lintDuration = ($lintEnd - $lintStart).TotalSeconds

if ($LASTEXITCODE -eq 0) {
    Write-Host "   [PASS] Lint check passed ($([math]::Round($lintDuration, 1))s)" -ForegroundColor Green
    $lintStatus = "PASS"
} else {
    Write-Host "   [FAIL] Lint errors found" -ForegroundColor Red
    $lintStatus = "FAIL"
}

$report += @"
## 2. Lint Check

**Status:** $lintStatus
**Duration:** $([math]::Round($lintDuration, 1))s

``````
$lintResult
``````

---

"@

# ============================================================
# 3. Build Check
# ============================================================

Write-Host "3. Running build check..." -ForegroundColor Yellow

$buildStart = Get-Date
$buildResult = & npm run build 2>&1 | Out-String
$buildEnd = Get-Date
$buildDuration = ($buildEnd - $buildStart).TotalSeconds

if ($LASTEXITCODE -eq 0) {
    Write-Host "   [PASS] Build successful ($([math]::Round($buildDuration, 1))s)" -ForegroundColor Green
    $buildStatus = "PASS"
} else {
    Write-Host "   [FAIL] Build failed" -ForegroundColor Red
    $buildStatus = "FAIL"
}

$report += @"
## 3. Build Check

**Status:** $buildStatus
**Duration:** $([math]::Round($buildDuration, 1))s

``````
$buildResult
``````

---

"@

# ============================================================
# 4. Unit Tests (if available)
# ============================================================

Write-Host "4. Running unit tests..." -ForegroundColor Yellow

$testResult = "No test script found or tests skipped"
$testStatus = "SKIP"

# Check if test script exists in package.json
$packageJson = Get-Content "package.json" -Raw -ErrorAction SilentlyContinue | ConvertFrom-Json -ErrorAction SilentlyContinue
if ($packageJson.scripts.test) {
    $testStart = Get-Date
    $testResult = & npm run test 2>&1 | Out-String
    $testEnd = Get-Date
    $testDuration = ($testEnd - $testStart).TotalSeconds

    if ($LASTEXITCODE -eq 0) {
        Write-Host "   [PASS] Tests passed ($([math]::Round($testDuration, 1))s)" -ForegroundColor Green
        $testStatus = "PASS"
    } else {
        Write-Host "   [FAIL] Tests failed" -ForegroundColor Red
        $testStatus = "FAIL"
    }
} else {
    Write-Host "   [SKIP] No test script configured" -ForegroundColor Gray
}

$report += @"
## 4. Unit Tests

**Status:** $testStatus

``````
$testResult
``````

---

"@

# ============================================================
# 5. Vercel Preview Deploy (optional)
# ============================================================

if ($DeployPreview) {
    Write-Host "5. Deploying Vercel preview..." -ForegroundColor Yellow

    $deployStart = Get-Date
    $deployResult = & vercel --prod=false 2>&1 | Out-String
    $deployEnd = Get-Date
    $deployDuration = ($deployEnd - $deployStart).TotalSeconds

    # Extract preview URL from output
    $previewUrl = ""
    if ($deployResult -match "(https://[^\s]+\.vercel\.app)") {
        $previewUrl = $Matches[1]
        Write-Host "   [PASS] Preview deployed: $previewUrl ($([math]::Round($deployDuration, 1))s)" -ForegroundColor Green
        $deployStatus = "PASS"
    } else {
        Write-Host "   [FAIL] Deploy failed" -ForegroundColor Red
        $deployStatus = "FAIL"
    }

    $report += @"
## 5. Vercel Preview

**Status:** $deployStatus
**URL:** $previewUrl
**Duration:** $([math]::Round($deployDuration, 1))s

``````
$deployResult
``````

---

"@
} else {
    Write-Host "5. Vercel preview skipped (use -DeployPreview to enable)" -ForegroundColor Gray
}

# ============================================================
# Summary
# ============================================================

$allPassed = ($typecheckStatus -eq "PASS") -and ($lintStatus -eq "PASS") -and ($buildStatus -eq "PASS")
if ($testStatus -eq "FAIL") { $allPassed = $false }

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "          Verification Summary                              " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  TypeScript: $typecheckStatus" -ForegroundColor $(if ($typecheckStatus -eq "PASS") { "Green" } else { "Red" })
Write-Host "  Lint:       $lintStatus" -ForegroundColor $(if ($lintStatus -eq "PASS") { "Green" } else { "Red" })
Write-Host "  Build:      $buildStatus" -ForegroundColor $(if ($buildStatus -eq "PASS") { "Green" } else { "Red" })
Write-Host "  Tests:      $testStatus" -ForegroundColor $(if ($testStatus -eq "PASS") { "Green" } elseif ($testStatus -eq "SKIP") { "Gray" } else { "Red" })
Write-Host ""

if ($allPassed) {
    Write-Host "  OVERALL: PASS" -ForegroundColor Green
    $overallStatus = "PASS"
} else {
    Write-Host "  OVERALL: FAIL" -ForegroundColor Red
    $overallStatus = "FAIL"
}

$report += @"
## Summary

| Check | Status |
|-------|--------|
| TypeScript | $typecheckStatus |
| Lint | $lintStatus |
| Build | $buildStatus |
| Tests | $testStatus |
| **Overall** | **$overallStatus** |

---

*Report generated by verify-task.ps1*
"@

# Write report
$report | Out-File -FilePath $reportFile -Encoding utf8
Write-Host ""
Write-Host "  Report saved: $reportFile" -ForegroundColor Gray
Write-Host ""

# Return exit code
if ($allPassed) {
    exit 0
} else {
    exit 1
}
