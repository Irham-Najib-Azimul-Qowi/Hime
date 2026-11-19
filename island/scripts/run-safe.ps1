<#
.SYNOPSIS
    Runs Bloom / Hime Shell with strict watchdog timeout, screenshot capture, and guaranteed taskbar recovery.
.DESCRIPTION
    Mandatory safety harness for executing the shell in local dev or test.
    Ensures that under no circumstance the Windows taskbar is left hidden if Bloom crashes or times out.
#>

[CmdletBinding()]
param(
    [int]$Seconds = 15,
    [string]$ScreenshotPath = "",
    [string]$Command = "bun run tauri dev",
    [string]$ExePath = "",
    [switch]$NoDev
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$restoreScript = Join-Path $scriptDir "restore-taskbar.ps1"

if (Test-Path "C:\Users\Satoshi\.bun\bin") {
    $env:PATH = "C:\Users\Satoshi\.bun\bin;" + $env:PATH
}

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  HIME / BLOOM SAFE RUNNER (TIMEOUT: $Seconds s)" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

$proc = $null
try {
    $workDir = Split-Path -Parent $scriptDir
    if (-not $ExePath) {
        $rel = Join-Path $workDir "src-tauri\target\release\bloom.exe"
        if (Test-Path $rel) { $ExePath = $rel }
    }
    if ($ExePath -and -not (Test-Path $ExePath)) {
        $candidate = Join-Path $workDir $ExePath
        if (Test-Path $candidate) { $ExePath = $candidate }
    }
    $launcherCs = Join-Path $scriptDir "DesktopLauncher.cs"
    if (Test-Path $launcherCs) {
        Add-Type -Path $launcherCs -ErrorAction SilentlyContinue
    }

    if ($ExePath -and (Test-Path $ExePath)) {
        Write-Host "Launching executable on winsta0\Default: $ExePath" -ForegroundColor Green
        $targetPid = -1
        try {
            $targetPid = [DesktopLauncher]::LaunchOnDefaultDesktop($ExePath, "", $workDir)
        } catch {
            Write-Host "[WARN] DesktopLauncher invocation failed: $_" -ForegroundColor Yellow
        }

        if ($targetPid -gt 0) {
            $proc = [System.Diagnostics.Process]::GetProcessById($targetPid)
        } else {
            Write-Host "[WARN] Fallback to Start-Process" -ForegroundColor Yellow
            $proc = Start-Process -FilePath $ExePath -WorkingDirectory $workDir -PassThru
        }
    } else {
        Write-Host "Launching via command: $Command in $workDir" -ForegroundColor Green
        $parts = $Command -split ' ', 2
        $exe = $parts[0]
        $args = if ($parts.Length -gt 1) { $parts[1] } else { "" }
        $proc = Start-Process -FilePath $exe -ArgumentList $args -WorkingDirectory $workDir -PassThru
    }

    if ($Seconds -gt 0) {
        Write-Host "Process started with PID: $($proc.Id). Monitoring for $Seconds seconds..." -ForegroundColor Cyan
        $elapsed = 0
        $interval = 2
        while ($elapsed -lt $Seconds) {
            Start-Sleep -Seconds $interval
            $elapsed += $interval
            if ($proc.HasExited) {
                Write-Host "Process exited early with exit code: $($proc.ExitCode)" -ForegroundColor Yellow
                break
            }
        }
    } else {
        Write-Host "Process started with PID: $($proc.Id). Running continuously (Press Ctrl+C to stop)..." -ForegroundColor Cyan
        while (-not $proc.HasExited) {
            Start-Sleep -Seconds 1
        }
    }

    if ($ScreenshotPath -and (-not $proc.HasExited)) {
        Write-Host "Capturing verification screenshot to: $ScreenshotPath" -ForegroundColor Cyan
        try {
            $captureCs = Join-Path $scriptDir "DesktopCapture.cs"
            if (Test-Path $captureCs) {
                Add-Type -Path $captureCs -ReferencedAssemblies "System.Windows.Forms", "System.Drawing" -ErrorAction SilentlyContinue
            }
            $targetDir = Split-Path -Parent $ScreenshotPath
            if ($targetDir -and (-not (Test-Path $targetDir))) {
                New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
            }
            $captured = [DesktopCapture]::Capture($ScreenshotPath)
            if ($captured) {
                Write-Host "[OK] Desktop screenshot saved to $ScreenshotPath" -ForegroundColor Green
            } else {
                Write-Host "[WARN] Desktop capture returned false" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "[WARN] Screenshot capture failed: $_" -ForegroundColor Yellow
        }
    }
}
finally {
    Write-Host "Initiating guaranteed shutdown and taskbar restoration..." -ForegroundColor Yellow
    if ($proc -and (-not $proc.HasExited)) {
        Write-Host "Stopping target process PID $($proc.Id)..." -ForegroundColor Yellow
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }

    if (Test-Path $restoreScript) {
        & powershell -ExecutionPolicy Bypass -File $restoreScript
    } else {
        Write-Host "[ERROR] Could not find restore-taskbar.ps1 at $restoreScript" -ForegroundColor Red
    }
}

Write-Host "Safe run completed." -ForegroundColor Green

