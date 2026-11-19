using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;

public class WinFinder {
    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr FindWindowEx(IntPtr parentHandle, IntPtr childAfter, string className, string windowTitle);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern bool IsWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    public struct RECT { public int Left, Top, Right, Bottom; }

    public static void FindAll() {
        IntPtr cur = IntPtr.Zero;
        int count = 0;
        int bloomCount = 0;
        while ((cur = FindWindowEx(IntPtr.Zero, cur, null, null)) != IntPtr.Zero) {
            count++;
            uint pid;
            GetWindowThreadProcessId(cur, out pid);
            var title = new StringBuilder(256);
            var cls = new StringBuilder(256);
            GetWindowText(cur, title, 256);
            GetClassName(cur, cls, 256);
            if (title.ToString().ToLower().Contains("bloom") || cls.ToString().ToLower().Contains("tauri") || cls.ToString().ToLower().Contains("bloom") || pid == 4436) {
                bloomCount++;
                RECT r;
                GetWindowRect(cur, out r);
                Console.WriteLine(string.Format("MATCH HWND 0x{0:X} | PID {1} | Vis {2} | Pos ({3},{4})-({5},{6}) | Cls {7} | Title {8}",
                    cur.ToInt64(), pid, IsWindowVisible(cur), r.Left, r.Top, r.Right, r.Bottom, cls, title));
            }
        }
        Console.WriteLine(string.Format("Total top-level windows: {0}, Bloom matches: {1}", count, bloomCount));
    }
}
