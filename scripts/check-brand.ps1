<#
.SYNOPSIS
    Hime Island Brand Compliance Gate (scripts/check-brand.ps1)
.DESCRIPTION
    Scans the repository for unauthorized occurrences of "Bloom" in user-facing UI,
    metadata, binary definitions, and strings.
    Ensures that productName, identifier, window titles, and user-facing artifacts adhere strictly
    to the Hime Island product identity (03-BRANDING-HIME-ISLAND.md).
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
Write-Host "  HIME ISLAND BRAND COMPLIANCE AUDIT" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# 1. Check tauri.conf.json identity
$tauriConfPath = Join-Path $repoRoot "island/src-tauri/tauri.conf.json"
if (Test-Path $tauriConfPath) {
    $tauriConf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
    if ($tauriConf.productName -ne "Hime Island") {
        $violations += "tauri.conf.json productName is '$($tauriConf.productName)', expected 'Hime Island'"
    }
    if ($tauriConf.identifier -ne "local.hime.island") {
        $violations += "tauri.conf.json identifier is '$($tauriConf.identifier)', expected 'local.hime.island'"
    }
    foreach ($win in $tauriConf.app.windows) {
        if ($win.title -like "*Bloom*") {
            $violations += "tauri.conf.json window '$($win.label)' title contains 'Bloom': '$($win.title)'"
        }
    }
} else {
    $violations += "Missing island/src-tauri/tauri.conf.json"
}

# 2. Check island package.json identity
$pkgJsonPath = Join-Path $repoRoot "island/package.json"
if (Test-Path $pkgJsonPath) {
    $pkgJson = Get-Content $pkgJsonPath -Raw | ConvertFrom-Json
    if ($pkgJson.name -ne "hime-island") {
        $violations += "island/package.json name is '$($pkgJson.name)', expected 'hime-island'"
    }
}

# 3. Whitelisted paths for GPL-3.0 attribution and legacy reference
$whitelist = @(
    "NOTICE",
    "LICENSE",
    "docs/legal/",
    "docs/hime/harvest/",
    "docs/agent-pack/",
    "docs/hime/UPSTREAM.md",
    "scripts/check-brand.ps1"
)

# 4. Scan active source directories for user-facing UI and logic brand leaks
$targetDirs = @(
    "island/src",
    "core/crates",
    "modules",
    "contracts",
    "design",
    "scripts"
)

foreach ($dir in $targetDirs) {
    $fullDir = Join-Path $repoRoot $dir
    if (-not (Test-Path $fullDir)) { continue }

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
                # Permissible GPL header or attribution text
                if ($line -match "derivative work based upon Bloom" -or $line -match "GPL-3.0") {
                    continue
                }
                # Internal bloom-* settings keys or legacy CSS class names migrating in phase U1 are permitted
                if ($line -match '["'']bloom-' -or $line -match 'bloom-[a-z0-9_-]+' -or $line -match 'className=\{?`?bloom\b') {
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
