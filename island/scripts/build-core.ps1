<#
.SYNOPSIS
    Builds the standalone Hime Core binary from core/ workspace.
.PARAMETER Release
    Compiles with --release optimizations.
#>

[CmdletBinding()]
param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$coreDir = Join-Path $repoRoot "core"

Write-Host "Building Hime Core workspace at $coreDir..." -ForegroundColor Cyan

Push-Location $coreDir
try {
    $cargoArgs = @("build", "--workspace", "--locked")
    if ($Release) {
        $cargoArgs += "--release"
    }

    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }

    $profile = if ($Release) { "release" } else { "debug" }
    $binaryPath = Join-Path $coreDir "target\$profile\hime-core.exe"
    if (Test-Path $binaryPath) {
        Write-Host "Hime Core successfully built: $binaryPath" -ForegroundColor Green
    } else {
        Write-Host "Core workspace built successfully (libraries compiled)." -ForegroundColor Green
    }
} finally {
    Pop-Location
}
