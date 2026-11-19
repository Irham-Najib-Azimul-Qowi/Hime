$src = @'
using System;
using System.Runtime.InteropServices;
using System.Threading;

public class KeySim2 {
    [DllImport("user32.dll", SetLastError = true)]
    static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    [DllImport("user32.dll")]
    static extern uint MapVirtualKey(uint uCode, uint uMapType);

    const int INPUT_KEYBOARD = 1;
    const uint KEYEVENTF_KEYUP = 0x0002;

    [StructLayout(LayoutKind.Explicit, Size = 40)]
    struct INPUT {
        [FieldOffset(0)] public int type;
        [FieldOffset(8)] public KEYBDINPUT ki;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct KEYBDINPUT {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    public static void SendCtrlShiftH() {
        ushort vkCtrl = 0x11;
        ushort vkShift = 0x10;
        ushort vkH = 0x48;

        INPUT[] inputs = new INPUT[6];

        // Down
        inputs[0].type = INPUT_KEYBOARD;
        inputs[0].ki.wVk = vkCtrl;
        inputs[0].ki.wScan = (ushort)MapVirtualKey(vkCtrl, 0);

        inputs[1].type = INPUT_KEYBOARD;
        inputs[1].ki.wVk = vkShift;
        inputs[1].ki.wScan = (ushort)MapVirtualKey(vkShift, 0);

        inputs[2].type = INPUT_KEYBOARD;
        inputs[2].ki.wVk = vkH;
        inputs[2].ki.wScan = (ushort)MapVirtualKey(vkH, 0);

        // Up
        inputs[3].type = INPUT_KEYBOARD;
        inputs[3].ki.wVk = vkH;
        inputs[3].ki.wScan = (ushort)MapVirtualKey(vkH, 0);
        inputs[3].ki.dwFlags = KEYEVENTF_KEYUP;

        inputs[4].type = INPUT_KEYBOARD;
        inputs[4].ki.wVk = vkShift;
        inputs[4].ki.wScan = (ushort)MapVirtualKey(vkShift, 0);
        inputs[4].ki.dwFlags = KEYEVENTF_KEYUP;

        inputs[5].type = INPUT_KEYBOARD;
        inputs[5].ki.wVk = vkCtrl;
        inputs[5].ki.wScan = (ushort)MapVirtualKey(vkCtrl, 0);
        inputs[5].ki.dwFlags = KEYEVENTF_KEYUP;

        uint sent = SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT)));
        int err = Marshal.GetLastWin32Error();
        Console.WriteLine("SendInput sent: " + sent + " (err: " + err + ")");
    }
}
'@

Add-Type -TypeDefinition $src
[KeySim2]::SendCtrlShiftH()
