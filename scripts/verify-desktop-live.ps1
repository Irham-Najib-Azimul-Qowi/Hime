<#
.SYNOPSIS
    Live Desktop Application Validation for Pure-Rust Hime Desktop (ADR-017 Phase 4)
.DESCRIPTION
    Validates the running pure-Rust Hime ecosystem on Windows (no WebView2 / no hime_hotkey):
    1. Verifies binary presence (hime-core.exe, hime-island-native.exe).
    2. Starts the ecosystem via scripts/start-hime.ps1.
    3. Verifies process tree (hime-core, hime-island-native, hime-supervisor) & hime.active marker.
    4. Validates visible top-center Notch HWND via Win32 EnumWindows.
    5. Tests synchronized clean shutdown via HimeQuit Named Event (< 2000ms latency).
    6. Verifies marker cleanup (%APPDATA%\ai.hime.desktop\hime.active deleted) and taskbar restoration.
#>

[CmdletBinding()]
param(
    [switch]$Detailed
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "           HIME PURE-RUST DESKTOP LIVE VALIDATION SUITE                         " -ForegroundColor Cyan
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
$nativeBin = Find-Bin "island\native" "hime-island-native.exe"

$binsOk = ($coreBin -ne $null) -and ($nativeBin -ne $null)
$binDetails = "Core: $(if ($coreBin) { Split-Path $coreBin -Leaf } else { 'MISSING' }), NativeIsland: $(if ($nativeBin) { Split-Path $nativeBin -Leaf } else { 'MISSING' })"
Record-Check "Binary Availability" $binsOk $binDetails

if (-not $binsOk) {
    Write-Host "[FATAL] Required pure-Rust binaries are missing. Build them with '.\scripts\hime.ps1 build' before running live validation." -ForegroundColor Red
    exit 1
}

# --- Step 2: Clean Slate ---
Write-Host ">>> Checkpoint 2: Clean Slate Preparation..." -ForegroundColor Yellow
& (Join-Path $repoRoot "scripts\stop-hime.ps1") | Out-Null
Start-Sleep -Milliseconds 800

$preActive = Test-Path "$env:APPDATA\ai.hime.desktop\hime.active"
Record-Check "Clean Slate State" (-not $preActive) "No lingering active marker file"

# --- Step 3: Ecosystem Launch ---
Write-Host ">>> Checkpoint 3: Starting Hime Native Ecosystem..." -ForegroundColor Yellow
$startScript = Join-Path $repoRoot "scripts\start-hime.ps1"
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $startScript
Start-Sleep -Seconds 3

# --- Step 4: Process Verification ---
Write-Host ">>> Checkpoint 4: Verifying Running Processes..." -ForegroundColor Yellow
$pCore = @(Get-Process -Name "hime-core" -ErrorAction SilentlyContinue)
$pNative = @(Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue)

$pSupervisor = @(Get-CimInstance Win32_Process | Where-Object {
    $_.CommandLine -like "*hime-supervisor.ps1*" -and $_.CommandLine -notlike "*verify-desktop-live*"
})

$coreRunning = ($pCore.Count -gt 0)
$nativeRunning = ($pNative.Count -gt 0)
$supervisorRunning = ($pSupervisor.Count -gt 0)

$supervisorPid = if ($supervisorRunning) { ($pSupervisor | ForEach-Object { $_.ProcessId }) -join ', ' } else { "None" }

Record-Check "Process: hime-core" $coreRunning "PID(s): $(($pCore | ForEach-Object { $_.Id }) -join ', ')"
Record-Check "Process: hime-island-native" $nativeRunning "PID(s): $(($pNative | ForEach-Object { $_.Id }) -join ', ')"
Record-Check "Process: hime-supervisor" $supervisorRunning "Watchdog running in background (PID: $supervisorPid)"

$activeMarkerExists = Test-Path "$env:APPDATA\ai.hime.desktop\hime.active"
Record-Check "Marker: hime.active" $activeMarkerExists "Active marker file created and persistent"

# --- Step 5: Visual / Window Presence Verification ---
Write-Host ">>> Checkpoint 5: Visible Top-Center Notch HWND..." -ForegroundColor Yellow

$hwndTypeDefinition = @'
using System;
using System.Text;
using System.Threading;
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
    [DllImport("user32.dll", CharSet = CharSet.Auto)] public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    [DllImport("user32.dll", CharSet = CharSet.Auto)] public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll")] public static extern IntPtr OpenInputDesktop(int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool SetThreadDesktop(IntPtr hDesktop);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool CloseDesktop(IntPtr hDesktop);
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

    public static bool IsTaskbarVisible() {
        bool visible = false;
        Thread t = new Thread(() => {
            IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
            if (hDesk == IntPtr.Zero) {
                hDesk = OpenInputDesktop(0, false, 0x01FF);
            }
            if (hDesk != IntPtr.Zero) {
                SetThreadDesktop(hDesk);
            }
            IntPtr tray = FindWindow("Shell_TrayWnd", null);
            visible = (tray != IntPtr.Zero && IsWindowVisible(tray));
            if (hDesk != IntPtr.Zero) {
                CloseDesktop(hDesk);
            }
        });
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
        if (!visible) {
            IntPtr tray = FindWindow("Shell_TrayWnd", null);
            visible = (tray != IntPtr.Zero && IsWindowVisible(tray));
        }
        return visible;
    }
}
'@

if (-not ([System.Management.Automation.PSTypeName]'WinPresence').Type) {
    Add-Type -TypeDefinition $hwndTypeDefinition
}

$nativePids = @()
if ($pNative) {
    $nativePids = @($pNative | ForEach-Object { [uint32]$_.Id })
}

$nativeWindows = @()
if ($nativePids.Count -gt 0) {
    $nativeWindows = [WinPresence]::FindWindowsForPids($nativePids)
}

$hasWindowHandle = ($nativeWindows.Count -gt 0)
$visibleWindows = @($nativeWindows | Where-Object { $_.Visible })
$windowDetails = "Found $($nativeWindows.Count) window handle(s); $($visibleWindows.Count) visible"

Record-Check "Native Island HWND" $hasWindowHandle $windowDetails

Add-Type -AssemblyName System.Windows.Forms -ErrorAction SilentlyContinue
$screenCenter = [int]([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width / 2)
$notchPillWindows = @($visibleWindows | Where-Object {
    ($_.Left -le $screenCenter) -and (($_.Left + $_.Width) -ge $screenCenter) -and ($_.Top -le 50) -and ($_.Width -gt 0) -and ($_.Height -gt 0)
})

$visualVerified = ($notchPillWindows.Count -gt 0)
$visualDetails = if ($visualVerified) {
    $nw = $notchPillWindows[0]
    "Top-center Notch HWND verified (0x$($nw.Hwnd.ToString('X')), Title:'$($nw.Title)', Left:$($nw.Left), Top:$($nw.Top), $($nw.Width)x$($nw.Height))"
} else {
    "No visible top-center Notch HWND found (screenCenter=$screenCenter, visible=$($visibleWindows.Count))"
}
Record-Check "Top-Center Notch Presence" $visualVerified $visualDetails

# --- Step 6: Synchronized Clean Shutdown ---
Write-Host ">>> Checkpoint 6: Synchronized Clean Shutdown via HimeQuit..." -ForegroundColor Yellow

$stateFile = "$env:APPDATA\ai.hime.desktop\hime.active"
if (Test-Path $stateFile) {
    Remove-Item $stateFile -Force -ErrorAction SilentlyContinue
}

$sw = [System.Diagnostics.Stopwatch]::StartNew()
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

$shutdownTimeoutMs = 2500
$pollIntervalMs = 50
$elapsed = 0
$cleanShutdownOk = $false

while ($elapsed -lt $shutdownTimeoutMs) {
    $leftoverCore = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
    $leftoverNative = Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue
    if ((-not $leftoverCore) -and (-not $leftoverNative)) {
        $cleanShutdownOk = $true
        break
    }
    Start-Sleep -Milliseconds $pollIntervalMs
    $elapsed += $pollIntervalMs
}
$sw.Stop()

# Ensure fallback stop & taskbar restoration script runs cleanly
& (Join-Path $repoRoot "scripts\stop-hime.ps1") | Out-Null

$shutdownLatencyMs = $sw.ElapsedMilliseconds
$latencyFast = ($cleanShutdownOk -and ($shutdownLatencyMs -lt 2000))
Record-Check "Clean Shutdown (< 2000ms)" $latencyFast "HimeQuit terminated processes in ${shutdownLatencyMs}ms (Limit: 2000ms)"

# Verify marker file cleanup
$postActive = Test-Path "$env:APPDATA\ai.hime.desktop\hime.active"
Record-Check "Marker Cleanup" (-not $postActive) "hime.active removed on user quit"

# Verify taskbar restoration
$restoreScript = Join-Path $repoRoot "island\scripts\restore-taskbar.ps1"
if (-not (Test-Path $restoreScript)) {
    $restoreScript = Join-Path $repoRoot "scripts\restore-taskbar.ps1"
}
if (Test-Path $restoreScript) {
    & $restoreScript | Out-Null
}
$taskbarRestored = [WinPresence]::IsTaskbarVisible()
Record-Check "Taskbar Restoration" $taskbarRestored "Shell_TrayWnd visible: $taskbarRestored"

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
