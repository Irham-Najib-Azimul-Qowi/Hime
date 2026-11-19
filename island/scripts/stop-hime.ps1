<#
.SYNOPSIS
    Convenient 1-click script to stop Hime and restore the Windows taskbar.
#>

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$restoreScript = Join-Path $scriptDir "restore-taskbar.ps1"

Write-Host "Menghentikan Hime dan memulihkan taskbar Windows..." -ForegroundColor Cyan
& powershell -ExecutionPolicy Bypass -File $restoreScript
Write-Host "Selesai! Taskbar Windows telah kembali normal." -ForegroundColor Green
