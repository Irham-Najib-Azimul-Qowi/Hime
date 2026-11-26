<#
.SYNOPSIS
    Hime Island Single Command Entry Point (scripts/hime.ps1)
.DESCRIPTION
    Unified Pure-Rust CLI for managing the Hime monorepo (ADR-017 Phase 4).
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
$repoRoot = Split-Path -Parent $PSScriptRoot

$coreManifest = Join-Path $repoRoot "core\Cargo.toml"
$nativeManifest = Join-Path $repoRoot "island\native\Cargo.toml"

function Show-Help {
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host "  HIME ISLAND - PURE-RUST MONOREPO CLI (scripts/hime.ps1)" -ForegroundColor Cyan
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host 'Usage: .\scripts\hime.ps1 <command> [target] [-Safe]'
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  doctor            Check Rust toolchain and system prerequisites"
    Write-Host "  setup             Fetch Cargo dependencies for core and island/native"
    Write-Host "  dev [-Safe]       Start hime-core and hime-island-native"
    Write-Host "  build [target]    Build release binaries (core, island, or all)"
    Write-Host "  test [target]     Run Rust test suite (core, island, or all)"
    Write-Host "  check             Run brand, hygiene, and Rust test gates"
    Write-Host "  restore-taskbar   Safely restore Windows native taskbars"
    Write-Host "  clean             Remove Rust target build artifacts"
}

switch ($Command.ToLower()) {
    "doctor" {
        Write-Host "Checking Hime Pure-Rust prerequisites..." -ForegroundColor Cyan
        $requiredTools = @("rustc", "cargo", "git")
        foreach ($t in $requiredTools) {
            $cmd = Get-Command $t -ErrorAction SilentlyContinue
            if ($cmd) {
                Write-Host "  [OK] $t found: $($cmd.Source)" -ForegroundColor Green
            } else {
                Write-Host "  [WARN] $t not found in PATH" -ForegroundColor Yellow
            }
        }
        $optionalTools = @("node")
        foreach ($t in $optionalTools) {
            $cmd = Get-Command $t -ErrorAction SilentlyContinue
            if ($cmd) {
                Write-Host "  [INFO] optional $t found: $($cmd.Source)" -ForegroundColor Gray
            }
        }
    }

    "setup" {
        Write-Host "Fetching Cargo dependencies for Hime Core & Native Island..." -ForegroundColor Cyan
        cargo fetch --manifest-path $coreManifest
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        cargo fetch --manifest-path $nativeManifest
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        Write-Host "Pure-Rust workspace setup completed." -ForegroundColor Green
    }

    "dev" {
        if ($Safe) {
            Write-Host "Safe mode requested: restoring Windows taskbar first..." -ForegroundColor Yellow
            & (Join-Path $repoRoot "scripts\restore-taskbar.ps1")
        }
        & (Join-Path $repoRoot "scripts\start-hime.ps1")
    }

    "test" {
        $t = $Target.ToLower()
        if ($t -eq "core" -or $t -eq "all") {
            Write-Host "Running Hime Core test suite..." -ForegroundColor Cyan
            cargo test --workspace --manifest-path $coreManifest
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
        if ($t -eq "island" -or $t -eq "native" -or $t -eq "all") {
            Write-Host "Running Hime Island Native test suite..." -ForegroundColor Cyan
            cargo test --manifest-path $nativeManifest
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
        Write-Host "All requested Rust tests PASSED." -ForegroundColor Green
    }

    "check" {
        Write-Host "Running Brand Compliance Gate..." -ForegroundColor Cyan
        & (Join-Path $repoRoot "scripts\check-brand.ps1")
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "Running Hygiene Gate..." -ForegroundColor Cyan
        & (Join-Path $repoRoot "scripts\check-hygiene.ps1")
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "Running Hime Core Test Suite..." -ForegroundColor Cyan
        cargo test --workspace --manifest-path $coreManifest
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "Running Hime Island Native Test Suite..." -ForegroundColor Cyan
        cargo test --manifest-path $nativeManifest
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        Write-Host "ALL GATES PASSED (100%)." -ForegroundColor Green
    }

    "build" {
        $t = $Target.ToLower()
        if ($t -eq "core" -or $t -eq "all") {
            Write-Host "Building Hime Core (Release)..." -ForegroundColor Cyan
            cargo build --release --workspace --manifest-path $coreManifest
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
        if ($t -eq "island" -or $t -eq "native" -or $t -eq "all") {
            Write-Host "Building Hime Island Native Shell (Release)..." -ForegroundColor Cyan
            cargo build --release --manifest-path $nativeManifest
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
    }

    "restore-taskbar" {
        $rtPath = Join-Path $repoRoot "island\scripts\restore-taskbar.ps1"
        if (-not (Test-Path $rtPath)) {
            $rtPath = Join-Path $repoRoot "scripts\restore-taskbar.ps1"
        }
        if (Test-Path $rtPath) {
            & $rtPath
        } else {
            Write-Host "Restoration script not found at $rtPath" -ForegroundColor Red
        }
    }

    "clean" {
        Write-Host "Cleaning Rust build targets..." -ForegroundColor Yellow
        $coreTarget = Join-Path $repoRoot "core\target"
        $nativeTarget = Join-Path $repoRoot "island\native\target"
        if (Test-Path $coreTarget) { Remove-Item $coreTarget -Recurse -Force -ErrorAction SilentlyContinue }
        if (Test-Path $nativeTarget) { Remove-Item $nativeTarget -Recurse -Force -ErrorAction SilentlyContinue }
        Write-Host "Clean complete." -ForegroundColor Green
    }

    Default {
        Show-Help
    }
}
