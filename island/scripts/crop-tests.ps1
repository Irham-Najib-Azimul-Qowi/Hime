Add-Type -AssemblyName System.Drawing
$artifactDir = "C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45"

function Crop-File($src, $dst) {
    if (Test-Path $src) {
        $bmp = [System.Drawing.Image]::FromFile($src)
        $crop = New-Object System.Drawing.Rectangle(450, 0, 466, 320)
        $cropped = New-Object System.Drawing.Bitmap(466, 320)
        $g = [System.Drawing.Graphics]::FromImage($cropped)
        $g.DrawImage($bmp, 0, 0, $crop, [System.Drawing.GraphicsUnit]::Pixel)
        $cropped.Save($dst, [System.Drawing.Imaging.ImageFormat]::Png)
        $g.Dispose()
        $cropped.Dispose()
        $bmp.Dispose()
        Write-Host "Cropped $src -> $dst"
    }
}

Crop-File "$artifactDir\test_autohide_clean.png" "$artifactDir\crop_autohide_clean.png"
Crop-File "$artifactDir\test_devices_panel.png" "$artifactDir\crop_devices_panel.png"
