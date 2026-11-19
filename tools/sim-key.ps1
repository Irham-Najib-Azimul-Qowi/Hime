$src = @'
using System;
using System.Runtime.InteropServices;
using System.Threading;
public class TestSimulator {
    [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
    public const int KEYEVENTF_KEYUP = 0x0002;
    public const byte VK_CONTROL = 0x11;
    public const byte VK_SHIFT = 0x10;
    public const byte VK_H = 0x48;
    public static void PressCtrlShiftH() {
        keybd_event(VK_CONTROL, 0, 0, UIntPtr.Zero);
        keybd_event(VK_SHIFT, 0, 0, UIntPtr.Zero);
        keybd_event(VK_H, 0, 0, UIntPtr.Zero);
        Thread.Sleep(50);
        keybd_event(VK_H, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        keybd_event(VK_SHIFT, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
    }
}
'@
Add-Type -TypeDefinition $src
[TestSimulator]::PressCtrlShiftH()
Write-Host "Simulated Ctrl+Shift+H successfully sent."
