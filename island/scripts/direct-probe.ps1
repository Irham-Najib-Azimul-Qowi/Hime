Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class WinDirectProbe2 {
    [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll", SetLastError = true)] public static extern IntPtr OpenDesktop(string lpszDesktop, uint dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool SetThreadDesktop(IntPtr hDesktop);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool CloseDesktop(IntPtr hDesktop);

    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static void Probe(IntPtr hWnd) {
        IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
        if (hDesk != IntPtr.Zero) SetThreadDesktop(hDesk);
        bool isW = IsWindow(hWnd);
        bool vis = IsWindowVisible(hWnd);
        RECT r;
        GetWindowRect(hWnd, out r);
        Console.WriteLine(string.Format("HWND=0x{0:X} IsWindow={1} Vis={2} Rect=[{3},{4} - {5},{6}]",
            hWnd.ToInt64(), isW, vis, r.Left, r.Top, r.Right, r.Bottom));
        if (hDesk != IntPtr.Zero) CloseDesktop(hDesk);
    }
}
"@
[WinDirectProbe2]::Probe([IntPtr]0xC06BC)
[WinDirectProbe2]::Probe([IntPtr]0x1606B0)
