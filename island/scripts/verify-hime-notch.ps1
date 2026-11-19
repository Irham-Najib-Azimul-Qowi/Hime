Add-Type -ReferencedAssemblies 'System.Drawing', 'System.Windows.Forms' -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;

public class NotchVerification {
    [DllImport("user32.dll")]
    public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll")]
    public static extern IntPtr OpenInputDesktop(int dwFlags, bool fInherit, uint dwDesiredAccess);

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

    const uint MOUSEEVENTF_MOVE = 0x0001;
    const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
    const uint MOUSEEVENTF_LEFTUP = 0x0004;

    public static void MoveMouse(int x, int y) {
        Cursor.Position = new Point(x, y);
        mouse_event(MOUSEEVENTF_MOVE, 1, 0, 0, UIntPtr.Zero);
        Thread.Sleep(20);
        mouse_event(MOUSEEVENTF_MOVE, unchecked((uint)-1), 0, 0, UIntPtr.Zero);
        Thread.Sleep(50);
    }

    public static void Click(int x, int y) {
        MoveMouse(x, y);
        Thread.Sleep(120);
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, UIntPtr.Zero);
        Thread.Sleep(60);
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, UIntPtr.Zero);
        Thread.Sleep(100);
    }

    public static void Hover(int x, int y, int durationMs = 300) {
        MoveMouse(x, y);
        for (int i = 0; i < durationMs / 50; i++) {
            Thread.Sleep(50);
            mouse_event(MOUSEEVENTF_MOVE, 1, 0, 0, UIntPtr.Zero);
            mouse_event(MOUSEEVENTF_MOVE, unchecked((uint)-1), 0, 0, UIntPtr.Zero);
        }
    }

    public static void CaptureScreen(string outPath) {
        var bounds = Screen.PrimaryScreen.Bounds;
        using (Bitmap bmp = new Bitmap(bounds.Width, bounds.Height)) {
            using (Graphics g = Graphics.FromImage(bmp)) {
                IntPtr hdcDest = g.GetHdc();
                IntPtr hdcSrc = GetDC(IntPtr.Zero);
                BitBlt(hdcDest, 0, 0, bounds.Width, bounds.Height, hdcSrc, 0, 0, 0x40CC0020);
                ReleaseDC(IntPtr.Zero, hdcSrc);
                g.ReleaseHdc(hdcDest);
            }
            bmp.Save(outPath, ImageFormat.Png);
        }
    }

    public static void RunFullTest(string baseDir) {
        Thread t = new Thread(() => {
            try {
                IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
                if (hDesk == IntPtr.Zero) hDesk = OpenInputDesktop(0, false, 0x01FF);
                if (hDesk != IntPtr.Zero) SetThreadDesktop(hDesk);

                // 1. Initial State
                CaptureScreen(System.IO.Path.Combine(baseDir, "verify_step1_initial.png"));
                Thread.Sleep(300);

                // 2. Hover top edge at (683, 2) to trigger top-edge reveal
                Hover(683, 2, 400);
                CaptureScreen(System.IO.Path.Combine(baseDir, "verify_step2_hovered.png"));
                Thread.Sleep(200);

                // 3. Click the time center at (683, 16) to open Hime Assistant
                Click(683, 16);
                Thread.Sleep(800);
                CaptureScreen(System.IO.Path.Combine(baseDir, "verify_step3_hime_open.png"));

                // 4. Click prev arrow at (634, 16) to cycle to previous panel (Connected Devices)
                Click(634, 16);
                Thread.Sleep(800);
                CaptureScreen(System.IO.Path.Combine(baseDir, "verify_step4_prev_panel.png"));

                // 5. Click prev arrow at (634, 16) again to cycle to Command Center (Controls)
                Click(634, 16);
                Thread.Sleep(800);
                CaptureScreen(System.IO.Path.Combine(baseDir, "verify_step5_controls_panel.png"));

                // 6. Click next arrow at (732, 16) to cycle back toward Hime Assistant
                Click(732, 16);
                Thread.Sleep(800);
                CaptureScreen(System.IO.Path.Combine(baseDir, "verify_step6_next_panel.png"));

            } catch (Exception ex) {
                Console.WriteLine("Error: " + ex.Message);
            }
        });
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
    }
}
'@
[NotchVerification]::RunFullTest('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45')
