Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\crop_hover_dock_edge.png')
# Search around Y=50..100 in the crop
for ($y = 40; $y -lt 110; $y += 5) {
    for ($x = 250; $x -lt 650; $x += 10) {
        $c = $img.GetPixel($x, $y)
        # Check for vibrant colors (Chrome yellow/red/green or Windows blue or folder yellow)
        if ($c.R -gt 200 -and $c.G -gt 150 -and $c.B -lt 50) { # Yellow
            Write-Host "Yellow at X=$x, Y=$y ($($c.R), $($c.G), $($c.B))"
        }
        if ($c.R -lt 50 -and $c.G -gt 150 -and $c.B -gt 200) { # Cyan / Blue
            Write-Host "Blue at X=$x, Y=$y ($($c.R), $($c.G), $($c.B))"
        }
    }
}
$img.Dispose()
