Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class MouseMoveTest {
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern IntPtr OpenDesktop(string lpszDesktop, int dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll")] public static extern bool SetThreadDesktop(IntPtr hDesktop);
}
"@

# Switch thread desktop if needed
$h = [MouseMoveTest]::OpenDesktop("Default", 0, $false, 0x01FF)
if ($h -ne [IntPtr]::Zero) { [MouseMoveTest]::SetThreadDesktop($h) }

# Move mouse to bottom edge (X=683, Y=766)
[MouseMoveTest]::SetCursorPos(683, 766)
[MouseMoveTest]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 100
[MouseMoveTest]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
# Wait for dwell (200ms) + animation (400ms)
Start-Sleep -Milliseconds 600

$cs = 'X:\folder_ai\Hime-Bloom\scripts\DesktopCapture.cs'
Add-Type -Path $cs -ReferencedAssemblies 'System.Windows.Forms', 'System.Drawing'
[DesktopCapture]::Capture('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\test_hover_dock_edge.png')

Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Image]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\test_hover_dock_edge.png')
$w = $img.Width
$h = $img.Height
$cropW = 900
$cropH = 120
$cropX = [int](($w - $cropW) / 2)
$cropY = $h - $cropH
$rect = New-Object System.Drawing.Rectangle($cropX, $cropY, $cropW, $cropH)
$bmp = New-Object System.Drawing.Bitmap($cropW, $cropH)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.DrawImage($img, (New-Object System.Drawing.Rectangle(0, 0, $cropW, $cropH)), $rect, [System.Drawing.GraphicsUnit]::Pixel)
$bmp.Save('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_hover_dock_edge.png', [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
$img.Dispose()
Write-Host 'Hover test completed'
