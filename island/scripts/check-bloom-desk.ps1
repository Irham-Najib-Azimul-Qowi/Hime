Add-Type -Path X:\folder_ai\Hime-Bloom\scripts\ProcDesk.cs
$threads = (Get-Process -Id 4436).Threads | Select-Object -First 3
foreach ($t in $threads) {
    [ProcDesk]::CheckProc($t.Id)
}
