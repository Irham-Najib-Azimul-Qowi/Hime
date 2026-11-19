@echo off
title Hime Desktop Shell Launcher
echo ================================================================================
echo   HIME DESKTOP SHELL (DYNAMIC ISLAND ^& DOCK)
echo ================================================================================
echo Starting Hime Desktop Shell...
powershell -ExecutionPolicy Bypass -File "%~dp0scripts\dev-hime.ps1" -Seconds 0
