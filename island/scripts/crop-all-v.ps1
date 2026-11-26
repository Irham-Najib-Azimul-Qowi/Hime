Add-Type -AssemblyName System.Drawing

function Crop-Notch($file, $outFile, $h = 420) {
    $img = [System.Drawing.Bitmap]::FromFile("C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\$file")
    $cx = [int]($img.Width / 2)
    $crop = New-Object System.Drawing.Bitmap 480, $h
    $g = [System.Drawing.Graphics]::FromImage($crop)
    $g.DrawImage($img, (New-Object System.Drawing.Rectangle 0, 0, 480, $h), (New-Object System.Drawing.Rectangle ($cx - 240), 0, 480, $h), [System.Drawing.GraphicsUnit]::Pixel)
    $crop.Save("C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\$outFile", [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $crop.Dispose()
    $img.Dispose()
    Write-Output "Cropped $file -> $outFile"
}

Crop-Notch "verify_step1_initial.png" "crop_v1_initial.png" 60
Crop-Notch "verify_step2_hovered.png" "crop_v2_hovered.png" 60
Crop-Notch "verify_step3_hime_open.png" "crop_v3_hime.png" 420
Crop-Notch "verify_step5_controls_panel.png" "crop_v5_controls.png" 300
