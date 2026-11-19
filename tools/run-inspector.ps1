try {
    $res = [WinInspector2]::GetAll()
    Write-Host "Count: $($res.Length)"
    foreach ($line in $res) {
        Write-Host $line
    }
} catch {
    Write-Host "Error: $_"
}
