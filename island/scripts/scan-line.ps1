Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\.user_uploaded\media_1794876187122_ee4b3a7c.jpg')
Write-Output "Image dims: $($img.Width)x$($img.Height)"
$cx = [int]($img.Width / 2)
# Check every 2 pixels from cx - 200 to cx + 200 at y=16
$line = ""
for ($x = ($cx - 150); $x -lt ($cx + 150); $x += 2) {
    $c = $img.GetPixel($x, 16)
    if ($c.R -gt 150) { $line += "#" } else { $line += " " }
}
Write-Output "y=16 scan: '$line'"
$img.Dispose()
