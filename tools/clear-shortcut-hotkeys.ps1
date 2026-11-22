$WshShell = New-Object -ComObject WScript.Shell
$links = Get-ChildItem -Path "$env:USERPROFILE\Desktop", "$env:APPDATA\Microsoft\Windows\Start Menu\Programs" -Recurse -Filter "*.lnk" -ErrorAction SilentlyContinue
foreach ($f in $links) {
    try {
        $sc = $WshShell.CreateShortcut($f.FullName)
        if ($sc.Hotkey) {
            Write-Host ("Clearing Hotkey '" + $sc.Hotkey + "' from " + $f.FullName)
            $sc.Hotkey = ""
            $sc.WindowStyle = 7
            $sc.Save()
        }
    } catch {}
}
Write-Host "Shortcut hotkeys cleared."
