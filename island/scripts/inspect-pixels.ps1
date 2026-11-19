Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_bottom_live.png')
# Check row by row from top of crop (y=668) to bottom (y=767)
Write-Output "Sampling vertical line at x=500 in bottom crop (100px high):"
for ($y = 0; $y -lt 100; $y += 10) {
    $c = $img.GetPixel(500, $y)
    Write-Output "y=$($y+668): R=$($c.R), G=$($c.G), B=$($c.B)"
}
$img.Dispose()
