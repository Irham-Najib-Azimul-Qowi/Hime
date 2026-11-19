@echo off
title Stop Hime and Restore Taskbar
powershell -ExecutionPolicy Bypass -File "%~dp0scripts\stop-hime.ps1"
pause
