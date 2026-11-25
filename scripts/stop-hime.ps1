# stop-hime.ps1 - Cleanly stop Hime Ecosystem and restore Windows taskbar
$ErrorActionPreference = "SilentlyContinue"

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

# 3. Wait up to 3 seconds for graceful exit of hime-island and hime-core before fallback force-kill
$timeoutMs = 3000
$intervalMs = 100
$elapsed = 0
while ($elapsed -lt $timeoutMs) {
    $running = Get-Process -Name "hime-island", "hime-core" -ErrorAction SilentlyContinue
    if (-not $running) {
        break
    }
    Start-Sleep -Milliseconds $intervalMs
    $elapsed += $intervalMs
}

# 4. Fallback force termination only if processes did not exit cleanly within 3 seconds
Get-Process -Name "hime-core" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process -Name "hime-island" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 5a. Kill hotkey daemon(s) so the next start-hime.ps1 can re-register all hotkeys cleanly.
#     All hime_hotkey instances must exit before start-hime spawns a new one, otherwise
#     RegisterHotKey returns ERROR_HOTKEY_ALREADY_REGISTERED (0x581) for every key and
#     the new daemon is permanently deaf.
Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 5. Safely restore Windows native taskbars & work area
$restoreScript = "X:\folder_ai\Hime\island\scripts\restore-taskbar.ps1"
if (-not (Test-Path $restoreScript)) {
    $restoreScript = "X:\folder_ai\Hime\scripts\restore-taskbar.ps1"
}
if (Test-Path $restoreScript) {
    & $restoreScript
}

Write-Host "Hime Ecosystem Stopped (including hotkey daemon) and Taskbar Restored." -ForegroundColor Green
Write-Host "Use start-hime.ps1 or HimeLaunchTask to start Hime again." -ForegroundColor Cyan
