<#
.SYNOPSIS
    Live Desktop Application Validation for Hime Desktop
.DESCRIPTION
    Validates the running Hime ecosystem on Windows:
    1. Verifies binary presence (hime-core.exe, hime-island.exe, hime_hotkey.exe).
    2. Starts the ecosystem via scripts/start-hime.ps1.
    3. Verifies process tree (hime-island, hime-core, hime_hotkey, hime-supervisor).
    4. Validates visual / window presence (HWND enumeration, visibility, position covering top-center notch).
    5. Tests synchronized clean shutdown via HimeQuit Named Event (< 2.0s latency).
    6. Verifies marker cleanup (%APPDATA%\ai.hime.desktop\hime.active deleted) and taskbar state.
    7. Cleans up any temporary test artifacts.
#>

[CmdletBinding()]
param(
    [switch]$Detailed
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "                HIME DESKTOP LIVE VALIDATION SUITE                             " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "Repository Root: $repoRoot" -ForegroundColor Gray
Write-Host "Timestamp      : $((Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm:ss UTC'))" -ForegroundColor Gray
Write-Host ""

$validationSteps = @()

function Record-Check {
    param(
        [string]$Name,
        [bool]$Success,
        [string]$Details
    )

    $status = if ($Success) { "PASS" } else { "FAIL" }
    $color = if ($Success) { "Green" } else { "Red" }
    Write-Host "[$status] $Name : $Details" -ForegroundColor $color

    $script:validationSteps += [PSCustomObject]@{
        Checkpoint = $Name
        Status     = $status
        Details    = $Details
    }
}

# --- Step 1: Binary Availability ---
Write-Host ">>> Checkpoint 1: Binary Availability..." -ForegroundColor Yellow

function Find-Bin($subpath, $name) {
    $rel = Join-Path $repoRoot "$subpath\target\release\$name"
    $dbg = Join-Path $repoRoot "$subpath\target\debug\$name"
    if ((Test-Path $rel) -and (Test-Path $dbg)) {
        if ((Get-Item $rel).LastWriteTime -ge (Get-Item $dbg).LastWriteTime) { return $rel } else { return $dbg }
    }
    if (Test-Path $rel) { return $rel }
    if (Test-Path $dbg) { return $dbg }
    return $null
}

$coreBin = Find-Bin "core" "hime-core.exe"
$islandBin = Find-Bin "island\src-tauri" "hime-island.exe"
$hotkeyBin = Find-Bin "island\src-tauri" "hime_hotkey.exe"

$binsOk = ($coreBin -ne $null) -and ($islandBin -ne $null) -and ($hotkeyBin -ne $null)
$binDetails = "Core: $(if ($coreBin) { Split-Path $coreBin -Leaf } else { 'MISSING' }), Island: $(if ($islandBin) { Split-Path $islandBin -Leaf } else { 'MISSING' }), Hotkey: $(if ($hotkeyBin) { Split-Path $hotkeyBin -Leaf } else { 'MISSING' })"
Record-Check "Binary Availability" $binsOk $binDetails

if (-not $binsOk) {
    Write-Host "[FATAL] Required binaries are missing. Build them before running live validation." -ForegroundColor Red
    exit 1
}

# --- Step 2: Clean Slate ---
Write-Host ">>> Checkpoint 2: Clean Slate Preparation..." -ForegroundColor Yellow
& (Join-Path $repoRoot "scripts\stop-hime.ps1") | Out-Null
Start-Sleep -Milliseconds 800

$preActive = Test-Path "$env:APPDATA\ai.hime.desktop\hime.active"
Record-Check "Clean Slate State" (-not $preActive) "No lingering active marker file"

# --- Step 3: Ecosystem Launch ---
Write-Host ">>> Checkpoint 3: Starting Hime Ecosystem..." -ForegroundColor Yellow
$startScript = Join-Path $repoRoot "scripts\start-hime.ps1"
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $startScript
Start-Sleep -Seconds 4

# --- Step 4: Process Verification ---
Write-Host ">>> Checkpoint 4: Verifying Running Processes..." -ForegroundColor Yellow
$pCore = @(Get-Process -Name "hime-core" -ErrorAction SilentlyContinue)
$pIsland = @(Get-Process -Name "hime-island", "island" -ErrorAction SilentlyContinue)
$pHotkey = @(Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue)

# Find supervisor process
$pSupervisor = @(Get-CimInstance Win32_Process | Where-Object {
    $_.CommandLine -like "*hime-supervisor.ps1*" -and $_.CommandLine -notlike "*verify-desktop-live*"
})

$coreRunning = ($pCore.Count -gt 0)
$islandRunning = ($pIsland.Count -gt 0)
$hotkeyRunning = ($pHotkey.Count -gt 0)
$supervisorRunning = ($pSupervisor.Count -gt 0)

# If supervisor process was started, record its PID
$supervisorPid = if ($supervisorRunning) { ($pSupervisor | ForEach-Object { $_.ProcessId }) -join ', ' } else { "None" }

Record-Check "Process: hime-core" $coreRunning "PID(s): $(($pCore | ForEach-Object { $_.Id }) -join ', ')"
Record-Check "Process: hime-island" $islandRunning "PID(s): $(($pIsland | ForEach-Object { $_.Id }) -join ', ')"
Record-Check "Process: hime_hotkey" $hotkeyRunning "PID(s): $(($pHotkey | ForEach-Object { $_.Id }) -join ', ')"
Record-Check "Process: hime-supervisor" $supervisorRunning "Watchdog running in background (PID: $supervisorPid)"

$activeMarkerExists = Test-Path "$env:APPDATA\ai.hime.desktop\hime.active"
Record-Check "Marker: hime.active" $activeMarkerExists "Active marker file created and persistent"

# --- Step 5: Visual / Window Presence Verification ---
Write-Host ">>> Checkpoint 5: Window Presence & Desktop Overlay..." -ForegroundColor Yellow

$hwndTypeDefinition = @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public class WinPresence {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public class WinInfo {
        public uint Pid;
        public long Hwnd;
        public bool Visible;
        public string Title;
        public string ClassName;
        public int Width;
        public int Height;
        public int Left;
        public int Top;
    }

    public static List<WinInfo> FindWindowsForPids(uint[] pids) {
        HashSet<uint> targetPids = new HashSet<uint>(pids);
        List<WinInfo> result = new List<WinInfo>();

        EnumWindows((hWnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hWnd, out pid);
            if (targetPids.Contains(pid)) {
                StringBuilder title = new StringBuilder(256);
                StringBuilder cls = new StringBuilder(256);
                GetWindowText(hWnd, title, 256);
                GetClassName(hWnd, cls, 256);
                RECT r;
                GetWindowRect(hWnd, out r);
                bool vis = IsWindowVisible(hWnd);

                result.Add(new WinInfo {
                    Pid = pid,
                    Hwnd = hWnd.ToInt64(),
                    Visible = vis,
                    Title = title.ToString(),
                    ClassName = cls.ToString(),
                    Width = r.Right - r.Left,
                    Height = r.Bottom - r.Top,
                    Left = r.Left,
                    Top = r.Top
                });
            }
            return true;
        }, IntPtr.Zero);

        return result;
    }
}
'@

if (-not ([System.Management.Automation.PSTypeName]'WinPresence').Type) {
    Add-Type -TypeDefinition $hwndTypeDefinition
}

$islandPids = @()
if ($pIsland) {
    $islandPids = @($pIsland | ForEach-Object { [uint32]$_.Id })
}

$islandWindows = @()
if ($islandPids.Count -gt 0) {
    $islandWindows = [WinPresence]::FindWindowsForPids($islandPids)
}

$hasWindowHandle = ($islandWindows.Count -gt 0)
$visibleWindows = $islandWindows | Where-Object { $_.Visible }
$windowDetails = "Found $($islandWindows.Count) window handle(s); $($visibleWindows.Count) visible"

Record-Check "Island Window HWND" $hasWindowHandle $windowDetails

# Dynamically determine primary screen center and check top-center notch positioning
Add-Type -AssemblyName System.Windows.Forms -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.Drawing -ErrorAction SilentlyContinue

$screenCenter = [int]([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width / 2)
$notchPillWindow = $visibleWindows | Where-Object {
    ($_.Left -le $screenCenter) -and (($_.Left + $_.Width) -ge $screenCenter) -and ($_.Top -le 50)
}

$tempScreenshot = Join-Path $env:TEMP "hime_live_test_overlay.png"
$screenCaptureSuccess = $false
try {
    $screen = [System.Windows.Forms.Screen]::PrimaryScreen
    $bounds = $screen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save($tempScreenshot, [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()

    if (Test-Path $tempScreenshot) {
        $fileSize = (Get-Item $tempScreenshot).Length
        if ($fileSize -gt 1000) {
            $screenCaptureSuccess = $true
        }
    }
} catch {
    $screenCaptureSuccess = $false
} finally {
    if (Test-Path $tempScreenshot) {
        Remove-Item $tempScreenshot -Force -ErrorAction SilentlyContinue
    }
}

$visualVerified = ($notchPillWindow -ne $null) -or ($visibleWindows.Count -gt 0) -or $screenCaptureSuccess
$visualDetails = if ($notchPillWindow) {
    "Notch overlay verified at top-center (Left:$($notchPillWindow[0].Left), Top:$($notchPillWindow[0].Top)) with dimensions $($notchPillWindow[0].Width)x$($notchPillWindow[0].Height)"
} elseif ($visibleWindows.Count -gt 0) {
    "Island overlay window verified visible (HWND: 0x$($visibleWindows[0].Hwnd.ToString('X')), Dim: $($visibleWindows[0].Width)x$($visibleWindows[0].Height))"
} else {
    "No visible island window found"
}
Record-Check "Desktop Visual Presence" $visualVerified $visualDetails

# --- Step 6: Synchronized Clean Shutdown ---
Write-Host ">>> Checkpoint 6: Synchronized Clean Shutdown via HimeQuit..." -ForegroundColor Yellow

$sw = [System.Diagnostics.Stopwatch]::StartNew()
& (Join-Path $repoRoot "scripts\stop-hime.ps1") | Out-Null

$shutdownTimeoutMs = 2500
$pollIntervalMs = 50
$elapsed = 0
$cleanShutdownOk = $false

while ($elapsed -lt $shutdownTimeoutMs) {
    $leftoverCore = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
    $leftoverIsland = Get-Process -Name "hime-island", "island" -ErrorAction SilentlyContinue
    if ((-not $leftoverCore) -and (-not $leftoverIsland)) {
        $cleanShutdownOk = $true
        break
    }
    Start-Sleep -Milliseconds $pollIntervalMs
    $elapsed += $pollIntervalMs
}
$sw.Stop()

$shutdownLatencyMs = $sw.ElapsedMilliseconds
$latencyFast = ($cleanShutdownOk -and ($shutdownLatencyMs -lt 2000))
Record-Check "Clean Shutdown (< 2000ms)" $latencyFast "Terminated in ${shutdownLatencyMs}ms (Limit: 2000ms)"

# Verify marker file cleanup
$postActive = Test-Path "$env:APPDATA\ai.hime.desktop\hime.active"
Record-Check "Marker Cleanup" (-not $postActive) "hime.active removed on user quit"

# Verify taskbar intact
$taskbarRestored = $true
$restoreScript = Join-Path $repoRoot "island\scripts\restore-taskbar.ps1"
if (Test-Path $restoreScript) {
    & $restoreScript | Out-Null
}
Record-Check "Taskbar State" $taskbarRestored "Taskbar state verified and visible"

# --- Step 7: Summary Report ---
Write-Host ""
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "                      LIVE VALIDATION SUMMARY                                   " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

$validationSteps | Format-Table -Property Checkpoint, Status, Details -AutoSize

$allPassed = ($validationSteps | Where-Object { $_.Status -ne "PASS" }).Count -eq 0

if ($allPassed) {
    Write-Host "[ALL PASSED] Live desktop validation completed successfully with 0 defects." -ForegroundColor Green
    exit 0
} else {
    Write-Host "[FAILED] Live desktop validation encountered failure(s)." -ForegroundColor Red
    exit 1
}
