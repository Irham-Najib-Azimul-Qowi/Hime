Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\notch_crop_debug.png')
Write-Output "Size: $($img.Width)x$($img.Height)"
$cx = [int]($img.Width / 2)
Write-Output "Sampling at cx=$cx from y=0 to y=60:"
for ($y = 0; $y -lt 60; $y += 4) {
    $c = $img.GetPixel($cx, $y)
    Write-Output "y=${y}: R=$($c.R), G=$($c.G), B=$($c.B)"
}
$img.Dispose()
