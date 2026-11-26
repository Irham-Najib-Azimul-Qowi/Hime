Add-Type -AssemblyName System.Drawing

$img = [System.Drawing.Image]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\current_live_desktop.png')
$w = $img.Width
$h = $img.Height
Write-Output "Image size: ${w}x${h}"

# Crop top notch area (center 500px, top 100px)
$cropTop = New-Object System.Drawing.Bitmap 600, 150
$gTop = [System.Drawing.Graphics]::FromImage($cropTop)
$srcX = [int](($w - 600) / 2)
$gTop.DrawImage($img, (New-Object System.Drawing.Rectangle 0, 0, 600, 150), (New-Object System.Drawing.Rectangle $srcX, 0, 600, 150), [System.Drawing.GraphicsUnit]::Pixel)
$cropTop.Save('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_top_live.png', [System.Drawing.Imaging.ImageFormat]::Png)
$gTop.Dispose()
$cropTop.Dispose()

# Crop bottom area (full width, bottom 100px)
$cropBottom = New-Object System.Drawing.Bitmap $w, 100
$gBottom = [System.Drawing.Graphics]::FromImage($cropBottom)
$gBottom.DrawImage($img, (New-Object System.Drawing.Rectangle 0, 0, $w, 100), (New-Object System.Drawing.Rectangle 0, ($h - 100), $w, 100), [System.Drawing.GraphicsUnit]::Pixel)
$cropBottom.Save('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_bottom_live.png', [System.Drawing.Imaging.ImageFormat]::Png)
$gBottom.Dispose()
$cropBottom.Dispose()

$img.Dispose()
Write-Output "Saved crop_top_live.png and crop_bottom_live.png"
