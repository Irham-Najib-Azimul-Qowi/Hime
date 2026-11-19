# setup-shortcuts.ps1 - Configure Desktop Shortcuts, Start Menu, Autostart & Hotkeys
$ErrorActionPreference = "SilentlyContinue"

$WshShell = New-Object -ComObject WScript.Shell

$desktopDir = [System.Environment]::GetFolderPath('Desktop')
$startMenuDir = Join-Path ([System.Environment]::GetFolderPath('Programs')) "Hime"
$startupDir = [System.Environment]::GetFolderPath('Startup')

if (-not (Test-Path $startMenuDir)) {
    New-Item -ItemType Directory -Path $startMenuDir -Force | Out-Null
}

$himeRoot = "X:\folder_ai\Hime"
$iconHime = "$himeRoot\island\src-tauri\icons\icon.ico"
$psExe = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"

$hotkeyExe = "$himeRoot\island\src-tauri\target\release\hime_hotkey.exe"
if (-not (Test-Path $hotkeyExe)) {
    $hotkeyExe = "$himeRoot\island\src-tauri\target\debug\hime_hotkey.exe"
}

# 1. Desktop: Hidupkan Hime (Ctrl+Shift+H)
$scStartDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Hidupkan Hime.lnk"))
$scStartDesktop.TargetPath = $psExe
$scStartDesktop.Arguments = "-WindowStyle Hidden -ExecutionPolicy Bypass -File `"$himeRoot\scripts\start-hime.ps1`""
$scStartDesktop.WorkingDirectory = $himeRoot
$scStartDesktop.Hotkey = "Ctrl+Shift+H"
if (Test-Path $iconHime) { $scStartDesktop.IconLocation = $iconHime }
$scStartDesktop.Description = "Hidupkan Hime Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scStartDesktop.Save()

# 2. Desktop: Matikan Hime (Ctrl+Shift+Q)
$scStopDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Matikan Hime.lnk"))
$scStopDesktop.TargetPath = $psExe
$scStopDesktop.Arguments = "-WindowStyle Hidden -ExecutionPolicy Bypass -File `"$himeRoot\scripts\stop-hime.ps1`""
$scStopDesktop.WorkingDirectory = $himeRoot
$scStopDesktop.Hotkey = "Ctrl+Shift+Q"
$scStopDesktop.IconLocation = "$env:SystemRoot\System32\shell32.dll,27"
$scStopDesktop.Description = "Matikan Hime Ecosystem (Ctrl+Shift+Q / Ctrl+Alt+Q)"
$scStopDesktop.Save()

# 3. Start Menu Programs\Hime
$scStartMenu = $WshShell.CreateShortcut((Join-Path $startMenuDir "Hidupkan Hime.lnk"))
$scStartMenu.TargetPath = $psExe
$scStartMenu.Arguments = "-WindowStyle Hidden -ExecutionPolicy Bypass -File `"$himeRoot\scripts\start-hime.ps1`""
$scStartMenu.WorkingDirectory = $himeRoot
$scStartMenu.Hotkey = "Ctrl+Shift+H"
if (Test-Path $iconHime) { $scStartMenu.IconLocation = $iconHime }
$scStartMenu.Description = "Hidupkan Hime Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scStartMenu.Save()

$scStopMenu = $WshShell.CreateShortcut((Join-Path $startMenuDir "Matikan Hime.lnk"))
$scStopMenu.TargetPath = $psExe
$scStopMenu.Arguments = "-WindowStyle Hidden -ExecutionPolicy Bypass -File `"$himeRoot\scripts\stop-hime.ps1`""
$scStopMenu.WorkingDirectory = $himeRoot
$scStopMenu.Hotkey = "Ctrl+Shift+Q"
$scStopMenu.IconLocation = "$env:SystemRoot\System32\shell32.dll,27"
$scStopMenu.Description = "Matikan Hime Ecosystem (Ctrl+Shift+Q / Ctrl+Alt+Q)"
$scStopMenu.Save()

# 4. Startup Folder: Autostart Hime & Hotkey Daemon
$scStartup = $WshShell.CreateShortcut((Join-Path $startupDir "Hime.lnk"))
$scStartup.TargetPath = $psExe
$scStartup.Arguments = "-WindowStyle Hidden -ExecutionPolicy Bypass -File `"$himeRoot\scripts\start-hime.ps1`" -Autostart"
$scStartup.WorkingDirectory = $himeRoot
if (Test-Path $iconHime) { $scStartup.IconLocation = $iconHime }
$scStartup.Description = "Hime Ecosystem Persistent Autostart"
$scStartup.Save()

# 5. Registry HKCU Run
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "Hime" -Value "`"$psExe`" -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$himeRoot\scripts\start-hime.ps1`" -Autostart"
if (Test-Path $hotkeyExe) {
    Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "HimeHotkey" -Value "`"$hotkeyExe`""
}

Write-Host "Hime Desktop shortcuts, Start Menu, Autostart & Hotkeys configured successfully!" -ForegroundColor Green
