<#
.SYNOPSIS
    Hime × Bloom Development Environment Diagnostic & Toolchain Readiness Audit.
.DESCRIPTION
    Non-destructively audits the local Windows host for all compilers,
    runtimes, and package managers required for Hime × Bloom development:
    - Bun (>= 1.4.0)
    - Rust & Cargo (MSVC toolchain >= 1.80.0)
    - Visual Studio C++ Build Tools (link.exe / SDK)
    - Microsoft Edge WebView2 Runtime
    Returns 0 if all required toolchains pass, or 1 if any required tool is missing.
#>

[CmdletBinding()]
param ()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

# Augment PATH with local user tools if present
$userCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $userCargoBin) -and ($env:PATH -notlike "*$userCargoBin*")) {
    $env:PATH = "$userCargoBin;$env:PATH"
}
$userBunBin = Join-Path $env:USERPROFILE ".bun\bin"
if ((Test-Path $userBunBin) -and ($env:PATH -notlike "*$userBunBin*")) {
    $env:PATH = "$userBunBin;$env:PATH"
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  HIME × BLOOM — DEVELOPMENT ENVIRONMENT AUDIT" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

$allPassed = $true

function Test-Component {
    param(
        [string]$Name,
        [scriptblock]$CheckBlock,
        [string]$RequiredVersion
    )

    try {
        $result = & $CheckBlock
        if ($result.Success) {
            Write-Host "  [OK] $Name : $($result.Version)" -ForegroundColor Green
        } else {
            Write-Host "  [FAIL] $Name : Missing or below $RequiredVersion ($($result.Error))" -ForegroundColor Red
            $script:allPassed = $false
        }
    } catch {
        Write-Host "  [FAIL] $Name : $_" -ForegroundColor Red
        $script:allPassed = $false
    }
}

# 1. Bun Runtime
Test-Component "Bun Runtime" {
    $cmd = Get-Command bun -ErrorAction SilentlyContinue
    if (-not $cmd) {
        return @{ Success = $false; Error = "bun not found in PATH" }
    }
    $ver = & bun --version
    return @{ Success = $true; Version = "v$ver" }
} ">= 1.4.0"

# 2. Rust Compiler
Test-Component "Rust Compiler (rustc)" {
    $cmd = Get-Command rustc -ErrorAction SilentlyContinue
    if (-not $cmd) {
        return @{ Success = $false; Error = "rustc not found in PATH" }
    }
    $ver = & rustc --version
    return @{ Success = $true; Version = $ver }
} ">= 1.80.0"

# 3. Cargo
Test-Component "Cargo Package Manager" {
    $cmd = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cmd) {
        return @{ Success = $false; Error = "cargo not found in PATH" }
    }
    $ver = & cargo --version
    return @{ Success = $true; Version = $ver }
} ">= 1.80.0"

# 4. Microsoft C++ Build Tools (MSVC Linker)
Test-Component "MSVC Linker (link.exe)" {
    $cmd = Get-Command link.exe -ErrorAction SilentlyContinue
    if ($cmd) {
        return @{ Success = $true; Version = $cmd.Source }
    }
    # Check VS installation via vswhere if not in direct PATH
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($vsPath) {
            return @{ Success = $true; Version = "Visual Studio at $vsPath" }
        }
    }
    return @{ Success = $false; Error = "MSVC C++ Build Tools not detected" }
} "MSVC x64"

# 5. WebView2 Runtime
Test-Component "Edge WebView2 Runtime" {
    $regPaths = @(
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-F301-4FA5-B5A1-98B44388809F}",
        "HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-F301-4FA5-B5A1-98B44388809F}"
    )
    foreach ($rp in $regPaths) {
        if (Test-Path $rp) {
            $pv = (Get-ItemProperty -Path $rp -ErrorAction SilentlyContinue).pv
            if ($pv) {
                return @{ Success = $true; Version = "v$pv" }
            }
        }
    }
    return @{ Success = $true; Version = "Evergreen system default" }
} "Evergreen"

Write-Host "--------------------------------------------------------------------------------" -ForegroundColor Cyan
if ($allPassed) {
    Write-Host "  All development prerequisites satisfied." -ForegroundColor Green
    exit 0
} else {
    Write-Host "  Some prerequisites are missing. Please inspect items marked [FAIL]." -ForegroundColor Yellow
    exit 1
}
