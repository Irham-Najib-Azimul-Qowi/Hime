<#
.SYNOPSIS
    Automated Pure-Rust E2E Verification Suite for Hime Desktop (ADR-017 Phase 4)
.DESCRIPTION
    Executes all pure-Rust verification gates and test suites (zero Bun/Tauri required):
    1. Brand Compliance Gate (scripts/check-brand.ps1)
    2. Code Hygiene & Quality Gate (scripts/check-hygiene.ps1)
    3. Core Workspace Tests (core/Cargo.toml)
    4. Native Island Shell Tests (island/native/Cargo.toml)

    Exits with code 0 on complete pass across all tiers.
#>

[CmdletBinding()]
param(
    [switch]$Detailed
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$totalStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$results = @()

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "           HIME DESKTOP PURE-RUST AUTOMATED E2E VERIFICATION SUITE              " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "Repository Root: $repoRoot" -ForegroundColor Gray
Write-Host "Timestamp      : $((Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm:ss UTC'))" -ForegroundColor Gray
Write-Host ""

function Invoke-VerificationTier {
    param(
        [string]$Name,
        [string]$Description,
        [scriptblock]$Action
    )

    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor DarkCyan
    Write-Host "[RUNNING] $Name : $Description" -ForegroundColor Cyan
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor DarkCyan

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $status = "PASS"
    $errorMsg = ""

    try {
        & $Action
        if ($LASTEXITCODE -ne $null -and $LASTEXITCODE -ne 0) {
            throw "Process exited with non-zero exit code: $LASTEXITCODE"
        }
    } catch {
        $status = "FAIL"
        $errorMsg = $_.Exception.Message
        Write-Host "[ERROR] $errorMsg" -ForegroundColor Red
    } finally {
        $sw.Stop()
    }

    $duration = "{0:N2}s" -f ($sw.ElapsedMilliseconds / 1000)
    if ($status -eq "PASS") {
        Write-Host ">>> $Name : PASSED ($duration)" -ForegroundColor Green
    } else {
        Write-Host ">>> $Name : FAILED ($duration)" -ForegroundColor Red
    }
    Write-Host ""

    $script:results += [PSCustomObject]@{
        Tier        = $Name
        Description = $Description
        Status      = $status
        Duration    = $duration
        Error       = $errorMsg
    }
}

# --- Tier 1: Brand Compliance Gate ---
Invoke-VerificationTier "Tier 1: Brand Compliance" "Scan active Rust source and scripts for brand leaks" {
    $script = Join-Path $repoRoot "scripts\check-brand.ps1"
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $script
    if ($LASTEXITCODE -ne 0) { throw "check-brand.ps1 reported violations" }
}

# --- Tier 2: Code Hygiene Gate ---
Invoke-VerificationTier "Tier 2: Code Hygiene" "Enforce task anchors, clean Rust source, zero stray artifacts" {
    $script = Join-Path $repoRoot "scripts\check-hygiene.ps1"
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $script
    if ($LASTEXITCODE -ne 0) { throw "check-hygiene.ps1 reported violations" }
}

# --- Tier 3: Core Workspace Tests ---
Invoke-VerificationTier "Tier 3: Core Tests" "Rust core workspace suite (core/Cargo.toml)" {
    $manifest = Join-Path $repoRoot "core\Cargo.toml"
    cargo test --workspace --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw "Core cargo tests failed" }
}

# --- Tier 4: Native Island Shell Tests ---
Invoke-VerificationTier "Tier 4: Native Island" "Pure-Rust Direct2D native shell tests (island/native/Cargo.toml)" {
    $manifest = Join-Path $repoRoot "island\native\Cargo.toml"
    cargo test --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw "Native Island cargo tests failed" }
}

$totalStopwatch.Stop()
$totalSecs = "{0:N2}s" -f ($totalStopwatch.ElapsedMilliseconds / 1000)

# --- Summary Report ---
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "                        VERIFICATION SUITE SUMMARY                              " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

$results | Format-Table -Property Tier, Status, Duration, Description -AutoSize

$failedCount = ($results | Where-Object { $_.Status -ne "PASS" }).Count

if ($failedCount -eq 0) {
    Write-Host "[ALL PASSED] All $($results.Count) pure-Rust verification tiers passed cleanly in $totalSecs." -ForegroundColor Green
    exit 0
} else {
    Write-Host "[FAILED] $failedCount tier(s) failed out of $($results.Count) in $totalSecs." -ForegroundColor Red
    exit 1
}
