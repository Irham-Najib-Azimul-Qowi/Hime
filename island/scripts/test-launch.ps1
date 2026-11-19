Add-Type -Path X:\folder_ai\Hime-Bloom\scripts\DesktopLauncher.cs
$procId = [DesktopLauncher]::LaunchOnDefaultDesktop("C:\Windows\System32\cmd.exe", "/c ping 127.0.0.1 -n 5", "C:\Windows\System32")
Write-Host "Launched CMD PID: $procId"
if ($procId -gt 0) {
    Start-Sleep -Milliseconds 500
    Get-Process -Id $procId -ErrorAction SilentlyContinue
    Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
}
