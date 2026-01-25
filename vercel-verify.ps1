# ============================================================
# Vercel Verification System
# ============================================================
# Deploys preview and captures verification feedback
#
# Usage: .\vercel-verify.ps1 [task-id]
# Example: .\vercel-verify.ps1 US-005
# ============================================================

param(
    [Parameter(Mandatory=$false)]
    [string]$TaskId = "milestone",

    [Parameter(Mandatory=$false)]
    [switch]$Production
)

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "          Vercel Verification System                        " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# ============================================================
# Pre-flight checks
# ============================================================

Write-Host "Running pre-flight checks..." -ForegroundColor Yellow
Write-Host ""

# Check if Vercel CLI is installed
$vercelVersion = & vercel --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Vercel CLI not installed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Install with: npm i -g vercel" -ForegroundColor Yellow
    Write-Host "Then run: vercel login" -ForegroundColor Yellow
    exit 1
}
Write-Host "  [OK] Vercel CLI: $vercelVersion" -ForegroundColor Green

# Check if project builds
Write-Host "  Checking build..." -ForegroundColor Gray
$buildResult = & npm run build 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [FAIL] Build failed - fix errors before deploying" -ForegroundColor Red
    Write-Host ""
    Write-Host $buildResult
    exit 1
}
Write-Host "  [OK] Build successful" -ForegroundColor Green

Write-Host ""

# ============================================================
# Deploy to Vercel
# ============================================================

Write-Host "Deploying to Vercel..." -ForegroundColor Yellow
Write-Host ""

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$deployName = "$TaskId-$timestamp"

if ($Production) {
    Write-Host "  Mode: PRODUCTION" -ForegroundColor Red
    $deployResult = & vercel --prod 2>&1 | Out-String
} else {
    Write-Host "  Mode: Preview" -ForegroundColor Cyan
    $deployResult = & vercel --prod=false 2>&1 | Out-String
}

# Extract deployment URL
$previewUrl = ""
if ($deployResult -match "(https://[^\s]+\.vercel\.app)") {
    $previewUrl = $Matches[1]
}

if ($previewUrl) {
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Green
    Write-Host "          DEPLOYMENT SUCCESSFUL                             " -ForegroundColor Green
    Write-Host "============================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Preview URL: " -NoNewline
    Write-Host "$previewUrl" -ForegroundColor Cyan
    Write-Host ""

    # Save deployment info
    $deploymentLog = "verification-reports/vercel-deployments.md"
    if (-not (Test-Path "verification-reports")) {
        New-Item -ItemType Directory -Path "verification-reports" -Force | Out-Null
    }

    $logEntry = @"

## $TaskId - $(Get-Date -Format "yyyy-MM-dd HH:mm")

- **URL:** $previewUrl
- **Mode:** $(if ($Production) { "Production" } else { "Preview" })
- **Status:** Deployed

"@
    Add-Content -Path $deploymentLog -Value $logEntry

    Write-Host "  Deployment logged to: $deploymentLog" -ForegroundColor Gray

} else {
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Red
    Write-Host "          DEPLOYMENT FAILED                                 " -ForegroundColor Red
    Write-Host "============================================================" -ForegroundColor Red
    Write-Host ""
    Write-Host $deployResult
    exit 1
}

# ============================================================
# Verification Checklist
# ============================================================

Write-Host ""
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host "          VERIFICATION CHECKLIST                            " -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Open the preview URL and verify:" -ForegroundColor White
Write-Host ""
Write-Host "  [ ] Application loads without errors" -ForegroundColor Gray
Write-Host "  [ ] No console errors in browser DevTools" -ForegroundColor Gray
Write-Host "  [ ] UI matches expected design" -ForegroundColor Gray
Write-Host "  [ ] Core functionality works as expected" -ForegroundColor Gray
Write-Host "  [ ] No visual glitches or layout issues" -ForegroundColor Gray
Write-Host ""
Write-Host "  For TerraNest specifically:" -ForegroundColor White
Write-Host ""
Write-Host "  [ ] 3D scene renders correctly" -ForegroundColor Gray
Write-Host "  [ ] Parcel sliders change terrain" -ForegroundColor Gray
Write-Host "  [ ] Unit presets generate buildings" -ForegroundColor Gray
Write-Host "  [ ] Camera controls work (orbit, zoom)" -ForegroundColor Gray
Write-Host "  [ ] Buildings position correctly on slope" -ForegroundColor Gray
Write-Host ""
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host ""

# ============================================================
# Get Deployment Details
# ============================================================

Write-Host "Fetching deployment details..." -ForegroundColor Gray

# Get deployment inspect info
$inspectResult = & vercel inspect $previewUrl 2>&1 | Out-String

# Save full deployment report
$reportFile = "verification-reports/deploy-$TaskId-$timestamp.md"
@"
# Vercel Deployment Report

**Task:** $TaskId
**Deployed:** $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
**URL:** $previewUrl

---

## Deployment Output

``````
$deployResult
``````

## Deployment Details

``````
$inspectResult
``````

---

## Verification Status

- [ ] Application loads
- [ ] No console errors
- [ ] UI correct
- [ ] Functionality works
- [ ] Performance acceptable

## Notes

(Add verification notes here)

"@ | Out-File -FilePath $reportFile -Encoding utf8

Write-Host ""
Write-Host "  Full report: $reportFile" -ForegroundColor Gray
Write-Host ""

# ============================================================
# Quick Actions
# ============================================================

Write-Host "Quick actions:" -ForegroundColor Yellow
Write-Host "  - Open URL: Start-Process '$previewUrl'" -ForegroundColor Gray
Write-Host "  - View logs: vercel logs $previewUrl" -ForegroundColor Gray
Write-Host "  - Inspect: vercel inspect $previewUrl" -ForegroundColor Gray
Write-Host "  - List all: vercel ls" -ForegroundColor Gray
Write-Host ""

exit 0
