using System;
using System.Threading;
using System.Runtime.InteropServices;

public class ThreadDesktopSwitcher {
    [DllImport("user32.dll")]
    public static extern IntPtr OpenInputDesktop(int dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll")]
    public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);

    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);

    public static bool SwitchToDefaultAndFindTaskbar() {
        bool found = false;
        Thread t = new Thread(() => {
            IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
            if (hDesk == IntPtr.Zero) {
                hDesk = OpenInputDesktop(0, false, 0x01FF);
            }
            if (hDesk == IntPtr.Zero) {
                Console.WriteLine("Could not open Default or Input desktop.");
                return;
            }

            bool switched = SetThreadDesktop(hDesk);
            int err = Marshal.GetLastWin32Error();
            Console.WriteLine(string.Format("New Thread SetThreadDesktop: {0} (err: {1})", switched, err));

            IntPtr tray = FindWindow("Shell_TrayWnd", null);
            Console.WriteLine(string.Format("Shell_TrayWnd HWND on Default desktop: 0x{0:X}", tray.ToInt64()));
            found = (tray != IntPtr.Zero);
        });

        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
        return found;
    }
}
