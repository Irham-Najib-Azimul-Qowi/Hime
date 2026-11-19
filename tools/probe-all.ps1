Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinProbeAll {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static void ListAll() {
        EnumWindows((hWnd, lParam) => {
            StringBuilder title = new StringBuilder(256);
            StringBuilder cls = new StringBuilder(256);
            GetWindowText(hWnd, title, 256);
            GetClassName(hWnd, cls, 256);
            string t = title.ToString();
            string c = cls.ToString();
            if (t.IndexOf("Hime", StringComparison.OrdinalIgnoreCase) >= 0 ||
                t.IndexOf("Bloom", StringComparison.OrdinalIgnoreCase) >= 0 ||
                t.IndexOf("Dock", StringComparison.OrdinalIgnoreCase) >= 0 ||
                c.IndexOf("Chrome_WidgetWin", StringComparison.OrdinalIgnoreCase) >= 0) {
                uint procId;
                GetWindowThreadProcessId(hWnd, out procId);
                RECT r;
                GetWindowRect(hWnd, out r);
                bool vis = IsWindowVisible(hWnd);
                Console.WriteLine("PID: " + procId + " | HWND: 0x" + hWnd.ToInt64().ToString("X") + " | Vis: " + vis + " | Rect: [" + r.Left + "," + r.Top + " - " + r.Right + "," + r.Bottom + "] | Class: " + c + " | Title: '" + t + "'");
            }
            return true;
        }, IntPtr.Zero);
    }
}
'@

[WinProbeAll]::ListAll()
