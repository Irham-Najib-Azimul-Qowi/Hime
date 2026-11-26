Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class HumanMouse {
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll")] public static extern bool SetThreadDesktop(IntPtr hDesktop);
}
"@

$h = [HumanMouse]::OpenDesktop("Default", 0, $false, 0x01FF)
if ($h -ne [IntPtr]::Zero) { [HumanMouse]::SetThreadDesktop($h) }

# 1. Approach bottom edge to arm dwell
[HumanMouse]::SetCursorPos(683, 766)
[HumanMouse]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 100
[HumanMouse]::SetCursorPos(683, 767)
[HumanMouse]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 250
[HumanMouse]::SetCursorPos(683, 766)
[HumanMouse]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 300

# 2. Move up into dock towards Chrome icon (X=608, Y=738)
for ($y = 765; $y -ge 738; $y -= 5) {
    [HumanMouse]::SetCursorPos(608, $y)
    [HumanMouse]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 30
}
[HumanMouse]::SetCursorPos(608, 738)
[HumanMouse]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 300

$cs = 'X:\folder_ai\Hime-Bloom\scripts\DesktopCapture.cs'
Add-Type -Path $cs -ReferencedAssemblies 'System.Windows.Forms', 'System.Drawing'
[DesktopCapture]::Capture('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\test_hover_smooth.png')

Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Image]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\test_hover_smooth.png')
$w = $img.Width
$h = $img.Height
$cropW = 900
$cropH = 140
$cropX = [int](($w - $cropW) / 2)
$cropY = $h - $cropH
$rect = New-Object System.Drawing.Rectangle($cropX, $cropY, $cropW, $cropH)
$bmp = New-Object System.Drawing.Bitmap($cropW, $cropH)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.DrawImage($img, (New-Object System.Drawing.Rectangle(0, 0, $cropW, $cropH)), $rect, [System.Drawing.GraphicsUnit]::Pixel)
$bmp.Save('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_hover_smooth.png', [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
$img.Dispose()
Write-Host 'Smooth human hover completed'
