$cs = 'X:\folder_ai\Hime-Bloom\scripts\DesktopCapture.cs'
Add-Type -Path $cs -ReferencedAssemblies 'System.Windows.Forms', 'System.Drawing'

# We can reuse MouseSim from test-reveal-dock.ps1 if loaded, or use System.Windows.Forms.Cursor
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class MouseMover {
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
}
"@

# Move cursor directly onto the top hidden line (X=683, Y=2)
[MouseMover]::SetCursorPos(683, 2)
[MouseMover]::mouse_event(1, 0, 1, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 400

[DesktopCapture]::Capture('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\test_hover_notch_line.png')

Add-Type -AssemblyName System.Drawing
$src = 'C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\test_hover_notch_line.png'
$img = [System.Drawing.Image]::FromFile($src)
$w = $img.Width
$cropW = 500
$cropH = 60
$cropX = [int](($w - $cropW) / 2)
$cropY = 0
$rect = New-Object System.Drawing.Rectangle($cropX, $cropY, $cropW, $cropH)
$bmp = New-Object System.Drawing.Bitmap($cropW, $cropH)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.DrawImage($img, (New-Object System.Drawing.Rectangle(0, 0, $cropW, $cropH)), $rect, [System.Drawing.GraphicsUnit]::Pixel)
$bmp.Save('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_hover_notch_line.png', [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
$img.Dispose()
Write-Host 'Hovered and cropped'
