$cs = 'X:\folder_ai\Hime-Bloom\scripts\DesktopCapture.cs'
Add-Type -Path $cs -ReferencedAssemblies 'System.Windows.Forms', 'System.Drawing'
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point(683, 18)
Start-Sleep -Milliseconds 400
[DesktopCapture]::Capture('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\hover_live_notch.png')
