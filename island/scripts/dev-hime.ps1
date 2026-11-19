<#
.SYNOPSIS
    Launches Hime Development Environment (Builds Core + Runs Shell safely via run-safe.ps1).
.PARAMETER Seconds
    Max watchdog duration in seconds (default 30).
.PARAMETER ScreenshotPath
    Optional path to capture screenshot before terminating.
.PARAMETER ExePath
    Optional custom executable path.
#>

[CmdletBinding()]
param(
    [int]$Seconds = 30,
    [string]$ScreenshotPath = "",
    [string]$ExePath = ""
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$buildCoreScript = Join-Path $scriptDir "build-core.ps1"
$runSafeScript = Join-Path $scriptDir "run-safe.ps1"
$restoreScript = Join-Path $scriptDir "restore-taskbar.ps1"

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  HIME x BLOOM - DEVELOPMENT ENVIRONMENT LAUNCHER" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# 1. Build Hime Core
Write-Host "[1/3] Building Hime Core..." -ForegroundColor Cyan
& powershell -ExecutionPolicy Bypass -File $buildCoreScript
if ($LASTEXITCODE -ne 0) {
    Write-Host "Core build failed. Aborting dev launch." -ForegroundColor Red
    exit 1
}

# Auto-detect executable if not provided: prefer standalone release build, fallback to debug
if (-not $ExePath) {
    $releaseExe = Join-Path $repoRoot "src-tauri\target\release\bloom.exe"
    $debugExe = Join-Path $repoRoot "src-tauri\target\debug\bloom.exe"
    if (Test-Path $releaseExe) {
        $ExePath = $releaseExe
    } elseif (Test-Path $debugExe) {
        $ExePath = $debugExe
    }
}

# Ensure hime-core.exe is placed beside the chosen bloom.exe
if ($ExePath -and (Test-Path $ExePath)) {
    $targetDir = Split-Path -Parent $ExePath
    $coreExe = Join-Path $repoRoot "core\target\debug\hime-core.exe"
    if (Test-Path $coreExe) {
        Copy-Item -Path $coreExe -Destination (Join-Path $targetDir "hime-core.exe") -Force -ErrorAction SilentlyContinue
    }
}

# 2. Ensure Vite frontend dev server is running on localhost:1420
$httpCode = ""
try {
    $httpCode = curl.exe -s -o NUL -w "%{http_code}" http://localhost:1420/
} catch {}

$localViteProc = $null
if ($httpCode -ne "200") {
    Write-Host "[2/3] Starting Vite dev server on http://localhost:1420..." -ForegroundColor Cyan
    $bunExe = "C:\Users\Satoshi\.bun\bin\bun.exe"
    if (Test-Path $bunExe) {
        $localViteProc = Start-Process -FilePath $bunExe -ArgumentList "run dev" -WorkingDirectory $repoRoot -WindowStyle Hidden -PassThru
    } else {
        $localViteProc = Start-Process -FilePath "bun" -ArgumentList "run dev" -WorkingDirectory $repoRoot -WindowStyle Hidden -PassThru
    }
    for ($i = 0; $i -lt 20; $i++) {
        Start-Sleep -Milliseconds 500
        try {
            $code = curl.exe -s -o NUL -w "%{http_code}" http://localhost:1420/
            if ($code -eq "200") {
                Write-Host "Vite dev server ready on http://localhost:1420" -ForegroundColor Green
                break
            }
        } catch {}
    }
} else {
    Write-Host "[2/3] Vite dev server already running on http://localhost:1420" -ForegroundColor Green
}

# 3. Launch Shell via run-safe.ps1
Write-Host "[3/3] Launching Shell under run-safe harness ($Seconds seconds timeout)..." -ForegroundColor Cyan
$safeArgs = @(
    "-ExecutionPolicy", "Bypass",
    "-File", $runSafeScript,
    "-Seconds", $Seconds
)
if ($ScreenshotPath) {
    $safeArgs += @("-ScreenshotPath", $ScreenshotPath)
}
if ($ExePath) {
    $safeArgs += @("-ExePath", $ExePath)
}

try {
    & powershell @safeArgs
} finally {
    if ($localViteProc -and (-not $localViteProc.HasExited)) {
        Stop-Process -Id $localViteProc.Id -Force -ErrorAction SilentlyContinue
    }
    Write-Host "Ensuring taskbar restoration..." -ForegroundColor Cyan
    & powershell -ExecutionPolicy Bypass -File $restoreScript
}
