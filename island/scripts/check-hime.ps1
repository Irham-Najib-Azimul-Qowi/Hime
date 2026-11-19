Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\verify_step3_hime_open.png')
$cx = [int]($img.Width / 2)
$hasWhite = $false
for ($y = 0; $y -lt 36; $y++) {
    for ($x = ($cx - 50); $x -lt ($cx + 50); $x += 2) {
        $c = $img.GetPixel($x, $y)
        if ($c.R -gt 200 -and $c.G -gt 200 -and $c.B -gt 200) {
            $hasWhite = $true
            break
        }
    }
    if ($hasWhite) { break }
}
Write-Output "Has white text in top 36px in hime: $hasWhite"
$img.Dispose()
