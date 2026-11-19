param([int]$TargetPid = 23472)

Add-Type -TypeDefinition @'
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
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static void CheckPid(uint targetPid) {
        EnumWindows((hWnd, lParam) => {
            uint procId;
            GetWindowThreadProcessId(hWnd, out procId);
            if (procId == targetPid) {
                StringBuilder title = new StringBuilder(256);
                StringBuilder cls = new StringBuilder(256);
                GetWindowText(hWnd, title, 256);
                GetClassName(hWnd, cls, 256);
                RECT r;
                GetWindowRect(hWnd, out r);
                bool vis = IsWindowVisible(hWnd);
                Console.WriteLine("HWND: 0x" + hWnd.ToInt64().ToString("X") + " | Vis: " + vis + " | Rect: [" + r.Left + "," + r.Top + " - " + r.Right + "," + r.Bottom + "] | Class: " + cls.ToString() + " | Title: '" + title.ToString() + "'");
            }
            return true;
        }, IntPtr.Zero);
    }
}
'@

[WinProbe]::CheckPid([uint32]$TargetPid)
