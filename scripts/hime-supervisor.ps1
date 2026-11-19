# hime-supervisor.ps1 - Background Supervisor for Hime Ecosystem
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

function Log-Msg($msg) {
    $line = "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')] $msg"
    Add-Content -Path $logFile -Value $line -ErrorAction SilentlyContinue
}

function Find-Bin($subpath, $name) {
    $rel = Join-Path $himeRoot "$subpath\target\release\$name"
    if (Test-Path $rel) { return $rel }
    $dbg = Join-Path $himeRoot "$subpath\target\debug\$name"
    if (Test-Path $dbg) { return $dbg }
    return $null
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
                Start-Process -FilePath $routerCmd -ArgumentList "-H 127.0.0.1 -p 20128 --skip-update" -WindowStyle Hidden
            }
        }

        # 2. Ensure Hime Core is running
        $core = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
        if (-not $core) {
            $coreExe = Find-Bin "core" "hime-core.exe"
            if ($coreExe) {
                Log-Msg "Hime Core not running. Starting from $coreExe..."
                Start-Process -FilePath $coreExe -WorkingDirectory $himeRoot -WindowStyle Hidden
            }
        }

        # 3. Ensure Island Shell is running
        $shell = Get-Process -Name "hime-island" -ErrorAction SilentlyContinue
        if (-not $shell) {
            $shellExe = Find-Bin "island\src-tauri" "hime-island.exe"
            if ($shellExe) {
                Log-Msg "Island Shell not running. Starting from $shellExe..."
                Start-Process -FilePath $shellExe -WorkingDirectory "$himeRoot\island"
            }
        }

        # 4. Ensure Hotkey Daemon is running
        $hotkey = Get-Process -Name "hime_hotkey" -ErrorAction SilentlyContinue
        if (-not $hotkey) {
            $hotkeyExe = Find-Bin "island\src-tauri" "hime_hotkey.exe"
            if ($hotkeyExe) {
                Log-Msg "hime_hotkey not running. Starting from $hotkeyExe..."
                Start-Process -FilePath $hotkeyExe -WorkingDirectory $himeRoot -WindowStyle Hidden
            }
        }

        Start-Sleep -Seconds 10
    }
} finally {
    if ($mutex) {
        $mutex.ReleaseMutex()
        $mutex.Dispose()
    }
}
