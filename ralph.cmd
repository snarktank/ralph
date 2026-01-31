@echo off
REM Ralph Wiggum - Long-running AI agent loop (Windows CMD wrapper)
REM Usage: ralph.cmd [--tool amp|claude] [max_iterations]
REM
REM This is a wrapper that calls the PowerShell script.
REM For full functionality, run ralph.ps1 directly in PowerShell.

setlocal enabledelayedexpansion

REM Get the directory where this script is located
set "SCRIPT_DIR=%~dp0"

REM Check if PowerShell is available
where pwsh >nul 2>&1
if %errorlevel% equ 0 (
    REM Use PowerShell Core (pwsh) if available
    pwsh -ExecutionPolicy Bypass -File "%SCRIPT_DIR%ralph.ps1" %*
) else (
    REM Fall back to Windows PowerShell
    powershell -ExecutionPolicy Bypass -File "%SCRIPT_DIR%ralph.ps1" %*
)

exit /b %errorlevel%
