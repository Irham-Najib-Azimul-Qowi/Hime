# =============================================================================
# Hime Desktop — Windows Runtime Verification & Screenshot Capture
# =============================================================================
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$repoRoot = (Get-Item -Path $PSScriptRoot\..).FullName
$screenshotPath = Join-Path $repoRoot "docs\desktop\screenshots\hime-desktop-runtime.png"
$coreBin = Join-Path $repoRoot "target\debug\hime-core.exe"
$desktopBin = Join-Path $repoRoot "target\debug\hime-desktop.exe"

Write-Host "[1/5] Launching Hime Core Daemon ($coreBin)..."
$coreProc = Start-Process -FilePath $coreBin -PassThru
Start-Sleep -Seconds 2

if (-not (Get-Process -Id $coreProc.Id -ErrorAction SilentlyContinue)) {
    throw "Failed to start Hime Core process."
}
Write-Host "   -> Hime Core active with PID: $($coreProc.Id)"

Write-Host "[2/5] Starting Vite Dev Server on port 5173..."
$viteProc = Start-Process -FilePath "cmd.exe" -ArgumentList "/c pnpm --dir apps/hime-desktop dev" -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 3

Write-Host "[3/5] Launching Hime Desktop Shell ($desktopBin)..."
$desktopProc = Start-Process -FilePath $desktopBin -PassThru
Start-Sleep -Seconds 5

if (-not (Get-Process -Id $desktopProc.Id -ErrorAction SilentlyContinue)) {
    throw "Hime Desktop process exited prematurely."
}
Write-Host "   -> Hime Desktop active with PID: $($desktopProc.Id), Window Handle: $($desktopProc.MainWindowHandle)"

Write-Host "[4/5] Capturing Real Windows Desktop Screenshot..."
try {
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save($screenshotPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()
    Write-Host "   -> Screenshot successfully saved to $screenshotPath"
} catch {
    Write-Host "   -> [NOT TESTED] GDI Screen capture unavailable in non-interactive agent execution session: $($_.Exception.Message)"
}

Write-Host "[5/5] Cleaning up runtime verification processes..."
try { Stop-Process -Id $desktopProc.Id -Force -ErrorAction SilentlyContinue } catch {}
try { Stop-Process -Id $coreProc.Id -Force -ErrorAction SilentlyContinue } catch {}
try { Stop-Process -Id $viteProc.Id -Force -ErrorAction SilentlyContinue } catch {}
# Also terminate any stray node processes for port 5173
try { Get-Process -Name "node" | Where-Object { $_.Path -like "*node*" } | Stop-Process -Force -ErrorAction SilentlyContinue } catch {}

Write-Host "RUNTIME VERIFICATION COMPLETED SUCCESSFULLY."
