Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class W {
    [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out int v, int s);
    public struct RECT { public int L, T, R, B; }
}
"@

foreach ($h in @(0x3E0694, 0x2C08DC)) {
    $ptr = [IntPtr]$h
    $isW = [W]::IsWindow($ptr)
    $vis = [W]::IsWindowVisible($ptr)
    $r = New-Object W+RECT
    [W]::GetWindowRect($ptr, [ref]$r)
    $c = 0
    [W]::DwmGetWindowAttribute($ptr, 14, [ref]$c, 4)
    Write-Host ("HWND 0x{0:X}: Valid={1}, Vis={2}, Cloaked={3}, Rect=({4},{5})-({6},{7})" -f $h, $isW, $vis, $c, $r.L, $r.T, $r.R, $r.B)
}
