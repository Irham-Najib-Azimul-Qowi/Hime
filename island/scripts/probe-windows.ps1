Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinProbe {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }

    public static void ListBloomWindows() {
        EnumWindows((hWnd, lParam) => {
            StringBuilder title = new StringBuilder(256);
            StringBuilder cls = new StringBuilder(256);
            GetWindowText(hWnd, title, 256);
            GetClassName(hWnd, cls, 256);
            uint procId;
            GetWindowThreadProcessId(hWnd, out procId);
            string t = title.ToString();
            string c = cls.ToString();
            if (t.Contains("Bloom") || t.Contains("Hime") || c.Contains("Bloom") || c.Contains("Chrome_WidgetWin_0")) {
                RECT r;
                GetWindowRect(hWnd, out r);
                bool vis = IsWindowVisible(hWnd);
                Console.WriteLine(string.Format("PID: {0} | HWND: 0x{1:X} | Vis: {2} | Rect: [{3},{4} - {5},{6}] | Class: {7} | Title: '{8}'",
                    procId, hWnd.ToInt64(), vis, r.Left, r.Top, r.Right, r.Bottom, c, t));
            }
            return true;
        }, IntPtr.Zero);
    }
}
"@
[WinProbe]::ListBloomWindows()
