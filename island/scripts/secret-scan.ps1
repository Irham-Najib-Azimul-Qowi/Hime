# Secret Scan Script
$coreFiles = Get-ChildItem -Path "X:\folder_ai\Hime-Bloom\core" -Recurse -File -Include *.rs, *.toml
$patterns = @(
    "AKIA[0-9A-Z]{16}",
    "ghp_[a-zA-Z0-9]{36}",
    "sk-[a-zA-Z0-9]{32,}",
    "-----BEGIN (RSA|EC|OPENSSH|PRIVATE) KEY-----"
)

$found = @()
foreach ($file in $coreFiles) {
    foreach ($pat in $patterns) {
        $hits = Select-String -Path $file.FullName -Pattern $pat -ErrorAction SilentlyContinue
        if ($hits) {
            foreach ($h in $hits) {
                # Exclude mock test patterns and intentional security test assertions
                if ($h.Line -notmatch "test|mock|dummy|placeholder|example|GH_TOKEN|AWS_SECRET_KEY|Bearer sk-1234567890abcdef|sk-1234567890" -and $file.FullName -notmatch "tests\\") {
                    $found += [PSCustomObject]@{
                        File = $file.Name
                        Line = $h.LineNumber
                        Match = $h.Line.Trim()
                    }
                }
            }
        }
    }
}

if ($found.Count -eq 0) {
    Write-Host "SECRET SCAN CLEAN: 0 real credentials or private keys detected in harvested core files." -ForegroundColor Green
    exit 0
} else {
    Write-Host "WARNING: Potential secrets found: $($found.Count)" -ForegroundColor Red
    $found | Format-Table -AutoSize
    exit 1
}
