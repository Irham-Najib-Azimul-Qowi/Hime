# stop-hime.ps1 - Cleanly stop Hime Ecosystem and restore Windows taskbar
$ErrorActionPreference = "SilentlyContinue"

$stateFile = "$env:APPDATA\ai.hime.desktop\hime.active"

# 1. Remove active state flag to stop supervisor loop
if (Test-Path $stateFile) {
    Remove-Item $stateFile -Force -ErrorAction SilentlyContinue
}

# 2. Terminate Hime Core
Get-Process -Name "hime-core" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 3. Terminate Island Shell
Get-Process -Name "hime-island" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 4. Safely restore Windows native taskbars & work area
$restoreScript = "X:\folder_ai\Hime\island\scripts\restore-taskbar.ps1"
if (Test-Path $restoreScript) {
    & $restoreScript
}

Write-Host "Hime Ecosystem Stopped and Taskbar Restored." -ForegroundColor Green
Write-Host "Hotkey daemon remains active: Press Ctrl+Shift+H or Ctrl+Alt+Space to start Hime anytime." -ForegroundColor Cyan
