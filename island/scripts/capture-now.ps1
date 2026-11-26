$cs = 'X:\folder_ai\Hime-Bloom\scripts\DesktopCapture.cs'
Add-Type -Path $cs -ReferencedAssemblies 'System.Windows.Forms', 'System.Drawing'
[DesktopCapture]::Capture('C:\Users\Satoshi\.gemini\antigravity\brain\b260b532-d8ff-4265-ab67-4dfe74bbfe45\current_live_desktop.png')
