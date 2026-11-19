Add-Type -Path "X:\folder_ai\Hime-Bloom\scripts\DesktopLauncher.cs"
$targetPid = [DesktopLauncher]::LaunchOnDefaultDesktop("X:\folder_ai\Hime-Bloom\src-tauri\target\release\bloom.exe", "", "X:\folder_ai\Hime-Bloom")
Write-Output "Started bloom.exe PID: $targetPid"
Start-Sleep -Seconds 3
Get-Process bloom, hime-core -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, MainWindowTitle
