$shell = New-Object -ComObject "Shell.Application"
$shell.ShellExecute("X:\folder_ai\Hime-Bloom\src-tauri\target\release\bloom.exe", "", "X:\folder_ai\Hime-Bloom", "open", 1)
Start-Sleep -Seconds 3
Get-CimInstance Win32_Process -Filter "Name LIKE 'bloom%'" | Select-Object ProcessId, ParentProcessId, Name
