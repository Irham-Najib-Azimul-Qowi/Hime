<#
.SYNOPSIS
    Hime Island Brand Compliance Gate (scripts/check-brand.ps1)
.DESCRIPTION
    Scans active pure-Rust source, contracts, modules, design, and scripts for
    unauthorized occurrences of "Bloom" in user-facing UI, metadata, and strings.
#>

[CmdletBinding()]
param(
    [switch]$Detailed
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$violations = @()

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  HIME ISLAND BRAND COMPLIANCE AUDIT (PURE-RUST NATIVE)" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# 1. Check island/native/Cargo.toml identity
$nativeCargoPath = Join-Path $repoRoot "island/native/Cargo.toml"
if (Test-Path $nativeCargoPath) {
    $nativeCargo = Get-Content $nativeCargoPath -Raw
    if ($nativeCargo -notmatch 'name\s*=\s*"hime-island-native"') {
        $violations += "island/native/Cargo.toml package name expected 'hime-island-native'"
    }
    if ($nativeCargo -match '(?im)^description\s*=.*\bbloom\b') {
        $violations += "island/native/Cargo.toml description contains 'Bloom'"
    }
} else {
    $violations += "Missing mandatory file: island/native/Cargo.toml"
}

# 2. Check core/Cargo.toml presence
$coreCargoPath = Join-Path $repoRoot "core/Cargo.toml"
if (-not (Test-Path $coreCargoPath)) {
    $violations += "Missing mandatory file: core/Cargo.toml"
}

# 3. Whitelisted paths for GPL-3.0 attribution and legacy reference
$whitelist = @(
    "NOTICE",
    "LICENSE",
    "Archive/",
    "docs/legal/",
    "docs/hime/harvest/",
    "docs/agent-pack/",
    "docs/hime/UPSTREAM.md",
    "scripts/check-brand.ps1"
)

# 4. Scan active pure-Rust source directories and metadata for brand leaks
$targetDirs = @(
    "island/native/src",
    "core/crates",
    "modules",
    "contracts",
    "design",
    "scripts",
    "tools"
)

foreach ($dir in $targetDirs) {
    $fullDir = Join-Path $repoRoot $dir
    if (-not (Test-Path $fullDir)) {
        $violations += "Mandatory scan directory does not exist: $dir"
        continue
    }

    $files = Get-ChildItem -Path $fullDir -Recurse -File -Exclude "*.lock", "*.png", "*.ico", "*.svg", "*.woff*", "*.ttf", "*.css"
    foreach ($file in $files) {
        $relPath = $file.FullName.Substring($repoRoot.Length + 1).Replace("\", "/")

        $isWhitelisted = $false
        foreach ($w in $whitelist) {
            if ($relPath.StartsWith($w)) {
                $isWhitelisted = $true
                break
            }
        }
        if ($isWhitelisted) { continue }

        $content = Get-Content -Path $file.FullName
        $lineNum = 0
        foreach ($line in $content) {
            $lineNum++
            $trimmed = $line.Trim()

            # Allow code comments referencing upstream Bloom algorithms / attribution
            if ($trimmed.StartsWith("//") -or $trimmed.StartsWith("/*") -or $trimmed.StartsWith("*") -or $trimmed.StartsWith("///") -or $trimmed.StartsWith("#")) {
                continue
            }

            if ($line -match "(?i)\bbloom\b") {
                # Permissible GPL header or attribution text, or persona rejection tests
                if ($line -match "derivative work based upon Bloom" -or $line -match "Derivative of Bloom" -or $line -match "GNU General Public License v3\.0" -or $line -match "GPL-3\.0" -or $line -match 'lower\.contains\("bloom"\)' -or $line -match 'Apakah kamu Bloom') {
                    continue
                }
                # Internal legacy-compatible bloom-* settings keys are permitted
                if ($line -match '["'']bloom-' -or $line -match 'bloom-[a-z0-9_-]+') {
                    continue
                }
                $violations += "${relPath}:${lineNum} - Found unwhitelisted brand token: $($trimmed)"
            }
        }
    }
}

# 5. Report results
if ($violations.Count -eq 0) {
    Write-Host "[OK] Brand compliance check PASSED! Zero unwhitelisted 'Bloom' brand tokens." -ForegroundColor Green
    exit 0
} else {
    Write-Host "[FAIL] Brand compliance check FAILED with $($violations.Count) violation(s):" -ForegroundColor Red
    foreach ($v in $violations) {
        Write-Host "  - $v" -ForegroundColor Yellow
    }
    exit 1
}
