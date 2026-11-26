# setup-shortcuts.ps1 - Configure Desktop Shortcuts, Start Menu & Autostart for Hime Native
$ErrorActionPreference = "SilentlyContinue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$WshShell = New-Object -ComObject WScript.Shell

$desktopDir = [System.Environment]::GetFolderPath('Desktop')
$startMenuDir = Join-Path ([System.Environment]::GetFolderPath('Programs')) "Hime"
$startupDir = [System.Environment]::GetFolderPath('Startup')

if (-not (Test-Path $startMenuDir)) {
    New-Item -ItemType Directory -Path $startMenuDir -Force | Out-Null
}

$iconHime = Join-Path $repoRoot "island\native\icons\icon.ico"
if (-not (Test-Path $iconHime)) {
    $iconHime = Join-Path $repoRoot "Archive\legacy-webview2-island\src-tauri\icons\icon.ico"
}
$wscriptExe = "$env:SystemRoot\System32\wscript.exe"
$launcherVbs = Join-Path $repoRoot "tools\hime_launcher.vbs"

# 1. Desktop: Hidupkan Hime (Zero console flash via wscript runner)
$scStartDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Hidupkan Hime.lnk"))
$scStartDesktop.TargetPath = $wscriptExe
$scStartDesktop.Arguments = "`"$launcherVbs`" start"
$scStartDesktop.WorkingDirectory = $repoRoot
$scStartDesktop.WindowStyle = 7
$scStartDesktop.Hotkey = ""
if (Test-Path $iconHime) { $scStartDesktop.IconLocation = $iconHime }
$scStartDesktop.Description = "Hidupkan Hime Native Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scStartDesktop.Save()

# 1b. Desktop: Hime (Zero console flash via wscript runner)
$scHimeDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Hime.lnk"))
$scHimeDesktop.TargetPath = $wscriptExe
$scHimeDesktop.Arguments = "`"$launcherVbs`" start"
$scHimeDesktop.WorkingDirectory = $repoRoot
$scHimeDesktop.WindowStyle = 7
$scHimeDesktop.Hotkey = ""
if (Test-Path $iconHime) { $scHimeDesktop.IconLocation = $iconHime }
$scHimeDesktop.Description = "Hime Native Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scHimeDesktop.Save()

# 2. Desktop: Matikan Hime (Zero console flash via wscript runner)
$scStopDesktop = $WshShell.CreateShortcut((Join-Path $desktopDir "Matikan Hime.lnk"))
$scStopDesktop.TargetPath = $wscriptExe
$scStopDesktop.Arguments = "`"$launcherVbs`" stop"
$scStopDesktop.WorkingDirectory = $repoRoot
$scStopDesktop.WindowStyle = 7
$scStopDesktop.Hotkey = ""
$scStopDesktop.IconLocation = "$env:SystemRoot\System32\shell32.dll,27"
$scStopDesktop.Description = "Matikan Hime Native Ecosystem (Ctrl+Shift+Q)"
$scStopDesktop.Save()

# 3. Start Menu Programs\Hime
$scStartMenu = $WshShell.CreateShortcut((Join-Path $startMenuDir "Hidupkan Hime.lnk"))
$scStartMenu.TargetPath = $wscriptExe
$scStartMenu.Arguments = "`"$launcherVbs`" start"
$scStartMenu.WorkingDirectory = $repoRoot
$scStartMenu.WindowStyle = 7
$scStartMenu.Hotkey = ""
if (Test-Path $iconHime) { $scStartMenu.IconLocation = $iconHime }
$scStartMenu.Description = "Hidupkan Hime Native Ecosystem (Ctrl+Shift+H / Ctrl+Alt+Space)"
$scStartMenu.Save()

$scStopMenu = $WshShell.CreateShortcut((Join-Path $startMenuDir "Matikan Hime.lnk"))
$scStopMenu.TargetPath = $wscriptExe
$scStopMenu.Arguments = "`"$launcherVbs`" stop"
$scStopMenu.WorkingDirectory = $repoRoot
$scStopMenu.WindowStyle = 7
$scStopMenu.Hotkey = ""
$scStopMenu.IconLocation = "$env:SystemRoot\System32\shell32.dll,27"
$scStopMenu.Description = "Matikan Hime Native Ecosystem (Ctrl+Shift+Q)"
$scStopMenu.Save()

# 4. Startup Folder: Autostart Hime (remove stale legacy HimeHotkey.lnk if present)
$staleHotkeyLnk = Join-Path $startupDir "HimeHotkey.lnk"
if (Test-Path $staleHotkeyLnk) {
    Remove-Item $staleHotkeyLnk -Force -ErrorAction SilentlyContinue
}
$scStartup = $WshShell.CreateShortcut((Join-Path $startupDir "Hime.lnk"))
$scStartup.TargetPath = $wscriptExe
$scStartup.Arguments = "`"$launcherVbs`" start-autostart"
$scStartup.WorkingDirectory = $repoRoot
if (Test-Path $iconHime) { $scStartup.IconLocation = $iconHime }
$scStartup.Description = "Hime Native Ecosystem Persistent Autostart"
$scStartup.Save()

# 5. Registry HKCU Run (hime-island-native.exe absorbs hime_hotkey.exe)
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "Hime" -Value "wscript.exe `"$launcherVbs`" start-autostart"
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "HimeHotkey" -ErrorAction SilentlyContinue

Write-Host "Hime Native Desktop shortcuts, Start Menu & Autostart configured successfully!" -ForegroundColor Green
