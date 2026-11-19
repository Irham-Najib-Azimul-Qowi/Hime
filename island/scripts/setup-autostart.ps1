$startupDir = [Environment]::GetFolderPath('Startup')
$shortcutPath = Join-Path $startupDir "Hime.lnk"
$ws = New-Object -ComObject WScript.Shell
$shortcut = $ws.CreateShortcut($shortcutPath)
$shortcut.TargetPath = "X:\folder_ai\start-hime.bat"
$shortcut.WorkingDirectory = "X:\folder_ai"
$shortcut.Description = "Hime Personal AI Ecosystem Launcher"
$shortcut.IconLocation = "X:\folder_ai\Hime-Bloom\src-tauri\icons\icon.ico"
$shortcut.Save()
Write-Host "Created startup shortcut at: $shortcutPath"
Test-Path $shortcutPath
