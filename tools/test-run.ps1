$proc = Start-Process -FilePath "X:\folder_ai\Hime\island\src-tauri\target\release\hime-island.exe" -RedirectStandardError "X:\folder_ai\Hime\err.log" -RedirectStandardOutput "X:\folder_ai\Hime\out.log" -Wait -PassThru
Write-Host "ExitCode: $($proc.ExitCode)"
if (Test-Path "X:\folder_ai\Hime\err.log") {
    Write-Host "--- STDERR ---"
    Get-Content "X:\folder_ai\Hime\err.log"
}
if (Test-Path "X:\folder_ai\Hime\out.log") {
    Write-Host "--- STDOUT ---"
    Get-Content "X:\folder_ai\Hime\out.log"
}
