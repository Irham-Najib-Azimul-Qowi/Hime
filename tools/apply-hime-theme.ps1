<#
.SYNOPSIS
    Applies Hime Crimson Cyber Theme to Windows System (Start Menu, Notifications, Calendar).
.DESCRIPTION
    Configures Windows registry for system dark mode, color prevalence on surfaces,
    and Hime Crimson Red (#E50914) accent color across DWM and Explorer.
    Broadcasts WM_SETTINGCHANGE and DWM colorization messages so changes apply immediately.
#>

[CmdletBinding()]
param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

Write-Host "=== Applying Hime Windows Theme ===" -ForegroundColor Cyan

# 1. Personalize Registry Keys (Dark mode: AppsUseLightTheme=0, SystemUsesLightTheme=0, ColorPrevalence=1)
$personalizePath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"
if (-not (Test-Path $personalizePath)) {
    New-Item -Path $personalizePath -Force | Out-Null
}

Write-Host "[1/3] Setting Windows Dark Mode and Color Prevalence..." -ForegroundColor Yellow
Set-ItemProperty -Path $personalizePath -Name "AppsUseLightTheme" -Value 0 -Type DWord
Set-ItemProperty -Path $personalizePath -Name "SystemUsesLightTheme" -Value 0 -Type DWord
Set-ItemProperty -Path $personalizePath -Name "ColorPrevalence" -Value 1 -Type DWord

# 2. DWM Registry Keys (Accent color: Hime Crimson Red #E50914)
# In Windows DWM:
# AccentColor is ABGR: 0xFF1409E5 -> unsigned: 4279503333, signed: -15463963
# ColorizationColor is ARGB: 0xFFE50914 -> unsigned: 4293200148, signed: -1767148
$dwmPath = "HKCU:\Software\Microsoft\Windows\DWM"
if (-not (Test-Path $dwmPath)) {
    New-Item -Path $dwmPath -Force | Out-Null
}

Write-Host "[2/3] Setting Hime Crimson Accent Color (#E50914) in DWM & Explorer..." -ForegroundColor Yellow
$accentColorABGR = [int]-15463963
$colorizationColorARGB = [int]-1767148

Set-ItemProperty -Path $dwmPath -Name "AccentColor" -Value $accentColorABGR -Type DWord
Set-ItemProperty -Path $dwmPath -Name "ColorizationColor" -Value $colorizationColorARGB -Type DWord
Set-ItemProperty -Path $dwmPath -Name "ColorizationAfterglow" -Value $colorizationColorARGB -Type DWord
Set-ItemProperty -Path $dwmPath -Name "ColorPrevalence" -Value 1 -Type DWord

# Explorer Accent for Start Menu, Taskbar, Action Center & Calendar flyout
$explorerAccentPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent"
if (-not (Test-Path $explorerAccentPath)) {
    New-Item -Path $explorerAccentPath -Force | Out-Null
}
Set-ItemProperty -Path $explorerAccentPath -Name "AccentColorMenu" -Value $accentColorABGR -Type DWord -ErrorAction SilentlyContinue

# AccentPalette: 8-color gradient palette centered on Hime Crimson
$accentPalette = [byte[]](
    0xFF, 0x7A, 0x6B, 0x00, # Light ruby highlight
    0xFF, 0x5C, 0x47, 0x00,
    0xFF, 0x3D, 0x24, 0x00,
    0xE5, 0x09, 0x14, 0x00, # Base Hime Crimson (#E50914)
    0xC2, 0x07, 0x10, 0x00,
    0x9F, 0x06, 0x0D, 0x00,
    0x7C, 0x04, 0x0A, 0x00,
    0x4D, 0x02, 0x06, 0x00  # Deep noir cyber shadow
)
Set-ItemProperty -Path $explorerAccentPath -Name "AccentPalette" -Value $accentPalette -Type Binary -ErrorAction SilentlyContinue

# 3. Broadcast WM_SETTINGCHANGE & DWM Setting Changes
Write-Host "[3/3] Broadcasting DWM & System Setting Changes..." -ForegroundColor Yellow

$signature = @"
using System;
using System.Runtime.InteropServices;

public class HimeThemeBroadcaster {
    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd,
        uint Msg,
        IntPtr wParam,
        string lParam,
        uint fuFlags,
        uint uTimeout,
        out IntPtr lpdwResult
    );
}
"@

try {
    Add-Type -TypeDefinition $signature -ErrorAction SilentlyContinue
} catch {}

$HWND_BROADCAST = [IntPtr]0xffff
$WM_SETTINGCHANGE = 0x001A
$WM_DWMCOLORIZATIONCOLORCHANGED = 0x0320
$SMTO_ABORTIFHUNG = 0x0002
$result = [IntPtr]::Zero

try {
    [HimeThemeBroadcaster]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [IntPtr]::Zero, "ImmersiveColorSet", $SMTO_ABORTIFHUNG, 1000, [ref]$result) | Out-Null
    [HimeThemeBroadcaster]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [IntPtr]::Zero, "Environment", $SMTO_ABORTIFHUNG, 1000, [ref]$result) | Out-Null
    [HimeThemeBroadcaster]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [IntPtr]::Zero, "WindowsThemeElement", $SMTO_ABORTIFHUNG, 1000, [ref]$result) | Out-Null
    [HimeThemeBroadcaster]::SendMessageTimeout($HWND_BROADCAST, $WM_DWMCOLORIZATIONCOLORCHANGED, [IntPtr]$colorizationColorARGB, "1", $SMTO_ABORTIFHUNG, 1000, [ref]$result) | Out-Null
    Write-Host "  WM_SETTINGCHANGE & DWM colorization broadcasts sent successfully." -ForegroundColor Green
} catch {
    Write-Warning "  Broadcast message encountered notice: $_"
}

# Verify written properties
Write-Host "`n--- Verification of Configured Registry Settings ---" -ForegroundColor Cyan
$pers = Get-ItemProperty -Path $personalizePath
Write-Host ("  AppsUseLightTheme:    " + $pers.AppsUseLightTheme)
Write-Host ("  SystemUsesLightTheme:  " + $pers.SystemUsesLightTheme)
Write-Host ("  ColorPrevalence:       " + $pers.ColorPrevalence)

$dwm = Get-ItemProperty -Path $dwmPath
Write-Host ("  DWM AccentColor:       0x" + ('{0:X8}' -f [uint32]$dwm.AccentColor))
Write-Host ("  DWM ColorizationColor: 0x" + ('{0:X8}' -f [uint32]$dwm.ColorizationColor))
Write-Host ("  DWM ColorPrevalence:   " + $dwm.ColorPrevalence)

Write-Host "`n=== Hime Windows Theming Applied Successfully! ===" -ForegroundColor Green
