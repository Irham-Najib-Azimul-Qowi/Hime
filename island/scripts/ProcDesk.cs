using System;
using System.Runtime.InteropServices;
using System.Text;

public class ProcDesk {
    [DllImport("user32.dll")]
    public static extern IntPtr GetThreadDesktop(int dwThreadId);
    [DllImport("user32.dll")]
    public static extern bool GetUserObjectInformation(IntPtr hObj, int nIndex, StringBuilder pvInfo, int nLength, out int lpnLengthNeeded);

    public static void CheckProc(int threadId) {
        IntPtr hDesk = GetThreadDesktop(threadId);
        StringBuilder name = new StringBuilder(256);
        int len;
        GetUserObjectInformation(hDesk, 2, name, 256, out len);
        Console.WriteLine(string.Format("Thread {0} Desktop: {1} (handle: 0x{2:X})", threadId, name, hDesk.ToInt64()));
    }
}
