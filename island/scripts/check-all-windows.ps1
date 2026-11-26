Add-Type @"
using System;
using System.Runtime.InteropServices;

public class WinCheck2 {
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern bool GetWindowPlacement(IntPtr hWnd, ref WINDOWPLACEMENT lpwndpl);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left, Top, Right, Bottom;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT {
        public int x, y;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct WINDOWPLACEMENT {
        public int length;
        public int flags;
        public int showCmd;
        public POINT ptMinPosition;
        public POINT ptMaxPosition;
        public RECT rcNormalPosition;
    }

    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    public static void Check() {
        EnumWindows((hWnd, lParam) => {
            if (!IsWindowVisible(hWnd)) return true;
            System.Text.StringBuilder sb = new System.Text.StringBuilder(256);
            GetWindowText(hWnd, sb, 256);
            string title = sb.ToString();
            if (string.IsNullOrEmpty(title)) return true;

            RECT r;
            GetWindowRect(hWnd, out r);
            if ((r.Right - r.Left) > 200 && (r.Bottom - r.Top) > 200) {
                WINDOWPLACEMENT wp = new WINDOWPLACEMENT();
                wp.length = Marshal.SizeOf(wp);
                GetWindowPlacement(hWnd, ref wp);
                Console.WriteLine("Title: " + title);
                Console.WriteLine("  Handle: 0x" + hWnd.ToInt64().ToString("X"));
                Console.WriteLine("  ShowCmd: " + wp.showCmd + " (1=Normal, 3=Maximized)");
                Console.WriteLine("  Rect: (" + r.Left + "," + r.Top + ") to (" + r.Right + "," + r.Bottom + ") -> " + (r.Right - r.Left) + "x" + (r.Bottom - r.Top));
            }
            return true;
        }, IntPtr.Zero);
    }
}
"@

[WinCheck2]::Check()
