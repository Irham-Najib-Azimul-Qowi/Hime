# hime-supervisor.ps1 - Background Supervisor for Hime Ecosystem (Native default)
param([switch]$LegacyWebView)

$ErrorActionPreference = "SilentlyContinue"

$stateFile = "$env:APPDATA\ai.hime.desktop\hime.active"
$himeRoot = "X:\folder_ai\Hime"
$logFile = "$env:APPDATA\ai.hime.desktop\supervisor.log"

# Named mutex to guarantee single supervisor instance
$mutexName = "Local\HimeSupervisorMutex"
$mutex = $null
try {
    $mutex = New-Object System.Threading.Mutex($false, $mutexName)
    $hasHandle = $mutex.WaitOne(500, $false)
    if (-not $hasHandle) {
        exit 0
    }
} catch {
    exit 0
}

# Win32 Named Event HimeQuit listener
$quitEvent = $null
try {
    $quitEvent = New-Object System.Threading.EventWaitHandle($false, [System.Threading.EventResetMode]::ManualReset, "HimeQuit")
} catch {
    try {
        $quitEvent = [System.Threading.EventWaitHandle]::OpenExisting("HimeQuit")
    } catch {
        $quitEvent = $null
    }
}

function Log-Msg($msg) {
    $line = "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')] $msg"
    Add-Content -Path $logFile -Value $line -ErrorAction SilentlyContinue
}

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

function Spawn-Detached($exePath, $args = "", $workDir = "", $showMode = 0) {
    $ws = if ($showMode -eq 0) { "Hidden" } else { "Normal" }
    if ($args) {
        Start-Process -FilePath $exePath -ArgumentList $args -WorkingDirectory $workDir -WindowStyle $ws
    } else {
        Start-Process -FilePath $exePath -WorkingDirectory $workDir -WindowStyle $ws
    }
}

Log-Msg "Hime Supervisor started."

try {
    while ($true) {
        # If user stopped Hime, exit supervisor cleanly
        if (-not (Test-Path $stateFile)) {
            Log-Msg "hime.active removed. Supervisor shutting down cleanly."
            break
        }

        # 1. Ensure 9Router Gateway is listening on 20128
        $listening = Get-NetTCPConnection -LocalPort 20128 -State Listen -ErrorAction SilentlyContinue
        if (-not $listening) {
            $routerCmd = "$env:APPDATA\npm\9router.cmd"
            if (Test-Path $routerCmd) {
                Log-Msg "9Router not running. Starting..."
                Spawn-Detached "cmd.exe" "/c `"$routerCmd`" -H 127.0.0.1 -p 20128 --skip-update" "$env:APPDATA\npm" 0
            }
        }

        # 2. Ensure Hime Core is running
        $core = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
        if (-not $core) {
            $coreExe = Find-Bin "core" "hime-core.exe"
            if ($coreExe) {
                Log-Msg "Hime Core not running. Starting from $coreExe..."
                Spawn-Detached $coreExe "" $himeRoot 0
            }
        }

        if ($LegacyWebView) {
            # 3-Legacy. Ensure WebView2 Island Shell is running
            $shell = Get-Process -Name "hime-island" -ErrorAction SilentlyContinue
            if (-not $shell) {
                $shellExe = Find-Bin "island\src-tauri" "hime-island.exe"
                if ($shellExe) {
                    Log-Msg "Island Shell not running. Starting from $shellExe..."
                    Spawn-Detached $shellExe "" "$himeRoot\island" 1
                }
            }

            # 4-Legacy. Ensure Hotkey Daemon is running
            $hotkey = Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue
            if (-not $hotkey) {
                $hotkeyExe = Find-Bin "island\src-tauri" "hime_hotkey.exe"
                if ($hotkeyExe) {
                    Log-Msg "hime_hotkey not running. Starting from $hotkeyExe..."
                    Spawn-Detached $hotkeyExe "" $himeRoot 0
                }
            }
        } else {
            # 3-Native (Default). Ensure Pure-Rust Direct2D Native Shell is running
            $nativeShell = Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue
            if (-not $nativeShell) {
                $nativeExe = Find-Bin "island\native" "hime-island-native.exe"
                if ($nativeExe) {
                    Log-Msg "Native Island Shell not running. Starting from $nativeExe..."
                    Spawn-Detached $nativeExe "" $himeRoot 0
                }
            }
        }

        # Listen for HimeQuit named event (or wait up to 10 seconds before next check)
        $signaled = $false
        if ($quitEvent) {
            $signaled = $quitEvent.WaitOne(10000)
        } else {
            Start-Sleep -Seconds 10
        }

        if ($signaled) {
            Log-Msg "HimeQuit event signaled. Supervisor shutting down cleanly."
            break
        }
    }
} finally {
    if ($quitEvent) {
        $quitEvent.Dispose()
    }
    if ($mutex) {
        $mutex.ReleaseMutex()
        $mutex.Dispose()
    }
}
