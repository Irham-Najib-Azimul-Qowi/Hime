@echo off
title Hime Build Tools Installer
echo ======================================================================
echo  Hime Engineering Foundation: Visual Studio 2022 Build Tools
echo ======================================================================
echo.
echo Installing C++ Workload (MSVC v143) and Windows 11 SDK...
echo A Windows UAC (User Account Control) prompt will appear.
echo Please click "Yes" / "Ya" to allow installation.
echo.

set "INSTALLER=C:\Users\Satoshi\AppData\Local\Temp\WinGet\Microsoft.VisualStudio.2022.BuildTools.17.14.44\vs_BuildTools.exe"

if not exist "%INSTALLER%" (
    echo [ERROR] Installer not found at: %INSTALLER%
    pause
    exit /b 1
)

echo Executing installer: %INSTALLER%
"%INSTALLER%" --passive --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended

set "EXITCODE=%ERRORLEVEL%"
echo.
echo Installer finished with exit code: %EXITCODE%
if "%EXITCODE%"=="0" (
    echo [SUCCESS] Visual Studio 2022 Build Tools successfully installed!
) else if "%EXITCODE%"=="3010" (
    echo [SUCCESS] Installed successfully (restart may be required later).
) else (
    echo [WARNING] Exit code was %EXITCODE%. Please check the installer log.
)
echo.
echo You can now close this window.
pause

