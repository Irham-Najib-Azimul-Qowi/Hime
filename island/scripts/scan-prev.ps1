Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\.user_uploaded\media_1794831108076_118f5ed9.jpg')
Write-Output "Image dims: $($img.Width)x$($img.Height)"
$cx = [int]($img.Width / 2)
# Scan y from 0 to 60 at cx
for ($y = 0; $y -lt 60; $y += 2) {
    $line = ""
    for ($x = ($cx - 100); $x -lt ($cx + 100); $x += 2) {
        $c = $img.GetPixel($x, $y)
        if ($c.R -gt 150) { $line += "#" } else { $line += " " }
    }
    if ($line.Trim().Length -gt 0) {
        Write-Output ("y={0:D2}: {1}" -f $y, $line)
    }
}
$img.Dispose()
