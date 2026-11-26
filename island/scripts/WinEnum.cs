using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;
using System.Threading;

public class WinEnum {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumChildWindows(IntPtr hWndParent, EnumWindowsProc enumProc, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern int GetWindowLong(IntPtr hWnd, int nIndex);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("dwmapi.dll")]
    public static extern int DwmGetWindowAttribute(IntPtr hwnd, int dwAttribute, out int pvAttribute, int cbAttribute);

    [DllImport("user32.dll")]
    public static extern IntPtr OpenInputDesktop(int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll")]
    public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);

    public struct RECT { public int Left, Top, Right, Bottom; }

    public static void ListAllWindowsForPids(int[] targetPids) {
        Thread t = new Thread(() => {
            IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
            if (hDesk == IntPtr.Zero) hDesk = OpenInputDesktop(0, false, 0x01FF);
            if (hDesk != IntPtr.Zero) SetThreadDesktop(hDesk);

            var pidSet = new HashSet<int>(targetPids);
            EnumWindows((hWnd, lParam) => {
                uint pid;
                GetWindowThreadProcessId(hWnd, out pid);
                if (pidSet.Contains((int)pid)) {
                    PrintWnd("TopLevel", hWnd, pid);
                    EnumChildWindows(hWnd, (childHwnd, l) => {
                        uint cpid;
                        GetWindowThreadProcessId(childHwnd, out cpid);
                        PrintWnd("  Child", childHwnd, cpid);
                        return true;
                    }, IntPtr.Zero);
                }
                return true;
            }, IntPtr.Zero);
        });
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
    }

    private static void PrintWnd(string prefix, IntPtr hWnd, uint pid) {
        var title = new StringBuilder(256);
        var cls = new StringBuilder(256);
        GetWindowText(hWnd, title, 256);
        GetClassName(hWnd, cls, 256);
        RECT r;
        GetWindowRect(hWnd, out r);
        int style = GetWindowLong(hWnd, -16);
        int exStyle = GetWindowLong(hWnd, -20);
        int cloaked = 0;
        DwmGetWindowAttribute(hWnd, 14, out cloaked, 4);

        Console.WriteLine(string.Format("{0} HWND: 0x{1:X} | PID: {2} | Vis: {3} | Cloaked: {4} | Pos: ({5},{6})-({7},{8}) [{9}x{10}] | Ex: 0x{11:X8} | Cls: {12} | Title: {13}",
            prefix, hWnd.ToInt64(), pid, IsWindowVisible(hWnd), cloaked, r.Left, r.Top, r.Right, r.Bottom, r.Right - r.Left, r.Bottom - r.Top, exStyle, cls, title));
    }
}
