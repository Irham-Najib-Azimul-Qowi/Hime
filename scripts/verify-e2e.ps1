<#
.SYNOPSIS
    Automated Full E2E Verification Suite for Hime Desktop
.DESCRIPTION
    Executes all verification gates and test suites across all tiers:
    1. Brand Compliance Gate (scripts/check-brand.ps1)
    2. Code Hygiene & Quality Gate (scripts/check-hygiene.ps1)
    3. Core Workspace Integration Tests (core/Cargo.toml: 124+ tests)
    4. Island Frontend Unit & Logic Tests (island/: 107 tests via Bun)
    5. Island Desktop Shell Tests (island/src-tauri/Cargo.toml: 104 tests)
    
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
Write-Host "              HIME DESKTOP AUTOMATED E2E VERIFICATION SUITE                     " -ForegroundColor Cyan
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
Invoke-VerificationTier "Tier 1: Brand Compliance" "Scan active source for brand leaks (scripts/check-brand.ps1)" {
    $script = Join-Path $repoRoot "scripts\check-brand.ps1"
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $script
    if ($LASTEXITCODE -ne 0) { throw "check-brand.ps1 reported violations" }
}

# --- Tier 2: Code Hygiene Gate ---
Invoke-VerificationTier "Tier 2: Code Hygiene" "Enforce task anchors, clean source, zero stray artifacts" {
    $script = Join-Path $repoRoot "scripts\check-hygiene.ps1"
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $script
    if ($LASTEXITCODE -ne 0) { throw "check-hygiene.ps1 reported violations" }
}

# --- Tier 3: Core Workspace Tests ---
Invoke-VerificationTier "Tier 3: Core Tests" "Rust core workspace integration suite (124+ unit & integration tests)" {
    $manifest = Join-Path $repoRoot "core\Cargo.toml"
    cargo test --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw "Core cargo tests failed" }
}

# --- Tier 4: Island Frontend Tests ---
Invoke-VerificationTier "Tier 4: Frontend Tests" "Island React & TypeScript logic test suite (107 tests via Bun)" {
    $bunExe = if (Get-Command bun -ErrorAction SilentlyContinue) {
        "bun"
    } elseif (Test-Path "$env:USERPROFILE\.bun\bin\bun.exe") {
        "$env:USERPROFILE\.bun\bin\bun.exe"
    } else {
        throw "Bun test runner not found in PATH or $env:USERPROFILE\.bun\bin\bun.exe"
    }
    
    $islandDir = Join-Path $repoRoot "island"
    Push-Location $islandDir
    try {
        & $bunExe test
        if ($LASTEXITCODE -ne 0) { throw "Island frontend bun tests failed" }
    } finally {
        Pop-Location
    }
}

# --- Tier 5: Island Shell Tests ---
Invoke-VerificationTier "Tier 5: Island Shell" "Tauri 2 native shell and services unit tests (104 tests)" {
    $manifest = Join-Path $repoRoot "island\src-tauri\Cargo.toml"
    cargo test --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw "Island shell cargo tests failed" }
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
    Write-Host "[ALL PASSED] All 5 verification tiers passed cleanly in $totalSecs with 0 errors." -ForegroundColor Green
    exit 0
} else {
    Write-Host "[FAILED] $failedCount tier(s) failed out of $($results.Count) in $totalSecs." -ForegroundColor Red
    exit 1
}
