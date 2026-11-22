# start-hime.ps1 - Start Hime Ecosystem with Self-Healing Supervisor
param([switch]$Autostart)

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
        Start-Process -FilePath $routerCmd -ArgumentList "-H 127.0.0.1 -p 20128 --skip-update" -WindowStyle Hidden
    }
}

# 4. Start Hime Core if not running
$core = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
if (-not $core) {
    $coreExe = Find-Bin "core" "hime-core.exe"
    if ($coreExe) {
        Start-Process -FilePath $coreExe -WorkingDirectory $himeRoot -WindowStyle Hidden
    }
}

# 5. Start Island Shell (hime-island.exe) if not running
$shell = Get-Process -Name "hime-island" -ErrorAction SilentlyContinue
if (-not $shell) {
    $shellExe = Find-Bin "island\src-tauri" "hime-island.exe"
    if ($shellExe) {
        Start-Process -FilePath $shellExe -WorkingDirectory "$himeRoot\island"
    }
} else {
    # If already running, signal HimeShowIsland to wake / toggle the notch UI
    try {
        $evt = [System.Threading.EventWaitHandle]::OpenExisting("HimeShowIsland")
        $evt.Set() | Out-Null
        $evt.Dispose()
    } catch {}
}

# 6. Start Hotkey Daemon if not running
$hotkey = Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue
if (-not $hotkey) {
    $hotkeyExe = Find-Bin "island\src-tauri" "hime_hotkey.exe"
    if ($hotkeyExe) {
        Start-Process -FilePath $hotkeyExe -WorkingDirectory $himeRoot -WindowStyle Hidden
    }
}

# 7. Start Supervisor Watchdog in background
$supervisorScript = "$himeRoot\scripts\hime-supervisor.ps1"
if (Test-Path $supervisorScript) {
    Start-Process -FilePath "powershell.exe" -ArgumentList @("-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-File", $supervisorScript) -WindowStyle Hidden
}

Write-Host "Hime Ecosystem Started (Active & Persistent)." -ForegroundColor Green
