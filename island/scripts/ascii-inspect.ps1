Add-Type -AssemblyName System.Drawing

function Show-Ascii($path) {
    Write-Output "=== $path ==="
    $img = [System.Drawing.Bitmap]::FromFile($path)
    for ($y = 0; $y -lt 40; $y += 2) {
        $line = ""
        for ($x = 100; $x -lt 200; $x += 2) {
            $c = $img.GetPixel($x, $y)
            if ($c.R -gt 180) {
                $line += "#"
            } elseif ($c.R -gt 80) {
                $line += "."
            } else {
                $line += " "
            }
        }
        Write-Output ("{0:D2}: {1}" -f $y, $line)
    }
    $img.Dispose()
}

Show-Ascii('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\user_top_crop.png')
Show-Ascii('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\user_top_crop2.png')
