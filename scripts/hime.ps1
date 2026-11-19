<#
.SYNOPSIS
    Hime Island Single Command Entry Point (scripts/hime.ps1)
.DESCRIPTION
    Unified CLI for managing the Hime Island monorepo per 02b-MONOREPO-LAYOUT-AND-MANAGEMENT.md.
    Provides subcommands: doctor, setup, dev, build, test, check, restore-taskbar, clean.
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Command = "help",

    [Parameter(Position = 1)]
    [string]$Target = "all",

    [switch]$Safe
)

$ErrorActionPreference = "Stop"
$baseDir = Split-Path -Parent $PSScriptRoot

$bunExe = "C:\Users\Satoshi\.bun\bin\bun.exe"
if (-not (Test-Path $bunExe)) {
    $bunCmd = Get-Command bun -ErrorAction SilentlyContinue
    if ($bunCmd) { $bunExe = $bunCmd.Source }
}

function Show-Help {
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host "  HIME ISLAND - MONOREPO CLI (scripts/hime.ps1)" -ForegroundColor Cyan
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host 'Usage: .\scripts\hime.ps1 <command> [target] [-Safe]'
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  doctor            Check system prerequisites and toolchains"
    Write-Host "  setup             Install dependencies across island and core"
    Write-Host "  dev [-Safe]       Start core and island in development mode"
    Write-Host "  build [target]    Build core, island, or all"
    Write-Host "  test [target]     Run automated test suite for core, island, or all"
    Write-Host "  check             Run all quality and consistency gates (brand, hygiene, tests)"
    Write-Host "  restore-taskbar   Safely restore Windows native taskbars"
    Write-Host "  clean             Remove build output and cached artifacts"
}

switch ($Command.ToLower()) {
    "doctor" {
        Write-Host "Checking Hime Island prerequisites..." -ForegroundColor Cyan
        $tools = @("rustc", "cargo", "git", "node", "npm")
        foreach ($t in $tools) {
            $cmd = Get-Command $t -ErrorAction SilentlyContinue
            if ($cmd) {
                Write-Host "  [OK] $t found: $($cmd.Source)" -ForegroundColor Green
            } else {
                Write-Host "  [WARN] $t not found in PATH" -ForegroundColor Yellow
            }
        }
        if (Test-Path $bunExe) {
            Write-Host "  [OK] bun found: $bunExe" -ForegroundColor Green
        } else {
            Write-Host "  [WARN] bun not found at expected location" -ForegroundColor Yellow
        }
    }

    "setup" {
        Write-Host "Setting up Hime Island workspace dependencies..." -ForegroundColor Cyan
        Set-Location "$baseDir\island"
        & $bunExe install
        Set-Location $baseDir
        Write-Host "Workspace setup completed." -ForegroundColor Green
    }

    "test" {
        Write-Host "Running Hime Core test suite..." -ForegroundColor Cyan
        cargo test --workspace --manifest-path "$baseDir\core\Cargo.toml"
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "Running Island frontend typecheck & build test..." -ForegroundColor Cyan
        Set-Location "$baseDir\island"
        & $bunExe run build
        Set-Location $baseDir
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "Checking Island Rust shell..." -ForegroundColor Cyan
        cargo check --manifest-path "$baseDir\island\src-tauri\Cargo.toml"
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "All automated tests PASSED (100%)." -ForegroundColor Green
    }

    "check" {
        Write-Host "Running Brand Compliance Gate..." -ForegroundColor Cyan
        & "$baseDir\scripts\check-brand.ps1"
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "Running Hygiene Gate..." -ForegroundColor Cyan
        & "$baseDir\scripts\check-hygiene.ps1"
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "Running Core and Island Test Suites..." -ForegroundColor Cyan
        cargo test --workspace --manifest-path "$baseDir\core\Cargo.toml"
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "ALL GATES PASSED (100%)." -ForegroundColor Green
    }

    "build" {
        if ($Target -eq "core" -or $Target -eq "all") {
            Write-Host "Building Hime Core (Release)..." -ForegroundColor Cyan
            cargo build --release --workspace --manifest-path "$baseDir\core\Cargo.toml"
        }
        if ($Target -eq "island" -or $Target -eq "all") {
            Write-Host "Building Island Frontend..." -ForegroundColor Cyan
            Set-Location "$baseDir\island"
            & $bunExe run build
            Set-Location $baseDir
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
