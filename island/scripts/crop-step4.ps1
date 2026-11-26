Add-Type -AssemblyName System.Drawing
$artifactDir = "C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45"

$step4 = "$artifactDir\verify_step4_prev_panel.png"
if (Test-Path $step4) {
    $bmp = [System.Drawing.Image]::FromFile($step4)
    $crop = New-Object System.Drawing.Rectangle(450, 0, 466, 380)
    $cropped = New-Object System.Drawing.Bitmap(466, 380)
    $g = [System.Drawing.Graphics]::FromImage($cropped)
    $g.DrawImage($bmp, 0, 0, $crop, [System.Drawing.GraphicsUnit]::Pixel)
    $cropped.Save("$artifactDir\crop_v4_devices.png", [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $cropped.Dispose()
    $bmp.Dispose()
    Write-Host "Cropped verify_step4_prev_panel.png -> crop_v4_devices.png"
}
