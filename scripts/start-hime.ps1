# start-hime.ps1 - Start Hime Pure-Rust Native Ecosystem with Self-Healing Supervisor
param([switch]$Autostart, [switch]$Native)

$ErrorActionPreference = "SilentlyContinue"
$repoRoot = Split-Path -Parent $PSScriptRoot
$stateDir = "$env:APPDATA\ai.hime.desktop"
$stateFile = "$stateDir\hime.active"
$settingsFile = "$stateDir\settings.json"

if (-not (Test-Path $stateDir)) {
    New-Item -ItemType Directory -Path $stateDir -Force | Out-Null
}

# 1. Ensure default Dock/Notch settings exist without overwriting user preferences
if (Test-Path $settingsFile) {
    try {
        $json = Get-Content -Path $settingsFile -Raw | ConvertFrom-Json
        $changed = $false
        if (-not $json.PSObject.Properties["bloom-dock-enabled"] -or [string]::IsNullOrWhiteSpace($json."bloom-dock-enabled")) {
            $json | Add-Member -NotePropertyName "bloom-dock-enabled" -NotePropertyValue "true" -Force
            $changed = $true
        }
        if (-not $json.PSObject.Properties["bloom-dock-mode"] -or [string]::IsNullOrWhiteSpace($json."bloom-dock-mode")) {
            $json | Add-Member -NotePropertyName "bloom-dock-mode" -NotePropertyValue "fixed" -Force
            $changed = $true
        }
        if (-not $json.PSObject.Properties["bloom-notch-mode"] -or [string]::IsNullOrWhiteSpace($json."bloom-notch-mode")) {
            $json | Add-Member -NotePropertyName "bloom-notch-mode" -NotePropertyValue "fixed" -Force
            $changed = $true
        }
        if ($changed) {
            $json | ConvertTo-Json -Compress | Set-Content -Path $settingsFile -Force
        }
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
    Set-Content -Path $stateFile -Value ("active=" + (Get-Date).ToString("o")) -Force
} else {
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

function Spawn-Detached($exePath, $argList = "", $workDir = "", $showMode = 0) {
    $ws = if ($showMode -eq 0) { "Hidden" } else { "Normal" }
    if ($argList) {
        Start-Process -FilePath $exePath -ArgumentList $argList -WorkingDirectory $workDir -WindowStyle $ws
    } else {
        Start-Process -FilePath $exePath -WorkingDirectory $workDir -WindowStyle $ws
    }
}

function Find-Bin($subpath, $name) {
    $rel = Join-Path $repoRoot "$subpath\target\release\$name"
    $dbg = Join-Path $repoRoot "$subpath\target\debug\$name"
    if ((Test-Path $rel) -and (Test-Path $dbg)) {
        if ((Get-Item $rel).LastWriteTime -ge (Get-Item $dbg).LastWriteTime) { return $rel } else { return $dbg }
    }
    if (Test-Path $rel) { return $rel }
    if (Test-Path $dbg) { return $dbg }
    return $null
}

# 4. Ensure optional 9Router Gateway is listening on 20128 if installed
$listening = Get-NetTCPConnection -LocalPort 20128 -State Listen -ErrorAction SilentlyContinue
if (-not $listening) {
    $routerCmd = "$env:APPDATA\npm\9router.cmd"
    if (Test-Path $routerCmd) {
        Spawn-Detached "cmd.exe" "/c `"$routerCmd`" -H 127.0.0.1 -p 20128 --skip-update" "$env:APPDATA\npm" 0
    }
}

# 5. Start Hime Core (hime-core.exe) if not running
$core = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
if (-not $core) {
    $coreExe = Find-Bin "core" "hime-core.exe"
    if ($coreExe) {
        Spawn-Detached $coreExe "" $repoRoot 0
    }
}

# 6. Start Pure-Rust Direct2D Native Shell (hime-island-native.exe; built-in global hotkeys)
$nativeShell = Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue
if (-not $nativeShell) {
    $nativeExe = Find-Bin "island\native" "hime-island-native.exe"
    if ($nativeExe) {
        Spawn-Detached $nativeExe "" $repoRoot 0
    }
}

# 7. Start Supervisor Watchdog in background
$supervisorScript = Join-Path $repoRoot "scripts\hime-supervisor.ps1"
if (Test-Path $supervisorScript) {
    $launcherVbs = Join-Path $repoRoot "tools\hime_launcher.vbs"
    $wscriptExe = "$env:SystemRoot\System32\wscript.exe"
    if ((Test-Path $launcherVbs) -and (Test-Path $wscriptExe)) {
        Spawn-Detached $wscriptExe "`"$launcherVbs`" `"$supervisorScript`"" $repoRoot 0
    } else {
        Spawn-Detached "powershell.exe" "-NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$supervisorScript`"" $repoRoot 0
    }
}

Write-Host "Hime Native Ecosystem Started (Pure-Rust Direct2D Shell + Core)." -ForegroundColor Green
exit 0
