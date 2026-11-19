$shell = New-Object -ComObject Shell.Application
$shell.ToggleDesktop()
Start-Sleep -Milliseconds 600

# Now run test-reveal-dock.ps1
& 'X:\folder_ai\Hime-Bloom\scripts\test-reveal-dock.ps1'
