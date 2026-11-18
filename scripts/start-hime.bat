@echo off
title Hime Personal AI Ecosystem Launcher
cls
echo ===================================================================
echo                     HIME PERSONAL AI ECOSYSTEM
echo ===================================================================
echo [1/3] Memeriksa ^& Mengaktifkan 9Router Gateway (Port 20128)...
powershell -NoProfile -Command "if (-not (Get-NetTCPConnection -LocalPort 20128 -ErrorAction SilentlyContinue)) { Start-Process -FilePath 'cmd.exe' -ArgumentList '/c 9router -H 127.0.0.1 -p 20128 --tray --skip-update' -WindowStyle Hidden }"

echo [2/3] Memeriksa Frontend Shell Runtime (Port 1420)...
powershell -NoProfile -Command "if (-not (Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue)) { $bun = 'C:\Users\Satoshi\.bun\bin\bun.exe'; if (Test-Path $bun) { Start-Process -FilePath $bun -ArgumentList 'run dev' -WorkingDirectory 'X:\folder_ai\Hime-Bloom' -WindowStyle Hidden } else { Start-Process -FilePath 'cmd.exe' -ArgumentList '/c npm run dev' -WorkingDirectory 'X:\folder_ai\Hime-Bloom' -WindowStyle Hidden }; Start-Sleep -Seconds 2 }"

echo [3/3] Menjalankan Hime Desktop Shell (Dynamic Island ^& Dock)...
powershell -NoProfile -Command "if (-not (Get-Process -Name 'bloom' -ErrorAction SilentlyContinue)) { Start-Process -FilePath 'X:\folder_ai\Hime-Bloom\src-tauri\target\release\bloom.exe' -WorkingDirectory 'X:\folder_ai\Hime-Bloom' }"

echo ===================================================================
echo [BERHASIL] Seluruh sistem Hime telah aktif untuk penggunaan sehari-hari!
echo - Dynamic Island ^& Dock: Siap di layar desktop
echo - 9Router Web UI: http://127.0.0.1:20128/dashboard
echo - Hermes AI Agent: Terhubung otomatis
echo ===================================================================
timeout /t 3 >nul
exit
