# stop-hime.ps1 - Cleanly stop Hime Pure-Rust Native Ecosystem and restore Windows taskbar
param([switch]$IncludeHotkey, [switch]$RestoreTaskbar)

$ErrorActionPreference = "SilentlyContinue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$stateFile = "$env:APPDATA\ai.hime.desktop\hime.active"
$hadActiveMarker = Test-Path $stateFile

# 1. Remove active state flag to stop supervisor loop
if ($hadActiveMarker) {
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

# 3. Wait up to 1.2 seconds (50ms poll) for graceful exit before fallback force-kill
$targetProcs = @("hime-island-native", "hime-core")
$running = Get-Process -Name $targetProcs -ErrorAction SilentlyContinue
$hadRunning = ($null -ne $running)
if ($hadRunning) {
    $timeoutMs = 1200
    $intervalMs = 50
    $elapsed = 0
    while ($elapsed -lt $timeoutMs) {
        Start-Sleep -Milliseconds $intervalMs
        $elapsed += $intervalMs
        $running = Get-Process -Name $targetProcs -ErrorAction SilentlyContinue
        if (-not $running) {
            break
        }
    }
}

# 4. Fallback force termination only if processes did not exit cleanly within timeout
Get-Process -Name "hime-core" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

if ($IncludeHotkey) {
    Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# 5. Restore Windows native taskbars & work area when processes were active/crashed or explicitly requested
if ($hadRunning -or $hadActiveMarker -or $RestoreTaskbar) {
    $restoreScript = Join-Path $repoRoot "island\scripts\restore-taskbar.ps1"
    if (-not (Test-Path $restoreScript)) {
        $restoreScript = Join-Path $repoRoot "scripts\restore-taskbar.ps1"
    }
    if (Test-Path $restoreScript) {
        & $restoreScript
    }
}

Write-Host "Hime Native Ecosystem Stopped and Taskbar Restored." -ForegroundColor Green
Write-Host "Use scripts\start-hime.ps1 or Ctrl+Shift+H (when shell is minimized/idle) to launch Hime." -ForegroundColor Cyan
