Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Threading;
using System.Runtime.InteropServices;
public class WinStyleCheck {
    [DllImport("user32.dll")]
    public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll")]
    public static extern IntPtr OpenInputDesktop(int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll")]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")]
    public static extern int GetWindowLong(IntPtr hWnd, int nIndex);
    [DllImport("user32.dll")]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern bool IsZoomed(IntPtr hWnd);
    public const int GWL_STYLE = -16;
    public const uint WS_CAPTION = 0x00C00000;
    public static void Check() {
        Thread t = new Thread(() => {
            IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
            if (hDesk == IntPtr.Zero) hDesk = OpenInputDesktop(0, false, 0x01FF);
            if (hDesk != IntPtr.Zero) SetThreadDesktop(hDesk);
            
            IntPtr hwnd = GetForegroundWindow();
            if (hwnd == IntPtr.Zero) {
                Console.WriteLine("No foreground window");
                return;
            }
            var sbCls = new StringBuilder(256);
            GetClassName(hwnd, sbCls, 256);
            var sbTitle = new StringBuilder(256);
            GetWindowText(hwnd, sbTitle, 256);
            int style = GetWindowLong(hwnd, GWL_STYLE);
            bool hasCaption = ((uint)style & WS_CAPTION) == WS_CAPTION;
            bool zoomed = IsZoomed(hwnd);
            Console.WriteLine("HWND: " + hwnd + " Cls: " + sbCls + " Title: " + sbTitle + " Zoomed: " + zoomed + " hasCaption: " + hasCaption + " style: 0x" + style.ToString("X8"));
        });
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
    }
}
"@
[WinStyleCheck]::Check()
