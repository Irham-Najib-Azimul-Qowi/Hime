Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class MouseMoveDock {
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extra);
}
"@

# Position on dock pill (center bottom)
[MouseMoveDock]::SetCursorPos(683, 745)
[MouseMoveDock]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 500

& "X:\folder_ai\Hime-Bloom\scripts\capture-now.ps1"
