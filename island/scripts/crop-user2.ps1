Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\.user_uploaded\media_1794876832438_55b4affa.jpg')
$cx = [int]($img.Width / 2)
$crop = New-Object System.Drawing.Bitmap 300, 80
$g = [System.Drawing.Graphics]::FromImage($crop)
$g.DrawImage($img, (New-Object System.Drawing.Rectangle 0, 0, 300, 80), (New-Object System.Drawing.Rectangle ($cx - 150), 0, 300, 80), [System.Drawing.GraphicsUnit]::Pixel)
$crop.Save('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\user_top_crop2.png', [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$crop.Dispose()
$img.Dispose()
Write-Output "Saved user_top_crop2.png"
