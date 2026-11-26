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
$wscriptExe = "$env:SystemRoot\System32\wscript.exe"
$launcherVbs = "$himeRoot\tools\hime_launcher.vbs"

$hotkeyExe = "$himeRoot\island\src-tauri\target\release\hime_hotkey.exe"
if (-not (Test-Path $hotkeyExe)) {
    $hotkeyExe = "$himeRoot\island\src-tauri\target\debug\hime_hotkey.exe"
}

# 1. Desktop: Hidupkan Hime (Zero console flash via wscript runner)
$scStartDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Hidupkan Hime.lnk"))
$scStartDesktop.TargetPath = $wscriptExe
$scStartDesktop.Arguments = "`"$launcherVbs`" start"
$scStartDesktop.WorkingDirectory = $himeRoot
$scStartDesktop.WindowStyle = 7
$scStartDesktop.Hotkey = ""
if (Test-Path $iconHime) { $scStartDesktop.IconLocation = $iconHime }
$scStartDesktop.Description = "Hidupkan Hime Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scStartDesktop.Save()

# 1b. Desktop: Hime (Zero console flash via wscript runner)
$scHimeDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Hime.lnk"))
$scHimeDesktop.TargetPath = $wscriptExe
$scHimeDesktop.Arguments = "`"$launcherVbs`" start"
$scHimeDesktop.WorkingDirectory = $himeRoot
$scHimeDesktop.WindowStyle = 7
$scHimeDesktop.Hotkey = ""
if (Test-Path $iconHime) { $scHimeDesktop.IconLocation = $iconHime }
$scHimeDesktop.Description = "Hime Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scHimeDesktop.Save()

# 2. Desktop: Matikan Hime (Zero console flash via wscript runner)
$scStopDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Matikan Hime.lnk"))
$scStopDesktop.TargetPath = $wscriptExe
$scStopDesktop.Arguments = "`"$launcherVbs`" stop"
$scStopDesktop.WorkingDirectory = $himeRoot
$scStopDesktop.WindowStyle = 7
$scStopDesktop.Hotkey = ""
$scStopDesktop.IconLocation = "$env:SystemRoot\System32\shell32.dll,27"
$scStopDesktop.Description = "Matikan Hime Ecosystem (Ctrl+Shift+Q)"
$scStopDesktop.Save()

# 3. Start Menu Programs\Hime
$scStartMenu = $WshShell.CreateShortcut((Join-Path $startMenuDir "Hidupkan Hime.lnk"))
$scStartMenu.TargetPath = $wscriptExe
$scStartMenu.Arguments = "`"$launcherVbs`" start"
$scStartMenu.WorkingDirectory = $himeRoot
$scStartMenu.WindowStyle = 7
$scStartMenu.Hotkey = ""
if (Test-Path $iconHime) { $scStartMenu.IconLocation = $iconHime }
$scStartMenu.Description = "Hidupkan Hime Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scStartMenu.Save()

$scStopMenu = $WshShell.CreateShortcut((Join-Path $startMenuDir "Matikan Hime.lnk"))
$scStopMenu.TargetPath = $wscriptExe
$scStopMenu.Arguments = "`"$launcherVbs`" stop"
$scStopMenu.WorkingDirectory = $himeRoot
$scStopMenu.WindowStyle = 7
$scStopMenu.Hotkey = ""
$scStopMenu.IconLocation = "$env:SystemRoot\System32\shell32.dll,27"
$scStopMenu.Description = "Matikan Hime Ecosystem (Ctrl+Shift+Q)"
$scStopMenu.Save()

# 4. Startup Folder: Autostart Hime
$staleHotkeyLnk = Join-Path $startupDir "HimeHotkey.lnk"
if (Test-Path $staleHotkeyLnk) {
    Remove-Item $staleHotkeyLnk -Force -ErrorAction SilentlyContinue
}
$scStartup = $WshShell.CreateShortcut((Join-Path $startupDir "Hime.lnk"))
$scStartup.TargetPath = $wscriptExe
$scStartup.Arguments = "`"$launcherVbs`" start-autostart"
$scStartup.WorkingDirectory = $himeRoot
if (Test-Path $iconHime) { $scStartup.IconLocation = $iconHime }
$scStartup.Description = "Hime Ecosystem Persistent Autostart"
$scStartup.Save()

# 5. Registry HKCU Run
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "Hime" -Value 'wscript.exe "X:\folder_ai\Hime\tools\hime_launcher.vbs" start-autostart'
if (Test-Path $hotkeyExe) {
    Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "HimeHotkey" -Value "`"$hotkeyExe`""
}

Write-Host "Hime Desktop shortcuts, Start Menu, Autostart & Hotkeys configured successfully!" -ForegroundColor Green
