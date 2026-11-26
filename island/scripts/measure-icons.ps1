Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_hover_smooth.png')

# Measure bounds of 1st Chrome (around X=360..400)
# Measure bounds of 2nd Chrome (around X=480..520)
$minX1 = 999; $maxX1 = 0; $minY1 = 999; $maxY1 = 0
$minX2 = 999; $maxX2 = 0; $minY2 = 999; $maxY2 = 0

for ($y = 0; $y -lt $img.Height; $y++) {
    for ($x = 0; $x -lt $img.Width; $x++) {
        $c = $img.GetPixel($x, $y)
        # Check Chrome green (#34a853 approx R<100, G>140, B<100)
        if ($c.R -lt 100 -and $c.G -gt 150 -and $c.B -lt 100) {
            if ($x -lt 450) {
                if ($x -lt $minX1) { $minX1 = $x }
                if ($x -gt $maxX1) { $maxX1 = $x }
                if ($y -lt $minY1) { $minY1 = $y }
                if ($y -gt $maxY1) { $maxY1 = $y }
            } else {
                if ($x -lt $minX2) { $minX2 = $x }
                if ($x -gt $maxX2) { $maxX2 = $x }
                if ($y -lt $minY2) { $minY2 = $y }
                if ($y -gt $maxY2) { $maxY2 = $y }
            }
        }
    }
}
Write-Host "Chrome 1: Width=$($maxX1 - $minX1 + 1), Height=$($maxY1 - $minY1 + 1), Y=$minY1..$maxY1"
Write-Host "Chrome 2: Width=$($maxX2 - $minX2 + 1), Height=$($maxY2 - $minY2 + 1), Y=$minY2..$maxY2"
$img.Dispose()
