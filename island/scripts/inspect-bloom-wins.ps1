Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinInspector {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static void Check() {
        EnumWindows((hWnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hWnd, out pid);
            StringBuilder t = new StringBuilder(256);
            GetWindowText(hWnd, t, 256);
            StringBuilder c = new StringBuilder(256);
            GetClassName(hWnd, c, 256);
            RECT r;
            GetWindowRect(hWnd, out r);
            bool vis = IsWindowVisible(hWnd);
            string title = t.ToString();
            string cls = c.ToString();
            if (title.Contains("Bloom") || title.Contains("Hime") || cls.Contains("Chrome_WidgetWin_0")) {
                Console.WriteLine(string.Format("PID={0} HWND=0x{1:X} Vis={2} Rect=[{3},{4} - {5},{6}] Title='{7}' Class='{8}'",
                    pid, hWnd.ToInt64(), vis, r.Left, r.Top, r.Right, r.Bottom, title, cls));
            }
            return true;
        }, IntPtr.Zero);
    }
}
"@
[WinInspector]::Check()
