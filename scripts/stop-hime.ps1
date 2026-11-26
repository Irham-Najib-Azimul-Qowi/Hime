# stop-hime.ps1 - Cleanly stop Hime Pure-Rust Native Ecosystem and restore Windows taskbar
param([switch]$IncludeHotkey)

$ErrorActionPreference = "SilentlyContinue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$stateFile = "$env:APPDATA\ai.hime.desktop\hime.active"

# 1. Remove active state flag to stop supervisor loop
if (Test-Path $stateFile) {
    Remove-Item $stateFile -Force -ErrorAction SilentlyContinue
}

# 2. Signal Win32 Named Event HimeQuit to initiate synchronized clean shutdown
try {
    $quitEvent = [System.Threading.EventWaitHandle]::OpenExisting("HimeQuit")
    $quitEvent.Set() | Out-Null
    $quitEvent.Dispose()
} catch {
    try {
        $quitEvent = New-Object System.Threading.EventWaitHandle($true, [System.Threading.EventResetMode]::ManualReset, "HimeQuit")
        $quitEvent.Set() | Out-Null
        $quitEvent.Dispose()
    } catch {}
}

# 3. Wait up to 3 seconds for graceful exit before fallback force-kill
$targetProcs = @("hime-island-native", "hime-core")
$timeoutMs = 3000
$intervalMs = 100
$elapsed = 0
while ($elapsed -lt $timeoutMs) {
    $running = Get-Process -Name $targetProcs -ErrorAction SilentlyContinue
    if (-not $running) {
        break
    }
    Start-Sleep -Milliseconds $intervalMs
    $elapsed += $intervalMs
}

# 4. Fallback force termination only if processes did not exit cleanly within 3 seconds
Get-Process -Name "hime-core" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

if ($IncludeHotkey) {
    Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# 5. Safely restore Windows native taskbars & work area
$restoreScript = Join-Path $repoRoot "island\scripts\restore-taskbar.ps1"
if (-not (Test-Path $restoreScript)) {
    $restoreScript = Join-Path $repoRoot "scripts\restore-taskbar.ps1"
}
if (Test-Path $restoreScript) {
    & $restoreScript
}

Write-Host "Hime Native Ecosystem Stopped and Taskbar Restored." -ForegroundColor Green
Write-Host "Use scripts\start-hime.ps1 or Ctrl+Shift+H (when shell is minimized/idle) to launch Hime." -ForegroundColor Cyan
