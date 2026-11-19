$ErrorActionPreference = "Continue"
$proc = Start-Process -FilePath "island\src-tauri\target\release\hime-island.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 2
Write-Host "PID: $($proc.Id) | Responding: $($proc.Responding) | HasExited: $($proc.HasExited)"
if ($proc.HasExited) {
    Write-Host "ExitCode: $($proc.ExitCode)"
}
