Add-Type -AssemblyName System.Windows.Forms
$s = [System.Windows.Forms.Screen]::PrimaryScreen
Write-Output "Bounds: $($s.Bounds.Width)x$($s.Bounds.Height) at ($($s.Bounds.X),$($s.Bounds.Y))"
Write-Output "WorkingArea: $($s.WorkingArea.Width)x$($s.WorkingArea.Height) at ($($s.WorkingArea.X),$($s.WorkingArea.Y))"
