Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinSearch2 {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);

    public static void Search() {
        int count = 0;
        EnumWindows((hWnd, lParam) => {
            if (IsWindowVisible(hWnd)) {
                StringBuilder title = new StringBuilder(256);
                GetWindowText(hWnd, title, 256);
                if (title.Length > 0) {
                    uint procId;
                    GetWindowThreadProcessId(hWnd, out procId);
                    Console.WriteLine(string.Format("HWND: {0}, PID: {1}, Title: {2}", hWnd, procId, title));
                    count++;
                }
            }
            return true;
        }, IntPtr.Zero);
        Console.WriteLine("Total visible windows with title: " + count);
    }
}
"@

[WinSearch2]::Search()
