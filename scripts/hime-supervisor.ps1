# hime-supervisor.ps1 - Background Supervisor for Hime Pure-Rust Native Ecosystem
param([switch]$WithAi, [switch]$NoAi)

$ErrorActionPreference = "SilentlyContinue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$stateDir = "$env:APPDATA\ai.hime.desktop"
$stateFile = "$stateDir\hime.active"
$settingsFile = "$stateDir\settings.json"
$logFile = "$stateDir\supervisor.log"

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

function Test-HimeAiEnabled {
    if ($WithAi) { return $true }
    if ($NoAi) { return $false }
    if (-not (Test-Path $settingsFile)) { return $false }
    try {
        $json = Get-Content -Path $settingsFile -Raw | ConvertFrom-Json
        $raw = $json."hime-ai-enabled"
        if ($null -eq $raw) { return $false }
        if ($raw -is [bool]) { return $raw }
        $s = "$raw".Trim().ToLowerInvariant()
        return ($s -eq "true" -or $s -eq "1" -or $s -eq "yes")
    } catch {
        return $false
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

function Spawn-Detached($exePath, $argList = "", $workDir = "", $showMode = 0) {
    $ws = if ($showMode -eq 0) { "Hidden" } else { "Normal" }
    if ($argList) {
        Start-Process -FilePath $exePath -ArgumentList $argList -WorkingDirectory $workDir -WindowStyle $ws
    } else {
        Start-Process -FilePath $exePath -WorkingDirectory $workDir -WindowStyle $ws
    }
}

Log-Msg "Hime Native Supervisor started (root: $repoRoot, aiEnabled: $(Test-HimeAiEnabled))."

try {
    while ($true) {
        # If user stopped Hime, exit supervisor cleanly
        if (-not (Test-Path $stateFile)) {
            Log-Msg "hime.active removed. Supervisor shutting down cleanly."
            break
        }

        # 1. Manage optional AI runtime (9Router + hime-core) ONLY when AI is enabled
        if (Test-HimeAiEnabled) {
            $listening = Get-NetTCPConnection -LocalPort 20128 -State Listen -ErrorAction SilentlyContinue
            if (-not $listening) {
                $routerCmd = "$env:APPDATA\npm\9router.cmd"
                if (Test-Path $routerCmd) {
                    Log-Msg "9Router not running. Starting..."
                    Spawn-Detached "cmd.exe" "/c `"$routerCmd`" -H 127.0.0.1 -p 20128 --skip-update" "$env:APPDATA\npm" 0
                }
            }

            $core = Get-Process -Name "hime-core" -ErrorAction SilentlyContinue
            if (-not $core) {
                $coreExe = Find-Bin "core" "hime-core.exe"
                if ($coreExe) {
                    Log-Msg "Hime Core not running. Starting from $coreExe..."
                    Spawn-Detached $coreExe "" $repoRoot 0
                }
            }
        }

        # 2. Ensure Pure-Rust Direct2D Native Shell is running (includes embedded hotkeys)
        $nativeShell = Get-Process -Name "hime-island-native" -ErrorAction SilentlyContinue
        if (-not $nativeShell) {
            $nativeExe = Find-Bin "island\native" "hime-island-native.exe"
            if ($nativeExe) {
                Log-Msg "Native Island Shell not running. Starting from $nativeExe..."
                Spawn-Detached $nativeExe "" $repoRoot 0
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
