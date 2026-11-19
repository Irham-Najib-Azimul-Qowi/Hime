Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinProbeAll2 {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static void CheckPid(uint targetPid) {
        EnumWindows((hWnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hWnd, out pid);
            if (pid == targetPid) {
                StringBuilder t = new StringBuilder(256);
                GetWindowText(hWnd, t, 256);
                StringBuilder c = new StringBuilder(256);
                GetClassName(hWnd, c, 256);
                RECT r;
                GetWindowRect(hWnd, out r);
                bool vis = IsWindowVisible(hWnd);
                Console.WriteLine(string.Format("HWND=0x{0:X} Vis={1} Rect=[{2},{3} - {4},{5}] Title='{6}' Class='{7}'",
                    hWnd.ToInt64(), vis, r.Left, r.Top, r.Right, r.Bottom, t.ToString(), c.ToString()));
            }
            return true;
        }, IntPtr.Zero);
    }
}
"@

Get-Process bloom | ForEach-Object {
    Write-Host "Checking PID: $($_.Id)"
    [WinProbeAll2]::CheckPid([uint32]$_.Id)
}
