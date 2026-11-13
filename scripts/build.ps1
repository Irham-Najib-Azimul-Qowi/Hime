<#
.SYNOPSIS
    Hime Monorepo Workspace Build Script.
.DESCRIPTION
    Builds the Hime multi-crate Rust workspace and Tauri 2.0 desktop frontend.
    Verifies that the required toolchains and workspace structure exist before
    initiating compilation. Fails closed with deterministic exit code if
    prerequisites or workspace members are missing.
.PARAMETER Release
    Compiles with release profile optimizations (--release).
.PARAMETER BackendOnly
    Builds only the Rust crates workspace.
.PARAMETER FrontendOnly
    Builds only the React/TypeScript desktop frontend.
.PARAMETER Clean
    Cleans previous build artifacts before building.
#>

[CmdletBinding()]
param (
    [switch]$Release,
    [switch]$BackendOnly,
    [switch]$FrontendOnly,
    [switch]$Clean
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Augment process PATH with default user toolchain directories if present
$userCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $userCargoBin) -and ($env:PATH -notlike "*$userCargoBin*")) {
    $env:PATH = "$userCargoBin;$env:PATH"
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  HIME MONOREPO WORKSPACE BUILD AUTOMATION" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

$workspaceRoot = Split-Path -Parent $PSScriptRoot
Set-Location $workspaceRoot

# -----------------------------------------------------------------------------
# 1. Prerequisite Verification (Fail-Closed Check)
# -----------------------------------------------------------------------------
Write-Host "[1/3] Verifying Build Toolchain Prerequisites..." -ForegroundColor Yellow

$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
$rustcCmd = Get-Command rustc -ErrorAction SilentlyContinue
$pnpmCmd  = Get-Command pnpm -ErrorAction SilentlyContinue
$nodeCmd  = Get-Command node -ErrorAction SilentlyContinue

$missingTools = @()
if (-not $cargoCmd) { $missingTools += "Cargo (Rust package manager)" }
if (-not $rustcCmd) { $missingTools += "rustc (Rust compiler)" }
if (-not $pnpmCmd -and -not $BackendOnly)  { $missingTools += "pnpm (Frontend package manager)" }
if (-not $nodeCmd -and -not $BackendOnly)  { $missingTools += "Node.js (JavaScript runtime)" }

if ($missingTools.Count -gt 0) {
    Write-Host "[FAIL] Missing required build toolchains:" -ForegroundColor Red
    foreach ($tool in $missingTools) {
        Write-Host "  - $tool" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "Please run .\scripts\check-env.ps1 for installation instructions." -ForegroundColor Yellow
    exit 1
}

Write-Host "[PASS] Required build toolchains verified." -ForegroundColor Green

# -----------------------------------------------------------------------------
# 2. Workspace Structural Verification
# -----------------------------------------------------------------------------
Write-Host "[2/3] Verifying Cargo Workspace Structure..." -ForegroundColor Yellow

$cargoTomlPath = Join-Path $workspaceRoot "Cargo.toml"
if (-not (Test-Path $cargoTomlPath)) {
    Write-Host "[FAIL] Root Cargo.toml manifest not found in $workspaceRoot." -ForegroundColor Red
    Write-Host "Phase 06 workspace has not been initialized." -ForegroundColor Yellow
    exit 1
}

# Check for required Phase 06 crate directories
$requiredCrates = @("crates/hime-core", "crates/hime-ipc", "crates/hime-pal", "crates/hime-storage", "crates/hime-security")
$missingCrates = @()
foreach ($crateRel in $requiredCrates) {
    $cratePath = Join-Path $workspaceRoot $crateRel
    if (-not (Test-Path $cratePath)) {
        $missingCrates += $crateRel
    }
}

if ($missingCrates.Count -gt 0 -and -not $FrontendOnly) {
    Write-Host "[FAIL] Cargo workspace members are missing on disk:" -ForegroundColor Red
    foreach ($c in $missingCrates) {
        Write-Host "  - $c (Source code not yet created; Phase 06 implementation pending)" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "ABORTING BUILD: Zero implementation code exists yet. Do not fabricate build outputs." -ForegroundColor Red
    exit 1
}

# -----------------------------------------------------------------------------
# 3. Execution of Build Pipeline
# -----------------------------------------------------------------------------
Write-Host "[3/3] Executing Build Targets..." -ForegroundColor Yellow

$profileArg = if ($Release) { "--release" } else { "" }

# A. Clean if requested
if ($Clean) {
    Write-Host "Cleaning target directory..." -ForegroundColor Cyan
    & cargo clean
    if (Test-Path (Join-Path $workspaceRoot "node_modules")) {
        # Optional pnpm clean
    }
}

# B. Frontend Build
if (-not $BackendOnly) {
    $desktopDir = Join-Path $workspaceRoot "apps/hime-desktop"
    if (Test-Path $desktopDir) {
        Write-Host "Building Hime Desktop frontend in $desktopDir..." -ForegroundColor Cyan
        Push-Location $desktopDir
        try {
            & pnpm install --frozen-lockfile
            & pnpm build
        } finally {
            Pop-Location
        }
    } else {
        Write-Host "[WARN] Desktop frontend directory ($desktopDir) not found; skipping frontend." -ForegroundColor Yellow
    }
}

# C. Backend Crates Workspace Build
if (-not $FrontendOnly) {
    Write-Host "Building Rust workspace crates (Profile: $(if ($Release) { 'Release' } else { 'Dev' }))..." -ForegroundColor Cyan
    if ($Release) {
        & cargo build --workspace --release
    } else {
        & cargo build --workspace
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "[FAIL] Cargo workspace build failed with exit code $LASTEXITCODE." -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Write-Host ""
Write-Host "================================================================================" -ForegroundColor Green
Write-Host "  BUILD COMPLETED SUCCESSFULLY" -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Green
exit 0

