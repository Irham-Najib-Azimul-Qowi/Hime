using System;
using System.Runtime.InteropServices;
using System.Text;

public class DeskCheck {
    [DllImport("user32.dll")]
    public static extern IntPtr GetThreadDesktop(int dwThreadId);
    [DllImport("kernel32.dll")]
    public static extern int GetCurrentThreadId();
    [DllImport("user32.dll")]
    public static extern bool GetUserObjectInformation(IntPtr hObj, int nIndex, StringBuilder pvInfo, int nLength, out int lpnLengthNeeded);
    [DllImport("user32.dll")]
    public static extern IntPtr OpenInputDesktop(int dwFlags, bool fInherit, int dwDesiredAccess);

    public static void Check() {
        IntPtr hDesk = GetThreadDesktop(GetCurrentThreadId());
        StringBuilder name = new StringBuilder(256);
        int len;
        GetUserObjectInformation(hDesk, 2, name, 256, out len);
        Console.WriteLine(string.Format("Current Thread Desktop: {0}", name));

        IntPtr inputDesk = OpenInputDesktop(0, false, 0x01FF);
        StringBuilder inputName = new StringBuilder(256);
        GetUserObjectInformation(inputDesk, 2, inputName, 256, out len);
        Console.WriteLine(string.Format("Input Desktop: {0} (handle: 0x{1:X})", inputName, inputDesk.ToInt64()));
    }
}
