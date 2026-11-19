Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class WCheck {
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    public struct RECT { public int Left, Top, Right, Bottom; }
    public static void Show(IntPtr hwnd) {
        RECT r;
        GetWindowRect(hwnd, out r);
        Console.WriteLine(string.Format("L={0}, T={1}, R={2}, B={3}, W={4}, H={5}", r.Left, r.Top, r.Right, r.Bottom, r.Right-r.Left, r.Bottom-r.Top));
    }
}
'@
$log = Get-Content "$env:LOCALAPPDATA\com.sehaz.bloom\diagnostics.log" -Tail 30
$m = $log | Select-String "main_init hwnd=(0x[0-9a-fA-F]+)" | Select-Object -Last 1
$mainHwnd = $m.Matches.Groups[1].Value
Write-Host "Main HWND: $mainHwnd"
[WCheck]::Show([IntPtr][Convert]::ToInt64($mainHwnd, 16))
