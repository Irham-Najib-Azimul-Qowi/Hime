Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\verify_step5_controls_panel.png')
Write-Output "Size: $($img.Width)x$($img.Height)"
$cx = [int]($img.Width / 2)
# Crop center 400px, top 120px
$crop = New-Object System.Drawing.Bitmap 400, 120
$g = [System.Drawing.Graphics]::FromImage($crop)
$g.DrawImage($img, (New-Object System.Drawing.Rectangle 0, 0, 400, 120), (New-Object System.Drawing.Rectangle ($cx - 200), 0, 400, 120), [System.Drawing.GraphicsUnit]::Pixel)
$crop.Save('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\controls_crop.png', [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$crop.Dispose()
$img.Dispose()
Write-Output "Saved controls_crop.png"
