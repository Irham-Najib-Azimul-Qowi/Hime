Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$artifactDir = "C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45"

# Capture current screen
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap($screen.Width, $screen.Height)
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
$gfx.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)

# Crop top notch region (x: 450 to 916, y: 0 to 400)
$cropRect = New-Object System.Drawing.Rectangle(450, 0, 466, 400)
$cropped = $bmp.Clone($cropRect, $bmp.PixelFormat)
$cropped.Save("$artifactDir\live_test_notch_crop.png", [System.Drawing.Imaging.ImageFormat]::Png)

$gfx.Dispose()
$bmp.Dispose()
$cropped.Dispose()
Write-Host "Captured live notch crop successfully."
