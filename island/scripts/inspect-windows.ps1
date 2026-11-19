Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public class WinInspector {
    [DllImport("user32.dll")]
    public static extern bool IsWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern int GetWindowLong(IntPtr hWnd, int nIndex);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")]
    public static extern bool GetLayeredWindowAttributes(IntPtr hwnd, out uint pcrKey, out byte pbAlpha, out uint pdwFlags);
    [DllImport("dwmapi.dll")]
    public static extern int DwmGetWindowAttribute(IntPtr hwnd, int dwAttribute, out int pvAttribute, int cbAttribute);

    public struct RECT { public int Left, Top, Right, Bottom; }

    public static void Inspect(IntPtr hwnd, string name) {
        if (!IsWindow(hwnd)) {
            Console.WriteLine(name + " is not a valid window.");
            return;
        }
        bool visible = IsWindowVisible(hwnd);
        int style = GetWindowLong(hwnd, -16);
        int exStyle = GetWindowLong(hwnd, -20);
        RECT r;
        GetWindowRect(hwnd, out r);
        uint key = 0, flags = 0;
        byte alpha = 0;
        bool hasLayered = GetLayeredWindowAttributes(hwnd, out key, out alpha, out flags);
        int cloaked = 0;
        DwmGetWindowAttribute(hwnd, 14, out cloaked, 4);

        Console.WriteLine(name + " (HWND: 0x" + hwnd.ToString("X") + "):");
        Console.WriteLine("  Visible: " + visible);
        Console.WriteLine("  Cloaked: " + cloaked);
        Console.WriteLine("  Rect: L=" + r.Left + ", T=" + r.Top + ", R=" + r.Right + ", B=" + r.Bottom + " (W=" + (r.Right - r.Left) + ", H=" + (r.Bottom - r.Top) + ")");
        Console.WriteLine("  Style: 0x" + style.ToString("X8"));
        Console.WriteLine("  ExStyle: 0x" + exStyle.ToString("X8"));
        Console.WriteLine("  WS_EX_LAYERED: " + ((exStyle & 0x00080000) != 0));
        Console.WriteLine("  WS_EX_TRANSPARENT: " + ((exStyle & 0x00000020) != 0));
        Console.WriteLine("  WS_EX_TOPMOST: " + ((exStyle & 0x00000008) != 0));
        Console.WriteLine("  WS_EX_TOOLWINDOW: " + ((exStyle & 0x00000080) != 0));
        Console.WriteLine("  LayeredAttrs: success=" + hasLayered + ", alpha=" + alpha + ", flags=0x" + flags.ToString("X"));
    }
}
"@

$p = Get-Process -Name "bloom" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $p) {
    Write-Host "No bloom process found."
    exit 0
}

Write-Host "Bloom PID: $($p.Id)"

# Find all windows belonging to Bloom
$hwnds = Get-Process -Id $p.Id | Select-Object -ExpandProperty MainWindowHandle
Write-Host "MainWindowHandle: 0x$($hwnds.ToString('X'))"

# Read diagnostics to get exact hwnds
$log = Get-Content "$env:LOCALAPPDATA\com.sehaz.bloom\diagnostics.log" -Tail 50
$mainHwnd = ($log | Select-String "main_init hwnd=(0x[0-9a-fA-F]+)" | Select-Object -Last 1).Matches.Groups[1].Value
$dockHwnd = ($log | Select-String "dock_init hwnd=(0x[0-9a-fA-F]+)" | Select-Object -Last 1).Matches.Groups[1].Value

if ($mainHwnd) {
    [WinInspector]::Inspect([IntPtr][Convert]::ToInt64($mainHwnd, 16), "Main Notch Window")
}
if ($dockHwnd) {
    [WinInspector]::Inspect([IntPtr][Convert]::ToInt64($dockHwnd, 16), "Dock Window")
}
