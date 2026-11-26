# start-hime.ps1 - Start Hime Ecosystem with Self-Healing Supervisor (Native default)
param([switch]$Autostart, [switch]$Native, [switch]$LegacyWebView)

$ErrorActionPreference = "SilentlyContinue"
$stateDir = "$env:APPDATA\ai.hime.desktop"
$stateFile = "$stateDir\hime.active"
$settingsFile = "$stateDir\settings.json"

if (-not (Test-Path $stateDir)) {
    New-Item -ItemType Directory -Path $stateDir -Force | Out-Null
}

# 1. Update settings to ensure Dock is enabled and fixed (visible on launch)
if (Test-Path $settingsFile) {
    try {
        $json = Get-Content -Path $settingsFile -Raw | ConvertFrom-Json
        $json."bloom-dock-enabled" = "true"
        $json."bloom-dock-mode" = "fixed"
        $json."bloom-notch-mode" = "fixed"
        $json | ConvertTo-Json -Compress | Set-Content -Path $settingsFile -Force
    } catch {}
} else {
    @{
        "bloom-dock-enabled" = "true"
        "bloom-dock-mode"    = "fixed"
        "bloom-notch-mode"   = "fixed"
        "bloom-scale"        = "1"
    } | ConvertTo-Json -Compress | Set-Content -Path $settingsFile -Force
}

# 2. Check Autostart condition
if ($Autostart) {
    # If launched on Windows boot/logon, only resume if Hime was active before shutdown
    if (-not (Test-Path $stateFile)) {
        Write-Host "Hime was not active before reboot. Skipping autostart."
        exit 0
    }
    # Preserve and refresh the active state flag
    Set-Content -Path $stateFile -Value ("active=" + (Get-Date).ToString("o")) -Force
} else {
    # Manual start: set active state flag
    Set-Content -Path $stateFile -Value ("active=" + (Get-Date).ToString("o")) -Force
}

# 3. Reset Win32 Named Event HimeQuit so processes start fresh
try {
    $quitEvent = [System.Threading.EventWaitHandle]::OpenExisting("HimeQuit")
    $quitEvent.Reset() | Out-Null
    $quitEvent.Dispose()
} catch {
    try {
        $quitEvent = New-Object System.Threading.EventWaitHandle($false, [System.Threading.EventResetMode]::ManualReset, "HimeQuit")
        $quitEvent.Reset() | Out-Null
        $quitEvent.Dispose()
    } catch {}
}

$himeRoot = "X:\folder_ai\Hime"

$sh = $null
try {
    $sh = New-Object -ComObject Shell.Application
} catch {}

function Spawn-Detached($exePath, $args = "", $workDir = "", $showMode = 0) {
    $ws = if ($showMode -eq 0) { "Hidden" } else { "Normal" }
    if ($args) {
        Start-Process -FilePath $exePath -ArgumentList $args -WorkingDirectory $workDir -WindowStyle $ws
    } else {
        Start-Process -FilePath $exePath -WorkingDirectory $workDir -WindowStyle $ws
    }
}

# Helper to find release or debug executable
function Find-Bin($subpath, $name) {
    $rel = Join-Path $himeRoot "$subpath\target\release\$name"
    $dbg = Join-Path $himeRoot "$subpath\target\debug\$name"
    if ((Test-Path $rel) -and (Test-Path $dbg)) {
        if ((Get-Item $rel).LastWriteTime -ge (Get-Item $dbg).LastWriteTime) { return $rel } else { return $dbg }
    }
    if (Test-Path $rel) { return $rel }
    if (Test-Path $dbg) { return $dbg }
    return $null
}

# 3. Ensure 9Router Gateway is listening on 20128
$listening = Get-NetTCPConnection -LocalPort 20128 -State Listen -ErrorAction SilentlyContinue
if (-not $listening) {
    $routerCmd = "$env:APPDATA\npm\9router.cmd"
    if (Test-Path $routerCmd) {
        Spawn-Detached "cmd.exe" "/c `"$routerCmd`" -H 127.0.0.1 -p 20128 --skip-update" "$env:APPDATA\npm" 0
    }
}

# 4. Start Hime Core if not running
$core = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
if (-not $core) {
    $coreExe = Find-Bin "core" "hime-core.exe"
    if ($coreExe) {
        Spawn-Detached $coreExe "" $himeRoot 0
    }
}

$useLegacy = [bool]$LegacyWebView
$supervisorScript = "$himeRoot\scripts\hime-supervisor.ps1"
$supervisorArgs = if ($useLegacy) { "-LegacyWebView" } else { "" }

if (-not $useLegacy) {
    # 5-Native (Default). Start Pure-Rust Direct2D Shell (built-in global hotkeys, no WebView2, no hime_hotkey.exe)
    $nativeShell = Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue
    if (-not $nativeShell) {
        $nativeExe = Find-Bin "island\native" "hime-island-native.exe"
        if ($nativeExe) {
            Spawn-Detached $nativeExe "" $himeRoot 0
        }
    }

    # Start Supervisor Watchdog in background (Native mode)
    if (Test-Path $supervisorScript) {
        $launcherVbs = "$himeRoot\tools\hime_launcher.vbs"
        $wscriptExe = "$env:SystemRoot\System32\wscript.exe"
        if ((Test-Path $launcherVbs) -and (Test-Path $wscriptExe)) {
            Spawn-Detached $wscriptExe "`"$launcherVbs`" `"$supervisorScript`"" $himeRoot 0
        } else {
            Spawn-Detached "powershell.exe" "-NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$supervisorScript`"" $himeRoot 0
        }
    }

    Write-Host "Hime Native Ecosystem Started (Pure-Rust Direct2D Shell + Core)." -ForegroundColor Green
    exit 0
}

# 5-Legacy. Start Island Shell (hime-island.exe) if not running
$shell = Get-Process -Name "hime-island" -ErrorAction SilentlyContinue
if (-not $shell) {
    $shellExe = Find-Bin "island\src-tauri" "hime-island.exe"
    if ($shellExe) {
        Spawn-Detached $shellExe "" "$himeRoot\island" 1
    }
} else {
    # If already running, signal HimeShowIsland to wake / toggle the notch UI
    try {
        $evt = [System.Threading.EventWaitHandle]::OpenExisting("HimeShowIsland")
        $evt.Set() | Out-Null
        $evt.Dispose()
    } catch {}
}

# 6. Start Hotkey Daemon (guarantee single up-to-date instance)
$hotkeyExe = Find-Bin "island\src-tauri" "hime_hotkey.exe"
$hotkeys = @(Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue)
$restartHotkey = $false
if ($hotkeys.Count -gt 1) {
    $restartHotkey = $true
} elseif ($hotkeys.Count -eq 1 -and $hotkeyExe) {
    try {
        $runningPath = $hotkeys[0].Path
        if ($runningPath -and (($runningPath -ne $hotkeyExe) -or ((Get-Item $hotkeyExe).LastWriteTime -gt $hotkeys[0].StartTime))) {
            $restartHotkey = $true
        }
    } catch {}
}
if ($restartHotkey) {
    $hotkeys | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 150
    $hotkeys = @()
}
if ($hotkeys.Count -eq 0 -and $hotkeyExe) {
    Spawn-Detached $hotkeyExe "" $himeRoot 0
}

# 7. Start Supervisor Watchdog in background (Legacy WebView2 mode)
if (Test-Path $supervisorScript) {
    $launcherVbs = "$himeRoot\tools\hime_launcher.vbs"
    $wscriptExe = "$env:SystemRoot\System32\wscript.exe"
    if ((Test-Path $launcherVbs) -and (Test-Path $wscriptExe)) {
        Spawn-Detached $wscriptExe "`"$launcherVbs`" `"$supervisorScript`" $supervisorArgs" $himeRoot 0
    } else {
        Spawn-Detached "powershell.exe" "-NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$supervisorScript`" $supervisorArgs" $himeRoot 0
    }
}

Write-Host "Hime Ecosystem Started (Active & Persistent)." -ForegroundColor Green
