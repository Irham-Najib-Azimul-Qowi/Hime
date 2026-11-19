Set-Location 'X:\folder_ai\Hime-Bloom'
$p = Start-Process -FilePath 'src-tauri\target\release\bloom.exe' -NoNewWindow -PassThru -RedirectStandardError 'stderr.txt' -RedirectStandardOutput 'stdout.txt'
Start-Sleep -Seconds 3
if ($p.HasExited) {
    Write-Output "Exited with code: $($p.ExitCode)"
} else {
    Write-Output "Still running PID: $($p.Id)"
    Stop-Process -Id $p.Id -Force
}
if (Test-Path 'stderr.txt') { Get-Content 'stderr.txt' -Tail 20 }
if (Test-Path 'stdout.txt') { Get-Content 'stdout.txt' -Tail 20 }
