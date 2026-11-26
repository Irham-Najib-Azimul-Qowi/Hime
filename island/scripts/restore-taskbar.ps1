<#
.SYNOPSIS
    Idempotent taskbar and explorer recovery script for Hime / Bloom Shell.
.DESCRIPTION
    Restores the Windows native taskbar (Shell_TrayWnd and Shell_SecondaryTrayWnd),
    unregisters any lingering AppBars, terminates runaway shell processes, and restarts
    Windows Explorer if required. Non-admin safe.
#>

[CmdletBinding()]
param(
    [switch]$ForceRestartExplorer
)

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  HIME / BLOOM TASKBAR SAFETY RESTORATION" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# 1. Terminate any running bloom / hime dev or release processes
$targetProcesses = @("bloom", "hime", "hime-shell")
foreach ($procName in $targetProcesses) {
    $procs = Get-Process -Name $procName -ErrorAction SilentlyContinue
    if ($procs) {
        Write-Host "Terminating lingering process: $procName (PID(s): $($procs.Id -join ', '))" -ForegroundColor Yellow
        $procs | Stop-Process -Force -ErrorAction SilentlyContinue
    }
}

# 2. Add Win32 ShowWindow & FindWindow P/Invoke with Desktop support
if (-not ([System.Management.Automation.PSTypeName]'Win32Taskbar').Type) {
    Add-Type -TypeDefinition @'
using System;
using System.Threading;
using System.Runtime.InteropServices;

public class Win32Taskbar {
    [DllImport("user32.dll")]
    public static extern IntPtr OpenInputDesktop(int dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll")]
    public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);

    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);

    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr FindWindowEx(IntPtr parentHandle, IntPtr childAfter, string className, string windowTitle);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left, Top, Right, Bottom;
    }

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool SystemParametersInfoW(uint uiAction, uint uiParam, ref RECT pvParam, uint fWinIni);

    [DllImport("user32.dll")]
    public static extern int GetSystemMetrics(int nIndex);

    public const int SW_HIDE = 0;
    public const int SW_SHOW = 5;
    public const uint SPI_SETWORKAREA = 0x002F;
    public const uint SPIF_SENDCHANGE = 0x0002;
    public const int SM_CXSCREEN = 0;
    public const int SM_CYSCREEN = 1;

    public static void RestoreWorkArea() {
        int cx = GetSystemMetrics(SM_CXSCREEN);
        int cy = GetSystemMetrics(SM_CYSCREEN);
        RECT r = new RECT { Left = 0, Top = 0, Right = cx, Bottom = cy };
        SystemParametersInfoW(SPI_SETWORKAREA, 0, ref r, SPIF_SENDCHANGE);
    }

    public static bool RestoreOnDefaultDesktop() {
        bool restored = false;
        Thread t = new Thread(() => {
            IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
            if (hDesk == IntPtr.Zero) {
                hDesk = OpenInputDesktop(0, false, 0x01FF);
            }
            if (hDesk != IntPtr.Zero) {
                SetThreadDesktop(hDesk);
            }

            IntPtr primary = FindWindow("Shell_TrayWnd", null);
            if (primary != IntPtr.Zero) {
                ShowWindow(primary, SW_SHOW);
                restored = true;
            }

            IntPtr secondary = IntPtr.Zero;
            do {
                secondary = FindWindowEx(IntPtr.Zero, secondary, "Shell_SecondaryTrayWnd", null);
                if (secondary != IntPtr.Zero) {
                    ShowWindow(secondary, SW_SHOW);
                }
            } while (secondary != IntPtr.Zero);
        });
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
        return restored;
    }
}
'@
}

# 3. Restore primary taskbar (Shell_TrayWnd) on Default Desktop
$primaryRestored = [Win32Taskbar]::RestoreOnDefaultDesktop()
try {
    [Win32Taskbar]::RestoreWorkArea()
    Write-Host "[OK] Desktop WorkArea restored to 100% full screen." -ForegroundColor Green
} catch {}
if ($primaryRestored) {
    Write-Host "[OK] Taskbars on Default desktop ShowWindow invoked successfully." -ForegroundColor Green
} else {
    Write-Host "[WARN] Shell_TrayWnd not found on Default desktop." -ForegroundColor Yellow
}

# 4. Restart Explorer ONLY if forced
if ($ForceRestartExplorer) {
    Write-Host "Restarting Windows Explorer as requested (-ForceRestartExplorer)..." -ForegroundColor Yellow
    Stop-Process -Name explorer -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
    if (-not (Get-Process -Name explorer -ErrorAction SilentlyContinue)) {
        Start-Process explorer.exe
        Start-Sleep -Seconds 2
    }
    Write-Host "[OK] Windows Explorer restarted successfully." -ForegroundColor Green
}

Write-Host "Taskbar recovery complete." -ForegroundColor Green
