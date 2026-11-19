<#
.SYNOPSIS
    Hime Island Single Command Entry Point (scripts/hime.ps1)
.DESCRIPTION
    Unified CLI for doctor, setup, dev, build, test, check, perf, and maintenance.
#>

[CmdletBinding()]
param(
    [Parameter(Position=0)]
    [ValidateSet("doctor", "setup", "dev", "build", "test", "check", "perf", "restore-taskbar", "clean", "help")]
    [string]$Command = "help",

    [Parameter(Position=1)]
    [string]$Target = "all",

    [switch]$Safe
)

$baseDir = Split-Path -Parent $PSScriptRoot
$bunExe = "C:\Users\Satoshi\.bun\bin\bun.exe"
if (-not (Test-Path $bunExe)) {
    $bunCmd = Get-Command bun -ErrorAction SilentlyContinue
    if ($bunCmd) { $bunExe = $bunCmd.Source }
}

function Show-Help {
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host "  HIME ISLAND — MONOREPO CLI (scripts/hime.ps1)" -ForegroundColor Cyan
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host "Usage: .\scripts\hime.ps1 <command> [target] [-Safe]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  doctor            Check system prerequisites and toolchains"
    Write-Host "  setup             Install dependencies across island and core"
    Write-Host "  dev [-Safe]       Start core and island in development mode"
    Write-Host "  build [target]    Build core, island, or all"
    Write-Host "  test [target]     Run automated test suite for core, island, or all"
    Write-Host "  check             Run all quality and consistency gates"
    Write-Host "  restore-taskbar   Safely restore Windows native taskbars"
    Write-Host "  clean             Remove build output and cached artifacts"
}

switch ($Command) {
    "doctor" {
        Write-Host "=== HIME ISLAND DOCTOR ===" -ForegroundColor Cyan
        $tools = @{
            "Rust (rustc)" = (Get-Command rustc -ErrorAction SilentlyContinue)
            "Cargo"        = (Get-Command cargo -ErrorAction SilentlyContinue)
            "Bun"          = (Test-Path $bunExe)
            "Node.js"      = (Get-Command node -ErrorAction SilentlyContinue)
            "NPM"          = (Get-Command npm -ErrorAction SilentlyContinue)
            "Git"          = (Get-Command git -ErrorAction SilentlyContinue)
        }
        foreach ($k in $tools.Keys) {
            if ($tools[$k]) {
                Write-Host "[OK] $k is available" -ForegroundColor Green
            } else {
                Write-Host "[MISSING] $k is not detected" -ForegroundColor Red
            }
        }
    }

    "setup" {
        Write-Host "Setting up Island dependencies..." -ForegroundColor Cyan
        Set-Location "$baseDir\island"
        & $bunExe install
        Write-Host "Verifying Core workspace..." -ForegroundColor Cyan
        cargo check --manifest-path "$baseDir\core\Cargo.toml"
    }

    "test" {
        if ($Target -eq "core" -or $Target -eq "all") {
            Write-Host "Running Core test suite..." -ForegroundColor Cyan
            cargo test --manifest-path "$baseDir\core\Cargo.toml"
        }
        if ($Target -eq "island" -or $Target -eq "all") {
            Write-Host "Running Island check..." -ForegroundColor Cyan
            cargo check --manifest-path "$baseDir\island\src-tauri\Cargo.toml"
        }
    }

    "build" {
        if ($Target -eq "core" -or $Target -eq "all") {
            Write-Host "Building Hime Core (Release)..." -ForegroundColor Cyan
            cargo build --release --manifest-path "$baseDir\core\Cargo.toml"
        }
        if ($Target -eq "island" -or $Target -eq "all") {
            Write-Host "Building Island Frontend..." -ForegroundColor Cyan
            Set-Location "$baseDir\island"
            & $bunExe run build
            Write-Host "Building Island Shell (Release)..." -ForegroundColor Cyan
            cargo build --release --manifest-path "$baseDir\island\src-tauri\Cargo.toml"
        }
    }

    "restore-taskbar" {
        $rtPath = "$baseDir\island\scripts\restore-taskbar.ps1"
        if (Test-Path $rtPath) {
            & $rtPath
        } else {
            Write-Host "Restoration script not found at $rtPath" -ForegroundColor Red
        }
    }

    "clean" {
        Write-Host "Cleaning build targets..." -ForegroundColor Yellow
        if (Test-Path "$baseDir\core\target") { Remove-Item "$baseDir\core\target" -Recurse -Force -ErrorAction SilentlyContinue }
        if (Test-Path "$baseDir\island\src-tauri\target") { Remove-Item "$baseDir\island\src-tauri\target" -Recurse -Force -ErrorAction SilentlyContinue }
        if (Test-Path "$baseDir\island\dist") { Remove-Item "$baseDir\island\dist" -Recurse -Force -ErrorAction SilentlyContinue }
        Write-Host "Clean complete." -ForegroundColor Green
    }

    Default {
        Show-Help
    }
}
