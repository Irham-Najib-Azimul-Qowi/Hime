<#
.SYNOPSIS
    Hime Development Environment Diagnostic & Toolchain Readiness Audit Script.
.DESCRIPTION
    Non-destructively audits the local host (Windows 11) for all compilers,
    runtimes, package managers, and platform SDKs required for Hime development.
    Does NOT install or modify any system software or configuration.
    Returns exit code 0 if all required toolchains are available, or 1 if any
    required component is missing or fails version verification.
#>

[CmdletBinding()]
param (
    [switch]$Detailed
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

# Automatically augment current process PATH with default user toolchain directories if present on disk
$userCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $userCargoBin) -and ($env:PATH -notlike "*$userCargoBin*")) {
    $env:PATH = "$userCargoBin;$env:PATH"
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  HIME ASSISTANT - DEVELOPMENT ENVIRONMENT READINESS AUDIT" -ForegroundColor Cyan
Write-Host "  Authority: System Architect and Technical Lead | Mode: Non-Destructive Audit" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

$auditResults = [System.Collections.Generic.List[PSCustomObject]]::new()
$hasMissingRequired = $false

function Register-AuditItem {
    param(
        [string]$Category,
        [string]$Component,
        [string]$RequiredVersion,
        [string]$DetectedVersion,
        [string]$Status,
        [string]$Priority,
        [string]$ActionRequired
    )

    $obj = [PSCustomObject]@{
        Category       = $Category
        Component      = $Component
        Required       = $RequiredVersion
        Detected       = $DetectedVersion
        Status         = $Status
        Priority       = $Priority
        ActionRequired = $ActionRequired
    }
    $script:auditResults.Add($obj)

    if ($Priority -eq "REQUIRED" -and $Status -ne "AVAILABLE") {
        $script:hasMissingRequired = $true
    }
}

# -----------------------------------------------------------------------------
# 1. Host Operating System & PowerShell
# -----------------------------------------------------------------------------
Write-Host "[1/7] Auditing Host Operating System and Core Runtimes..." -ForegroundColor Yellow

$osInfo = Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue
$osCaption = if ($osInfo) { $osInfo.Caption } else { [System.Environment]::OSVersion.VersionString }
$osVersion = if ($osInfo) { "$($osInfo.Version) (Build $($osInfo.BuildNumber))" } else { [System.Environment]::OSVersion.Version.ToString() }
$is64Bit = [System.Environment]::Is64BitOperatingSystem

if ($is64Bit -and $osVersion -match '2[0-9]{4}') {
    Register-AuditItem -Category "Operating System" -Component "Windows OS (x64)" -RequiredVersion "Windows 11 (Build 22000+)" -DetectedVersion "$osCaption $osVersion" -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Operating System" -Component "Windows OS (x64)" -RequiredVersion "Windows 11 (Build 22000+)" -DetectedVersion "$osCaption $osVersion" -Status "VERSION MISMATCH" -Priority "REQUIRED" -ActionRequired "Upgrade host to Windows 11 64-bit."
}

$psVer = $PSVersionTable.PSVersion.ToString()
if ($PSVersionTable.PSVersion.Major -ge 5) {
    Register-AuditItem -Category "Operating System" -Component "PowerShell" -RequiredVersion ">= 5.1" -DetectedVersion $psVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Operating System" -Component "PowerShell" -RequiredVersion ">= 5.1" -DetectedVersion $psVer -Status "VERSION MISMATCH" -Priority "REQUIRED" -ActionRequired "Install Windows Management Framework 5.1 or PowerShell 7."
}

# -----------------------------------------------------------------------------
# 2. Version Control (Git)
# -----------------------------------------------------------------------------
Write-Host "[2/7] Auditing Version Control and Git..." -ForegroundColor Yellow

$gitCmd = Get-Command git -ErrorAction SilentlyContinue
if ($gitCmd) {
    $gitVerRaw = & git --version 2>&1
    Register-AuditItem -Category "Version Control" -Component "Git" -RequiredVersion ">= 2.40" -DetectedVersion $gitVerRaw -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Version Control" -Component "Git" -RequiredVersion ">= 2.40" -DetectedVersion "None" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Download and install Git from https://git-scm.com/download/win"
}

# -----------------------------------------------------------------------------
# 3. Web & Frontend Toolchain (Node.js, npm, pnpm)
# -----------------------------------------------------------------------------
Write-Host "[3/7] Auditing Web and Frontend Toolchain..." -ForegroundColor Yellow

$nodeCmd = Get-Command node -ErrorAction SilentlyContinue
if ($nodeCmd) {
    $nodeVer = & node -v 2>&1
    Register-AuditItem -Category "Web Toolchain" -Component "Node.js" -RequiredVersion ">= 20.0" -DetectedVersion $nodeVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Web Toolchain" -Component "Node.js" -RequiredVersion ">= 20.0" -DetectedVersion "None" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Install Node.js 20+ LTS via https://nodejs.org/"
}

$npmCmd = Get-Command npm -ErrorAction SilentlyContinue
if ($npmCmd) {
    $npmVer = & npm -v 2>&1
    Register-AuditItem -Category "Web Toolchain" -Component "npm" -RequiredVersion ">= 10.0" -DetectedVersion "v$npmVer" -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Web Toolchain" -Component "npm" -RequiredVersion ">= 10.0" -DetectedVersion "None" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Bundled with Node.js."
}

$pnpmCmd = Get-Command pnpm -ErrorAction SilentlyContinue
if ($pnpmCmd) {
    $pnpmVer = & pnpm -v 2>&1
    Register-AuditItem -Category "Web Toolchain" -Component "pnpm" -RequiredVersion ">= 9.0" -DetectedVersion "v$pnpmVer" -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Web Toolchain" -Component "pnpm" -RequiredVersion ">= 9.0" -DetectedVersion "None" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Run: npm install -g pnpm"
}

# -----------------------------------------------------------------------------
# 4. Python & Agent Runtime Toolchain (Python, uv)
# -----------------------------------------------------------------------------
Write-Host "[4/7] Auditing Python and Agent Runtime Toolchain..." -ForegroundColor Yellow

$pyCmd = Get-Command python -ErrorAction SilentlyContinue
if ($pyCmd) {
    $pyVer = & python --version 2>&1
    Register-AuditItem -Category "Python Toolchain" -Component "Python" -RequiredVersion ">= 3.11" -DetectedVersion $pyVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Python Toolchain" -Component "Python" -RequiredVersion ">= 3.11" -DetectedVersion "None" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Install Python 3.11+ from https://python.org or via uv."
}

$uvCmd = Get-Command uv -ErrorAction SilentlyContinue
if ($uvCmd) {
    $uvVer = & uv --version 2>&1
    Register-AuditItem -Category "Python Toolchain" -Component "uv Package Manager" -RequiredVersion ">= 0.10" -DetectedVersion $uvVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Python Toolchain" -Component "uv Package Manager" -RequiredVersion ">= 0.10" -DetectedVersion "None" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Install uv: powershell -c 'irm https://astral.sh/uv/install.ps1 | iex'"
}

# -----------------------------------------------------------------------------
# 5. Rust Compiler & Cargo Monorepo Toolchain
# -----------------------------------------------------------------------------
Write-Host "[5/7] Auditing Rust Compiler and Cargo Monorepo Toolchain..." -ForegroundColor Yellow

$rustcCmd = Get-Command rustc -ErrorAction SilentlyContinue
if ($rustcCmd) {
    $rustcVer = & rustc --version 2>&1
    Register-AuditItem -Category "Rust Toolchain" -Component "Rust Compiler (rustc)" -RequiredVersion ">= 1.80.0" -DetectedVersion $rustcVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Rust Toolchain" -Component "Rust Compiler (rustc)" -RequiredVersion ">= 1.80.0" -DetectedVersion "Not Found" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Install Rust via winget: 'winget install Rustlang.Rustup' or run rustup-init.exe from https://rustup.rs/"
}

$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCmd) {
    $cargoVer = & cargo --version 2>&1
    Register-AuditItem -Category "Rust Toolchain" -Component "Cargo Package Manager" -RequiredVersion ">= 1.80.0" -DetectedVersion $cargoVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Rust Toolchain" -Component "Cargo Package Manager" -RequiredVersion ">= 1.80.0" -DetectedVersion "Not Found" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Bundled with Rustup installation."
}

$rustupCmd = Get-Command rustup -ErrorAction SilentlyContinue
if ($rustupCmd) {
    $rustupVer = & rustup --version 2>&1
    Register-AuditItem -Category "Rust Toolchain" -Component "Rustup Toolchain Manager" -RequiredVersion ">= 1.27.0" -DetectedVersion $rustupVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Rust Toolchain" -Component "Rustup Toolchain Manager" -RequiredVersion ">= 1.27.0" -DetectedVersion "Not Found" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Install Rustup via winget or https://rustup.rs/"
}

# -----------------------------------------------------------------------------
# 6. Windows C++ Native Build Tools, SDK & WebView2
# -----------------------------------------------------------------------------
Write-Host "[6/7] Auditing MSVC Build Tools, Windows SDK and WebView2..." -ForegroundColor Yellow

$vswherePaths = @(
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe",
    "${env:ProgramFiles}\Microsoft Visual Studio\Installer\vswhere.exe"
)
$msvcFound = $false
$msvcPath = ""
foreach ($vsw in $vswherePaths) {
    if (Test-Path $vsw) {
        $vcInstallation = & $vsw -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>&1
        if ($vcInstallation -and (Test-Path $vcInstallation)) {
            $msvcFound = $true
            $msvcPath = $vcInstallation
            break
        }
    }
}

# Fallback check for standard Visual Studio 2022 directories
if (-not $msvcFound) {
    $vsStandardDirs = @(
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Professional",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Enterprise",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community"
    )
    foreach ($vsd in $vsStandardDirs) {
        $vcvars = Join-Path $vsd "VC\Auxiliary\Build\vcvarsall.bat"
        if (Test-Path $vcvars) {
            $msvcFound = $true
            $msvcPath = $vsd
            break
        }
    }
}

$clCmd = Get-Command cl -ErrorAction SilentlyContinue
if ($clCmd -or $msvcFound) {
    $clVer = if ($clCmd) { "In PATH (MSVC)" } else { "Installed at $msvcPath" }
    Register-AuditItem -Category "Native Toolchain" -Component "MSVC C++ Build Tools" -RequiredVersion "VS 2022 v143" -DetectedVersion $clVer -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Native Toolchain" -Component "MSVC C++ Build Tools" -RequiredVersion "VS 2022 v143" -DetectedVersion "Not Found" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Install via winget: 'winget install Microsoft.VisualStudio.2022.BuildTools --override \"--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended\"' or via https://visualstudio.microsoft.com/visual-cpp-build-tools/"
}

$sdkFound = $false
$sdkVersion = ""
$sdkBasePaths = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\Include",
    "${env:ProgramFiles}\Windows Kits\10\Include"
)
foreach ($sbp in $sdkBasePaths) {
    if (Test-Path $sbp) {
        $sdkDirs = @(Get-ChildItem -Path $sbp -Directory -ErrorAction SilentlyContinue)
        if ($sdkDirs.Count -gt 0) {
            $sdkFound = $true
            $sdkVersion = ($sdkDirs | Select-Object -Last 1).Name
            break
        }
    }
}
if ($sdkFound) {
    Register-AuditItem -Category "Native Toolchain" -Component "Windows 10/11 SDK" -RequiredVersion ">= 10.0.19041.0" -DetectedVersion $sdkVersion -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Native Toolchain" -Component "Windows 10/11 SDK" -RequiredVersion ">= 10.0.19041.0" -DetectedVersion "Not Found" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Install Windows 10/11 SDK (included in VS Build Tools C++ workload)."
}

# WebView2 Runtime Audit
$wv2Paths = @(
    "${env:ProgramFiles(x86)}\Microsoft\EdgeWebView\Application",
    "${env:ProgramFiles(x86)}\Microsoft\Edge\Application"
)
$wv2Found = $false
$wv2Ver = ""
foreach ($p in $wv2Paths) {
    if (Test-Path $p) {
        $dirs = Get-ChildItem -Path $p -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^[0-9]+\.[0-9]+' }
        if ($dirs) {
            $wv2Found = $true
            $wv2Ver = ($dirs | Select-Object -First 1).Name
            break
        }
    }
}

if ($wv2Found) {
    Register-AuditItem -Category "Desktop Shell" -Component "WebView2 Evergreen Runtime" -RequiredVersion ">= 110.0" -DetectedVersion $wv2Ver -Status "AVAILABLE" -Priority "REQUIRED" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Desktop Shell" -Component "WebView2 Evergreen Runtime" -RequiredVersion ">= 110.0" -DetectedVersion "Not Found" -Status "MISSING" -Priority "REQUIRED" -ActionRequired "Download and install Evergreen Bootstrapper: https://developer.microsoft.com/en-us/microsoft-edge/webview2/"
}

# -----------------------------------------------------------------------------
# 7. Ancillary & Optional Tools (CMake, SQLite CLI, Tauri CLI)
# -----------------------------------------------------------------------------
Write-Host "[7/7] Auditing Ancillary Development Tools..." -ForegroundColor Yellow

$tauriCliFound = $false
$tauriVer = ""
$cargoTauriCmd = Get-Command "cargo-tauri" -ErrorAction SilentlyContinue
if ($cargoTauriCmd) {
    $tauriCliFound = $true
    $tauriVer = "cargo-tauri installed"
} else {
    $globalTauri = Get-Command "tauri" -ErrorAction SilentlyContinue
    if ($globalTauri) {
        $tauriCliFound = $true
        $tauriVer = "tauri CLI in PATH"
    }
}

if ($tauriCliFound) {
    Register-AuditItem -Category "Desktop Shell" -Component "Tauri CLI (v2)" -RequiredVersion ">= 2.0.0" -DetectedVersion $tauriVer -Status "AVAILABLE" -Priority "OPTIONAL" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Desktop Shell" -Component "Tauri CLI (v2)" -RequiredVersion ">= 2.0.0" -DetectedVersion "Not Found in PATH" -Status "PARTIALLY AVAILABLE" -Priority "OPTIONAL" -ActionRequired "Can be run via 'pnpm tauri' or 'npx @tauri-apps/cli' once frontend is initialized."
}

$cmakeCmd = Get-Command cmake -ErrorAction SilentlyContinue
if ($cmakeCmd) {
    $cmakeVer = & cmake --version 2>&1 | Select-Object -First 1
    Register-AuditItem -Category "Native Toolchain" -Component "CMake" -RequiredVersion ">= 3.20" -DetectedVersion $cmakeVer -Status "AVAILABLE" -Priority "OPTIONAL" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Native Toolchain" -Component "CMake" -RequiredVersion ">= 3.20" -DetectedVersion "Not Found" -Status "NOT REQUIRED" -Priority "NOT REQUIRED" -ActionRequired "Only needed if custom native C++ libraries require CMake compilation."
}

$sqliteCmd = Get-Command sqlite3 -ErrorAction SilentlyContinue
if ($sqliteCmd) {
    $sqliteVer = & sqlite3 --version 2>&1
    Register-AuditItem -Category "Database Tools" -Component "SQLite3 CLI" -RequiredVersion ">= 3.40" -DetectedVersion $sqliteVer -Status "AVAILABLE" -Priority "OPTIONAL" -ActionRequired "None"
} else {
    Register-AuditItem -Category "Database Tools" -Component "SQLite3 CLI" -RequiredVersion ">= 3.40" -DetectedVersion "Not Found in PATH" -Status "NOT REQUIRED" -Priority "NOT REQUIRED" -ActionRequired "Hime uses embedded SQLite via rusqlite (bundled); CLI tool is purely optional for debugging."
}

Write-Host ""
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  AUDIT RESULTS SUMMARY" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

$auditResults | Format-Table -Property Category, Component, Status, Priority, Detected, ActionRequired -AutoSize

Write-Host ""
if ($script:hasMissingRequired) {
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor Red
    Write-Host "  VERDICT: READINESS AUDIT FAILED - MISSING REQUIRED TOOLCHAINS" -ForegroundColor Red
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor Red
    Write-Host "The following required toolchains are missing and must be installed by Project Owner:" -ForegroundColor Red
    $missingItems = $script:auditResults | Where-Object { $_.Priority -eq "REQUIRED" -and $_.Status -ne "AVAILABLE" }
    foreach ($item in $missingItems) {
        Write-Host "  - $($item.Component): $($item.ActionRequired)" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "CONSOLIDATED OWNER ACTION REQUIRED TO UNBLOCK PHASE 06:" -ForegroundColor Cyan
    Write-Host "Option A (Interactive Installers):" -ForegroundColor White
    Write-Host "  1. Visual Studio 2022 Build Tools (C++ Workload): https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host "  2. Rustup (MSVC target default): https://rustup.rs/" -ForegroundColor Gray
    Write-Host "Option B (Windows Package Manager / WinGet via Admin Terminal):" -ForegroundColor White
    Write-Host "  winget install Microsoft.VisualStudio.2022.BuildTools --override `"--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended`"" -ForegroundColor Gray
    Write-Host "  winget install Rustlang.Rustup" -ForegroundColor Gray
    Write-Host ""
    Write-Host "After installation, re-run: .\scripts\check-env.ps1" -ForegroundColor White
    Write-Host ""
    exit 1
} else {
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor Green
    Write-Host "  VERDICT: ALL REQUIRED PREREQUISITES AVAILABLE (READINESS VERIFIED)" -ForegroundColor Green
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor Green
    Write-Host "The host environment is ready for Phase 06 implementation." -ForegroundColor Green
    Write-Host ""
    exit 0
}
