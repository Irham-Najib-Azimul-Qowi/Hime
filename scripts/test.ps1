<#
.SYNOPSIS
    Hime Test Suite Automation Script.
.DESCRIPTION
    Executes unit tests, integration tests, contract tests, and security tests
    across the Hime monorepo workspace.
    Fails closed if the required toolchain, workspace crates, or test harnesses
    are missing on disk. Does not fabricate test results.
.PARAMETER Crate
    Name of a specific crate to test (e.g., hime-ipc, hime-core).
.PARAMETER UnitOnly
    Executes only unit tests (--lib).
.PARAMETER IntegrationOnly
    Executes only integration tests (--test '*').
.PARAMETER DocTests
    Executes documentation tests (--doc).
#>

[CmdletBinding()]
param (
    [string]$Crate = "",
    [switch]$UnitOnly,
    [switch]$IntegrationOnly,
    [switch]$DocTests
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Augment process PATH with default user toolchain directories if present
$userCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $userCargoBin) -and ($env:PATH -notlike "*$userCargoBin*")) {
    $env:PATH = "$userCargoBin;$env:PATH"
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  HIME TEST SUITE EXECUTION AUTOMATION" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

$workspaceRoot = Split-Path -Parent $PSScriptRoot
Set-Location $workspaceRoot

# -----------------------------------------------------------------------------
# 1. Prerequisite Verification
# -----------------------------------------------------------------------------
Write-Host "[1/3] Verifying Testing Toolchains..." -ForegroundColor Yellow

$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoCmd) {
    Write-Host "[FAIL] Cargo is not installed or not in PATH." -ForegroundColor Red
    Write-Host "ABORTING TEST RUN: Cannot execute tests without Cargo." -ForegroundColor Red
    Write-Host "Run .\scripts\check-env.ps1 for installation guidance." -ForegroundColor Yellow
    exit 1
}

# -----------------------------------------------------------------------------
# 2. Workspace & Crate Verification
# -----------------------------------------------------------------------------
Write-Host "[2/3] Verifying Test Targets on Disk..." -ForegroundColor Yellow

$cargoTomlPath = Join-Path $workspaceRoot "Cargo.toml"
if (-not (Test-Path $cargoTomlPath)) {
    Write-Host "[FAIL] Root Cargo.toml manifest not found in $workspaceRoot." -ForegroundColor Red
    Write-Host "ABORTING TEST RUN: Workspace not initialized." -ForegroundColor Red
    exit 1
}

if ($Crate -ne "") {
    $targetPath = Join-Path $workspaceRoot "crates/$Crate"
    if (-not (Test-Path $targetPath)) {
        Write-Host "[FAIL] Specified crate '$Crate' does not exist at $targetPath." -ForegroundColor Red
        exit 1
    }
} else {
    $cratesDir = Join-Path $workspaceRoot "crates"
    $hasCrates = (Test-Path $cratesDir) -and ((Get-ChildItem $cratesDir -Directory -ErrorAction SilentlyContinue).Count -gt 0)
    if (-not $hasCrates) {
        Write-Host "[FAIL] No crates found in $cratesDir. Phase 06 implementation code has not been created." -ForegroundColor Red
        Write-Host "STATUS: 0 TESTS EXECUTED (SPECIFICATION STAGE)." -ForegroundColor Yellow
        exit 1
    }
}

# -----------------------------------------------------------------------------
# 3. Test Execution
# -----------------------------------------------------------------------------
Write-Host "[3/3] Executing Cargo Test Suite..." -ForegroundColor Yellow

$testArgs = @("test")
if ($Crate -ne "") {
    $testArgs += @("-p", $Crate)
} else {
    $testArgs += "--workspace"
}

if ($UnitOnly) {
    $testArgs += "--lib"
} elseif ($IntegrationOnly) {
    $testArgs += @("--test", "*")
} elseif ($DocTests) {
    $testArgs += "--doc"
}

Write-Host "Running: cargo $($testArgs -join ' ')..." -ForegroundColor Cyan
& cargo @testArgs

$exitCode = $LASTEXITCODE
if ($exitCode -ne 0) {
    Write-Host ""
    Write-Host "[FAIL] Test suite failed with exit code $exitCode." -ForegroundColor Red
    exit $exitCode
}

Write-Host ""
Write-Host "================================================================================" -ForegroundColor Green
Write-Host "  ALL TESTS PASSED SUCCESSFULLY" -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Green
exit 0

