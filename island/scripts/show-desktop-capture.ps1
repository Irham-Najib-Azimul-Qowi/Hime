$sh = New-Object -ComObject Shell.Application
$sh.MinimizeAll()
Start-Sleep -Milliseconds 800
& "X:\folder_ai\Hime-Bloom\scripts\capture-now.ps1"
