<#
.SYNOPSIS
    Idempotent taskbar and explorer recovery script for Hime Ecosystem.
.DESCRIPTION
    Restores the Windows native taskbar (Shell_TrayWnd and Shell_SecondaryTrayWnd),
    unregisters any lingering AppBars, terminates runaway shell processes, and restarts
    Windows Explorer if required. Non-admin safe.
#>

[CmdletBinding()]
param(
    [switch]$ForceRestartExplorer
)

$islandScript = Join-Path $PSScriptRoot "..\island\scripts\restore-taskbar.ps1"
if (Test-Path $islandScript) {
    & $islandScript @PSBoundParameters
    exit $LASTEXITCODE
}

Write-Host "Taskbar recovery fallback..." -ForegroundColor Cyan
