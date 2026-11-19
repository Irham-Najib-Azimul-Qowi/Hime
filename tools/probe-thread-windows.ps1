Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Diagnostics;
using System.Runtime.InteropServices;
public class ThreadProbe {
    public delegate bool EnumThreadWndProc(IntPtr hWnd, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool EnumThreadWindows(int threadId, EnumThreadWndProc lpfn, IntPtr lParam);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static void CheckProc(int pid) {
        try {
            Process p = Process.GetProcessById(pid);
            foreach (ProcessThread t in p.Threads) {
                EnumThreadWindows(t.Id, (hWnd, lp) => {
                    StringBuilder title = new StringBuilder(256);
                    StringBuilder cls = new StringBuilder(256);
                    GetWindowText(hWnd, title, 256);
                    GetClassName(hWnd, cls, 256);
                    RECT r;
                    GetWindowRect(hWnd, out r);
                    bool vis = IsWindowVisible(hWnd);
                    Console.WriteLine("Thread " + t.Id + " | HWND: 0x" + hWnd.ToInt64().ToString("X") + " | Vis: " + vis + " | Rect: [" + r.Left + "," + r.Top + " - " + r.Right + "," + r.Bottom + "] | Class: " + cls + " | Title: '" + title + "'");
                    return true;
                }, IntPtr.Zero);
            }
        } catch (Exception ex) {
            Console.WriteLine("Error: " + ex.Message);
        }
    }
}
'@

[ThreadProbe]::CheckProc(23472)
