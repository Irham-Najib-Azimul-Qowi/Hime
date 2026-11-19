Add-Type -ReferencedAssemblies 'System.Drawing', 'System.Windows.Forms' -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;

public class OutsideTester {
    [DllImport("user32.dll")]
    public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);

    [DllImport("user32.dll")]
    public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);

    [DllImport("user32.dll")]
    public static extern IntPtr GetDC(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);

    [DllImport("gdi32.dll")]
    public static extern bool BitBlt(IntPtr hdcDest, int nXDest, int nYDest, int nWidth, int nHeight, IntPtr hdcSrc, int nXSrc, int nYSrc, int dwRop);

    [DllImport("user32.dll")]
    public static extern IntPtr WindowFromPoint(Point pt);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    const uint MOUSEEVENTF_MOVE = 0x0001;
    const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
    const uint MOUSEEVENTF_LEFTUP = 0x0004;

    public static void MoveCursor(int x, int y) {
        Cursor.Position = new Point(x, y);
        mouse_event(MOUSEEVENTF_MOVE, 1, 0, 0, UIntPtr.Zero);
        Thread.Sleep(50);
    }

    public static void Click(int x, int y) {
        Cursor.Position = new Point(x, y);
        mouse_event(MOUSEEVENTF_MOVE, 1, 0, 0, UIntPtr.Zero);
        Thread.Sleep(50);
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, UIntPtr.Zero);
        Thread.Sleep(60);
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, UIntPtr.Zero);
        Thread.Sleep(100);
    }

    public static void Capture(string path) {
        var bounds = Screen.PrimaryScreen.Bounds;
        using (Bitmap bmp = new Bitmap(bounds.Width, bounds.Height)) {
            using (Graphics g = Graphics.FromImage(bmp)) {
                IntPtr hdcDest = g.GetHdc();
                IntPtr hdcSrc = GetDC(IntPtr.Zero);
                BitBlt(hdcDest, 0, 0, bounds.Width, bounds.Height, hdcSrc, 0, 0, 0x40CC0020);
                ReleaseDC(IntPtr.Zero, hdcSrc);
                g.ReleaseHdc(hdcDest);
            }
            bmp.Save(path, ImageFormat.Png);
        }
    }

    public static void Run(string outDir) {
        Thread t = new Thread(() => {
            IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
            if (hDesk != IntPtr.Zero) SetThreadDesktop(hDesk);

            // Step 1: Initial notch state (cursor away from notch)
            MoveCursor(683, 500);
            Thread.Sleep(800);
            Capture(System.IO.Path.Combine(outDir, "step1_initial_notch.png"));

            // Step 2: Click to open card
            Click(683, 16);
            Thread.Sleep(1200);
            Capture(System.IO.Path.Combine(outDir, "step2_opened_card.png"));

            // Step 3: Move cursor OUTSIDE card (hover leave)
            MoveCursor(683, 500);
            Thread.Sleep(1400); // 350ms delay + animation
            Capture(System.IO.Path.Combine(outDir, "step3_after_hover_leave.png"));

            // Step 4: Click to open card again
            Click(683, 16);
            Thread.Sleep(1200);
            Capture(System.IO.Path.Combine(outDir, "step4_reopened_card.png"));

            // Step 5: Click OUTSIDE card (immediate dismiss)
            Click(200, 200);
            Thread.Sleep(1000);
            Capture(System.IO.Path.Combine(outDir, "step5_after_click_outside.png"));
        });
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
    }
}
'@

$dir = "C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45"
[OutsideTester]::Run($dir)
Write-Host "OutsideTester completed."
