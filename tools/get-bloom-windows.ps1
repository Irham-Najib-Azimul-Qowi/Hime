Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public class WinInspector2 {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static string[] GetAll() {
        List<string> list = new List<string>();
        EnumWindows((hWnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hWnd, out pid);
            StringBuilder title = new StringBuilder(256);
            StringBuilder cls = new StringBuilder(256);
            GetWindowText(hWnd, title, 256);
            GetClassName(hWnd, cls, 256);
            string t = title.ToString();
            string c = cls.ToString();
            if (t.Length > 0) {
                RECT r;
                GetWindowRect(hWnd, out r);
                bool vis = IsWindowVisible(hWnd);
                list.Add("PID: " + pid + " | HWND: 0x" + hWnd.ToInt64().ToString("X") + " | Vis: " + vis + " | Class: " + c + " | Title: '" + t + "'");
            }
            return true;
        }, IntPtr.Zero);
        return list.ToArray();
    }
}
'@

$res = [WinInspector2]::GetAll()
$res | ForEach-Object { Write-Output $_ }
