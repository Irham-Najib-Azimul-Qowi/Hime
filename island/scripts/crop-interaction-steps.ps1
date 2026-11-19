Add-Type -AssemblyName System.Drawing

$dir = "C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45"
$steps = @("step1_initial_notch", "step2_opened_card", "step3_after_hover_leave", "step4_reopened_card", "step5_after_click_outside")

foreach ($s in $steps) {
    $src = "$dir\$s.png"
    if (Test-Path $src) {
        $bmp = [System.Drawing.Bitmap]::FromFile($src)
        $rect = [System.Drawing.Rectangle]::FromLTRB(450, 0, 916, 450)
        $crop = $bmp.Clone($rect, $bmp.PixelFormat)
        $bmp.Dispose()
        $out = "$dir\crop_$s.png"
        $crop.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
        $crop.Dispose()
        Write-Host "Cropped $s to $out"
    }
}
