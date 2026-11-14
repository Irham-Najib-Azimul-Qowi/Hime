<#
.SYNOPSIS
    Hime Local Development Environment Launcher.
.DESCRIPTION
    Launches the Hime development environment, configuring sandboxed local
    development paths under %LOCALAPPDATA%\Hime\dev\ and supervising background
    runtimes (Hime Core debug binary and Tauri desktop frontend).
    Implements a strict clean-shutdown trap (Zero Orphan Processes) on Ctrl+C.
#>

[CmdletBinding()]
param (
    [switch]$CoreOnly,
    [switch]$DesktopOnly,
    [string]$Port = "20128"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Augment process PATH with default user toolchain directories if present
$userCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $userCargoBin) -and ($env:PATH -notlike "*$userCargoBin*")) {
    $env:PATH = "$userCargoBin;$env:PATH"
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  HIME LOCAL DEVELOPMENT ENVIRONMENT LAUNCHER" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

$workspaceRoot = Split-Path -Parent $PSScriptRoot
Set-Location $workspaceRoot

# -----------------------------------------------------------------------------
# 1. Prerequisite Checks
# -----------------------------------------------------------------------------
Write-Host "[1/4] Checking Development Prerequisites..." -ForegroundColor Yellow

$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
$pnpmCmd  = Get-Command pnpm -ErrorAction SilentlyContinue

if (-not $cargoCmd) {
    Write-Host "[FAIL] Cargo compiler toolchain is missing." -ForegroundColor Red
    Write-Host "Cannot run development environment without Cargo." -ForegroundColor Red
    Write-Host "Run .\scripts\check-env.ps1 for installation guidance." -ForegroundColor Yellow
    exit 1
}

# -----------------------------------------------------------------------------
# 2. Local Development Sandbox Paths Initialization
# -----------------------------------------------------------------------------
Write-Host "[2/4] Initializing Local Development Sandbox..." -ForegroundColor Yellow

$devRoot = Join-Path $env:LOCALAPPDATA "Hime\dev"
$subdirs = @("config", "state", "logs", "tmp", "secrets", "fixtures")

foreach ($d in $subdirs) {
    $targetPath = Join-Path $devRoot $d
    if (-not (Test-Path $targetPath)) {
        New-Item -ItemType Directory -Path $targetPath -Force | Out-Null
    }
}

Write-Host "  Sandbox Root : $devRoot" -ForegroundColor Cyan
Write-Host "  State Dir    : $(Join-Path $devRoot 'state')" -ForegroundColor Cyan
Write-Host "  Logs Dir     : $(Join-Path $devRoot 'logs')" -ForegroundColor Cyan

# Set environment variables for the current session
$env:HIME_ENV = "development"
$env:HIME_DATA_DIR = $devRoot
$env:HIME_LOG_LEVEL = "debug"
$env:HIME_IPC_PIPE = "\\.\pipe\hime-hermes-dev"
$env:HIME_ROUTER_PORT = $Port

# -----------------------------------------------------------------------------
# 3. Workspace Readiness Verification
# -----------------------------------------------------------------------------
Write-Host "[3/4] Verifying Workspace Targets..." -ForegroundColor Yellow

$coreCargoToml = Join-Path $workspaceRoot "crates/hime-core/Cargo.toml"
$desktopPackageJson = Join-Path $workspaceRoot "apps/hime-desktop/package.json"

$canRunCore = Test-Path $coreCargoToml
$canRunDesktop = Test-Path $desktopPackageJson

if (-not $canRunCore -and -not $canRunDesktop) {
    Write-Host "[FAIL] Workspace targets not found on disk:" -ForegroundColor Red
    Write-Host "  - crates/hime-core/Cargo.toml (Missing - Phase 06 implementation pending)" -ForegroundColor Yellow
    Write-Host "  - apps/hime-desktop/package.json (Missing - Phase 06 implementation pending)" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "ABORTING LAUNCH: Phase 06 codebase has not been created yet." -ForegroundColor Red
    exit 1
}

# -----------------------------------------------------------------------------
# 4. Process Supervision & Clean Shutdown Trap
# -----------------------------------------------------------------------------
Write-Host "[4/4] Starting Development Processes..." -ForegroundColor Yellow

$spawnedProcesses = [System.Collections.Generic.List[System.Diagnostics.Process]]::new()

function Stop-DevProcesses {
    Write-Host ""
    Write-Host "Initiating Clean Shutdown (Zero Orphan Cleanup)..." -ForegroundColor Yellow
    foreach ($proc in $spawnedProcesses) {
        if (-not $proc.HasExited) {
            Write-Host "Terminating PID $($proc.Id) ($($proc.ProcessName))..." -ForegroundColor Cyan
            try {
                $proc.Kill()
                $proc.WaitForExit(3000)
            } catch {
                Write-Host "Could not kill PID $($proc.Id): $_" -ForegroundColor DarkGray
            }
        }
    }
    Write-Host "Development processes stopped cleanly." -ForegroundColor Green
}

try {
    # If Core is available and requested
    if ($canRunCore -and -not $DesktopOnly) {
        Write-Host "Launching Hime Core (daemon)..." -ForegroundColor Cyan
        $releaseCore = Join-Path $workspaceRoot "target\release\hime-core.exe"
        $debugCore = Join-Path $workspaceRoot "target\debug\hime-core.exe"
        if (Test-Path $releaseCore) {
            $coreProc = Start-Process -FilePath $releaseCore -PassThru -NoNewWindow
        } elseif (Test-Path $debugCore) {
            $coreProc = Start-Process -FilePath $debugCore -PassThru -NoNewWindow
        } else {
            $coreProc = Start-Process -FilePath "cargo" -ArgumentList "run -p hime-core" -PassThru -NoNewWindow
        }
        $spawnedProcesses.Add($coreProc)
        Start-Sleep -Milliseconds 1000
    }

    # If Desktop is available and requested
    if ($canRunDesktop -and -not $CoreOnly) {
        # Ensure frontend bundle exists
        $distIndex = Join-Path $workspaceRoot "apps\hime-desktop\dist\index.html"
        if (-not (Test-Path $distIndex)) {
            Write-Host "Building frontend bundle..." -ForegroundColor Yellow
            cmd.exe /c "pnpm --dir apps/hime-desktop build"
        }

        Write-Host "Launching Hime Desktop (Floating Dynamic Island)..." -ForegroundColor Cyan
        $releaseDesktop = Join-Path $workspaceRoot "target\release\hime-desktop.exe"
        $debugDesktop = Join-Path $workspaceRoot "target\debug\hime-desktop.exe"
        $desktopWorkingDir = Join-Path $workspaceRoot "apps\hime-desktop\src-tauri"

        if (Test-Path $releaseDesktop) {
            $desktopProc = Start-Process -FilePath $releaseDesktop -WorkingDirectory $desktopWorkingDir -PassThru
        } elseif (Test-Path $debugDesktop) {
            $desktopProc = Start-Process -FilePath $debugDesktop -WorkingDirectory $desktopWorkingDir -PassThru
        } else {
            $desktopProc = Start-Process -FilePath "cargo" -ArgumentList "run -p hime-desktop" -WorkingDirectory $desktopWorkingDir -PassThru
        }
        $spawnedProcesses.Add($desktopProc)
    }

    Write-Host "All development services running. Press Ctrl+C to terminate." -ForegroundColor Green
    
    # Wait for child process exit or user interrupt
    while ($true) {
        Start-Sleep -Seconds 1
        foreach ($proc in $spawnedProcesses) {
            if ($proc.HasExited) {
                Write-Host "[NOTICE] Process PID $($proc.Id) exited with code $($proc.ExitCode)." -ForegroundColor Yellow
                break
            }
        }
    }
} finally {
    Stop-DevProcesses
}

