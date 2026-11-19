Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinSearch {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll", SetLastError = true)] public static extern IntPtr OpenDesktop(string lpszDesktop, uint dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool SetThreadDesktop(IntPtr hDesktop);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool CloseDesktop(IntPtr hDesktop);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }

    public static void Search() {
        IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
        if (hDesk != IntPtr.Zero) SetThreadDesktop(hDesk);

        EnumWindows((hWnd, lParam) => {
            StringBuilder title = new StringBuilder(256);
            StringBuilder cls = new StringBuilder(256);
            GetWindowText(hWnd, title, 256);
            GetClassName(hWnd, cls, 256);
            string t = title.ToString();
            string c = cls.ToString();
            if (t.Contains("Hime") || t.Contains("Bloom") || c.Contains("Bloom") || c.Contains("Tauri") || c.Contains("Chrome_WidgetWin_0")) {
                uint procId;
                GetWindowThreadProcessId(hWnd, out procId);
                RECT r;
                GetWindowRect(hWnd, out r);
                bool vis = IsWindowVisible(hWnd);
                Console.WriteLine(string.Format("HWND: {0}, PID: {1}, Visible: {2}, Rect: [{3},{4} - {5},{6}], Class: {7}, Title: {8}",
                    hWnd, procId, vis, r.Left, r.Top, r.Right, r.Bottom, c, t));
            }
            return true;
        }, IntPtr.Zero);
        if (hDesk != IntPtr.Zero) CloseDesktop(hDesk);
    }
}
"@

[WinSearch]::Search()
