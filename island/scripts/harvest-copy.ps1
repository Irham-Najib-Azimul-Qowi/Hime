# Harvest and Copy Script for Stage H1
$ErrorActionPreference = "Stop"

$srcRoot = "X:\folder_ai\Hime"
$dstRoot = "X:\folder_ai\Hime-Bloom"

Write-Host "=== Starting Hime Legacy Harvest & Copy ==="

# 1. Directories setup
$dirs = @(
    "$dstRoot\core\crates",
    "$dstRoot\mobile",
    "$dstRoot\docs\architecture",
    "$dstRoot\docs\adr",
    "$dstRoot\docs\security",
    "$dstRoot\docs\environment",
    "$dstRoot\docs\product\requirements",
    "$dstRoot\docs\product\master-plan",
    "$dstRoot\docs\history\phases",
    "$dstRoot\docs\history\reports",
    "$dstRoot\docs\reference\nusa",
    "$dstRoot\docs\reference\hermes",
    "$dstRoot\docs\reference\9router",
    "$dstRoot\docs\agent-pack",
    "$dstRoot\.agent\skills"
)

foreach ($d in $dirs) {
    if (-not (Test-Path $d)) {
        New-Item -ItemType Directory -Path $d -Force | Out-Null
    }
}

# 2. Copy crates
Write-Host "Copying crates to core/crates/..."
Copy-Item -Path "$srcRoot\crates\*" -Destination "$dstRoot\core\crates\" -Recurse -Force

# 3. Copy mobile
Write-Host "Copying apps/mobile to mobile/..."
Copy-Item -Path "$srcRoot\apps\mobile\*" -Destination "$dstRoot\mobile\" -Recurse -Force

# 4. Adapt core/Cargo.toml
Write-Host "Creating core/Cargo.toml..."
$cargoToml = Get-Content -Path "$srcRoot\Cargo.toml" -Raw
# Replace members to remove apps/hime-desktop/src-tauri
$cargoTomlAdapted = $cargoToml -replace 'members = \[\r?\n\s*"crates/\*",\r?\n\s*"apps/hime-desktop/src-tauri",?\r?\n\s*\]', 'members = ["crates/*"]'
Set-Content -Path "$dstRoot\core\Cargo.toml" -Value $cargoTomlAdapted -Encoding UTF8

# Copy Cargo.lock to core/Cargo.lock
Copy-Item -Path "$srcRoot\Cargo.lock" -Destination "$dstRoot\core\Cargo.lock" -Force

# 5. Copy docs
Write-Host "Copying documentation..."
if (Test-Path "$srcRoot\docs\architecture") {
    Copy-Item -Path "$srcRoot\docs\architecture\*" -Destination "$dstRoot\docs\architecture\" -Recurse -Force
}
if (Test-Path "$srcRoot\docs\adr") {
    # Copy ADR 001-012, avoid overwriting new Bloom ADRs 013+
    Get-ChildItem -Path "$srcRoot\docs\adr" -Filter "*.md" | ForEach-Object {
        Copy-Item -Path $_.FullName -Destination "$dstRoot\docs\adr\" -Force
    }
}
if (Test-Path "$srcRoot\docs\security") {
    Copy-Item -Path "$srcRoot\docs\security\*" -Destination "$dstRoot\docs\security\" -Recurse -Force
}
if (Test-Path "$srcRoot\docs\environment") {
    Copy-Item -Path "$srcRoot\docs\environment\*" -Destination "$dstRoot\docs\environment\" -Recurse -Force
}
if (Test-Path "$srcRoot\docs\master-plan") {
    Copy-Item -Path "$srcRoot\docs\master-plan\*" -Destination "$dstRoot\docs\product\master-plan\" -Recurse -Force
}
if (Test-Path "$srcRoot\project-management\product-requirements") {
    Copy-Item -Path "$srcRoot\project-management\product-requirements\*" -Destination "$dstRoot\docs\product\requirements\" -Recurse -Force
}
if (Test-Path "$srcRoot\project-management\phases") {
    Copy-Item -Path "$srcRoot\project-management\phases\*" -Destination "$dstRoot\docs\history\phases\" -Recurse -Force
}
if (Test-Path "$srcRoot\docs\reports") {
    Copy-Item -Path "$srcRoot\docs\reports\*" -Destination "$dstRoot\docs\history\reports\" -Recurse -Force
}

# 6. Copy RE
Write-Host "Copying RE analysis documents..."
if (Test-Path "$srcRoot\RE\Nusa") {
    Copy-Item -Path "$srcRoot\RE\Nusa\*" -Destination "$dstRoot\docs\reference\nusa\" -Recurse -Force
}
if (Test-Path "$srcRoot\RE\Hermes") {
    Copy-Item -Path "$srcRoot\RE\Hermes\*" -Destination "$dstRoot\docs\reference\hermes\" -Recurse -Force
}
if (Test-Path "$srcRoot\RE\9ROUTER") {
    Copy-Item -Path "$srcRoot\RE\9ROUTER\*" -Destination "$dstRoot\docs\reference\9router\" -Recurse -Force
}

# 7. Copy Agent Pack & Skills
Write-Host "Copying Agent Pack and Skills..."
if (Test-Path "$srcRoot\hime-bloom-pack") {
    Copy-Item -Path "$srcRoot\hime-bloom-pack\*" -Destination "$dstRoot\docs\agent-pack\" -Recurse -Force
}
if (Test-Path "$srcRoot\.agent\skills") {
    Copy-Item -Path "$srcRoot\.agent\skills\*" -Destination "$dstRoot\.agent\skills\" -Recurse -Force
}

Write-Host "=== Files copied. Verifying SHA-256 for core/crates... ==="

$hashLog = @()
$cratesFiles = Get-ChildItem -Path "$dstRoot\core\crates" -Recurse -File
$matchCount = 0
$mismatchCount = 0

foreach ($dstFile in $cratesFiles) {
    $relPath = $dstFile.FullName.Substring(("$dstRoot\core\").Length)
    $srcFilePath = Join-Path $srcRoot $relPath
    if (Test-Path $srcFilePath) {
        $srcHash = (Get-FileHash -Path $srcFilePath -Algorithm SHA256).Hash
        $dstHash = (Get-FileHash -Path $dstFile.FullName -Algorithm SHA256).Hash
        if ($srcHash -eq $dstHash) {
            $matchCount++
            $hashLog += [PSCustomObject]@{
                Path = $relPath
                Status = "MATCH"
                SHA256 = $srcHash
            }
        } else {
            $mismatchCount++
            $hashLog += [PSCustomObject]@{
                Path = $relPath
                Status = "MISMATCH"
                SHA256 = "$srcHash vs $dstHash"
            }
        }
    }
}

Write-Host "Crates Verification: $matchCount matches, $mismatchCount mismatches."
$hashLog | Export-Csv -Path "$dstRoot\docs\hime\harvest\HASH-VERIFICATION.csv" -NoTypeInformation

Write-Host "=== Harvest & Copy Complete ==="
