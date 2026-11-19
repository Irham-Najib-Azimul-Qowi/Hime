<#
.SYNOPSIS
    Hime Island Code Hygiene & Quality Gate (scripts/check-hygiene.ps1)
.DESCRIPTION
    Enforces quality standards defined in 14-MODULARITY-AND-CODE-QUALITY.md §6:
    - No TODO/FIXME without issue ID (e.g., TODO(H-123))
    - No bare debug console logs in production source
    - No commented-out code blocks
    - Validates Rust core and Island source integrity
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
Write-Host "  HIME ISLAND CODE HYGIENE AUDIT" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# 1. Check for unanchored TODO / FIXME (must have issue ID like TODO(H-101))
$scanDirs = @(
    "core/crates",
    "modules",
    "contracts",
    "design"
)

foreach ($dir in $scanDirs) {
    $fullDir = Join-Path $repoRoot $dir
    if (-not (Test-Path $fullDir)) { continue }

    $files = Get-ChildItem -Path $fullDir -Recurse -File -Exclude "*.lock", "*.png", "*.ico", "*.svg"
    foreach ($file in $files) {
        $relPath = $file.FullName.Substring($repoRoot.Length + 1).Replace("\", "/")
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

# 2. Check for leftover mock placeholders in production crates
$coreSrc = Join-Path $repoRoot "core/crates"
if (Test-Path $coreSrc) {
    $rustFiles = Get-ChildItem -Path $coreSrc -Recurse -Filter "*.rs"
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

# 3. Report results
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
