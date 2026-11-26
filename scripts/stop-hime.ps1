# stop-hime.ps1 - Cleanly stop Hime Ecosystem and restore Windows taskbar
param([switch]$IncludeHotkey, [switch]$Native)

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

# 3. Wait up to 3 seconds for graceful exit before fallback force-kill
$targetProcs = if ($Native) { @("hime-island-native", "hime-core") } else { @("hime-island", "hime-core") }
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
if ($Native) {
    Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
} else {
    Get-Process -Name "hime-island" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# 5a. Only kill hotkey daemon when -IncludeHotkey is explicitly passed (e.g. before rebuilding).
#     On normal stop / Ctrl+Shift+Q, hime_hotkey stays alive so Ctrl+Shift+H / Ctrl+Shift+Space
#     can restart Hime anytime. Single-instance mutex + start-hime.ps1 prevent duplicates.
if ($IncludeHotkey) {
    Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# 5. Safely restore Windows native taskbars & work area
$restoreScript = "X:\folder_ai\Hime\island\scripts\restore-taskbar.ps1"
if (-not (Test-Path $restoreScript)) {
    $restoreScript = "X:\folder_ai\Hime\scripts\restore-taskbar.ps1"
}
if (Test-Path $restoreScript) {
    & $restoreScript
}

if ($IncludeHotkey) {
    Write-Host "Hime Ecosystem Stopped (including hotkey daemon) and Taskbar Restored." -ForegroundColor Green
    Write-Host "Use start-hime.ps1 or HimeLaunchTask to start Hime again." -ForegroundColor Cyan
} else {
    Write-Host "Hime Ecosystem Stopped and Taskbar Restored." -ForegroundColor Green
    Write-Host "Hotkey daemon remains active: Press Ctrl+Shift+H, Ctrl+Alt+Space, or Ctrl+Shift+Space to start Hime anytime." -ForegroundColor Cyan
}
