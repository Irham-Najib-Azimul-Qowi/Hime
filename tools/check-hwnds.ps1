Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class HwndChecker {
    [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr hWnd, int nIndex);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static void Check(long hwndVal) {
        IntPtr h = new IntPtr(hwndVal);
        bool isW = IsWindow(h);
        bool vis = IsWindowVisible(h);
        RECT r;
        GetWindowRect(h, out r);
        int style = GetWindowLong(h, -16);
        int exStyle = GetWindowLong(h, -20);
        Console.WriteLine(string.Format("HWND: 0x{0:X} | Valid: {1} | Vis: {2} | Rect: [{3},{4} - {5},{6}] | Style: 0x{7:X} | ExStyle: 0x{8:X}",
            hwndVal, isW, vis, r.Left, r.Top, r.Right, r.Bottom, style, exStyle));
    }
}
'@

Write-Host "Checking Main:"
[HwndChecker]::Check(0x3E05C4)

Write-Host "Checking Dock:"
[HwndChecker]::Check(0x260782)
