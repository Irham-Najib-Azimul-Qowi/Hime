<#
.SYNOPSIS
    Hime Island Code Hygiene & Quality Gate (scripts/check-hygiene.ps1)
.DESCRIPTION
    Enforces quality standards across pure-Rust Core & Native Island:
    - No TODO/FIXME without issue ID (e.g., TODO(H-123))
    - No production todo!() / unimplemented!() macros in Rust source
    - No stray temporary log/dump artifacts in root or island/
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
Write-Host "  HIME ISLAND CODE HYGIENE AUDIT (PURE-RUST NATIVE)" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# 1. Check for unanchored task anchors (must have issue ID like TODO(H-101))
$scanDirs = @(
    "island/native/src",
    "core/crates",
    "modules",
    "contracts",
    "design",
    "scripts",
    "tools"
)

foreach ($dir in $scanDirs) {
    $fullDir = Join-Path $repoRoot $dir
    if (-not (Test-Path $fullDir)) {
        $violations += "Mandatory scan directory does not exist: $dir"
        continue
    }

    $files = Get-ChildItem -Path $fullDir -Recurse -File -Exclude "*.lock", "*.png", "*.ico", "*.svg", "*.woff*", "*.ttf", "*.css"
    foreach ($file in $files) {
        $relPath = $file.FullName.Substring($repoRoot.Length + 1).Replace("\", "/")
        if ($relPath -eq "scripts/check-hygiene.ps1") { continue }

        $lines = Get-Content -Path $file.FullName
        $lineNum = 0
        foreach ($line in $lines) {
            $lineNum++
            $trimmed = $line.Trim()

            # Check for TODO / FIXME in comments without issue ID
            $isComment = $trimmed.StartsWith("//") -or $trimmed.StartsWith("/*") -or $trimmed.StartsWith("*") -or $trimmed.StartsWith("#")
            if ($isComment -and $trimmed -match '\b(TODO|FIXME)\b' -and $trimmed -notmatch '\b(TODO|FIXME)\([A-Za-z0-9_-]+\)') {
                $violations += "${relPath}:${lineNum} - Unanchored TODO/FIXME without issue ID: $($trimmed)"
            }
        }
    }
}

# 2. Check for leftover mock placeholders in production Rust source
$rustScanDirs = @(
    "core/crates",
    "island/native/src"
)

foreach ($rDir in $rustScanDirs) {
    $fullRDir = Join-Path $repoRoot $rDir
    if (-not (Test-Path $fullRDir)) {
        $violations += "Mandatory Rust scan directory does not exist: $rDir"
        continue
    }

    $rustFiles = Get-ChildItem -Path $fullRDir -Recurse -Filter "*.rs"
    foreach ($file in $rustFiles) {
        $relPath = $file.FullName.Substring($repoRoot.Length + 1).Replace("\", "/")
        if ($relPath -match "/tests/" -or $relPath -match "tests\.rs") { continue }

        $lines = Get-Content -Path $file.FullName
        $lineNum = 0
        foreach ($line in $lines) {
            $lineNum++
            $trimmed = $line.Trim()
            if ($trimmed -match 'unimplemented!\(' -or $trimmed -match 'todo!\(') {
                $violations += "${relPath}:${lineNum} - Production unhandled macro: $($trimmed)"
            }
        }
    }
}

# 3. Check for stray temporary log/dump files in root and island
$strayCheckPatterns = @("*.log", "stderr.txt", "stdout.txt", "*_survey.png")
$strayCheckDirs = @($repoRoot, (Join-Path $repoRoot "island"))
foreach ($sDir in $strayCheckDirs) {
    if (-not (Test-Path $sDir)) {
        $relSDir = if ($sDir -eq $repoRoot) { "." } else { $sDir.Substring($repoRoot.Length + 1).Replace("\", "/") }
        $violations += "Mandatory scan directory does not exist: $relSDir"
        continue
    }
    foreach ($pattern in $strayCheckPatterns) {
        $foundStrays = Get-ChildItem -Path $sDir -Filter $pattern -File -ErrorAction SilentlyContinue
        foreach ($sFile in $foundStrays) {
            $relPath = $sFile.FullName.Substring($repoRoot.Length + 1).Replace("\", "/")
            $violations += "${relPath} - Stray temporary debug artifact detected"
        }
    }
}

# 4. Report results
if ($violations.Count -eq 0) {
    Write-Host "[OK] Code hygiene check PASSED! Zero hygiene violations found." -ForegroundColor Green
    exit 0
} else {
    Write-Host "[FAIL] Code hygiene check FAILED with $($violations.Count) violation(s):" -ForegroundColor Red
    foreach ($v in $violations) {
        Write-Host "  - $v" -ForegroundColor Yellow
    }
    exit 1
}
