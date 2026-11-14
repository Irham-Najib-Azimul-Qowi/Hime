@echo off
title Hime Launcher
echo ================================================================================
echo   HIME DYNAMIC ISLAND LAUNCHER
echo ================================================================================
echo Starting Hime Core (Kernel Daemon)...
start "" "%~dp0target\release\hime-core.exe"
timeout /t 1 /nobreak >nul
echo Starting Hime Desktop (Dynamic Island)...
start "" "%~dp0target\release\hime-desktop.exe"
echo.
echo Hime is now running! Dynamic Island should appear on your screen.
timeout /t 3 /nobreak >nul

