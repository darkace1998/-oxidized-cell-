@echo off
REM Download and install PS3 firmware for oxidized-cell
REM This script downloads the official PS3 System Software from Sony's servers

setlocal enabledelayedexpansion

set FIRMWARE_URL=http://dus01.ps3.update.playstation.net/update/ps3/image/us/2025_0305_c179ad173bbc08b55431d30947725a4b/PS3UPDAT.PUP
set FIRMWARE_DIR=%~dp0..\firmware
set FIRMWARE_FILE=%FIRMWARE_DIR%\PS3UPDAT.PUP

echo ========================================
echo   PS3 Firmware Downloader
echo   For oxidized-cell PS3 Emulator
echo ========================================
echo.

REM Create firmware directory if it doesn't exist
if not exist "%FIRMWARE_DIR%" mkdir "%FIRMWARE_DIR%"

REM Check if firmware already exists
if exist "%FIRMWARE_FILE%" (
    echo Firmware already exists at: %FIRMWARE_FILE%
    set /p REPLY="Do you want to re-download? (y/N) "
    if /i not "!REPLY!"=="y" (
        echo Keeping existing firmware.
        goto :end
    )
)

echo Downloading PS3 System Software from Sony's servers...
echo URL: %FIRMWARE_URL%
echo.

REM Try PowerShell first (Windows 10+)
where powershell >nul 2>nul
if %errorlevel% equ 0 (
    echo Using PowerShell to download...
    powershell -Command "& {$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '%FIRMWARE_URL%' -OutFile '%FIRMWARE_FILE%'}"
    goto :verify
)

REM Try curl (Windows 10 1803+)
where curl >nul 2>nul
if %errorlevel% equ 0 (
    echo Using curl to download...
    curl -L -o "%FIRMWARE_FILE%" "%FIRMWARE_URL%"
    goto :verify
)

echo Error: Could not find PowerShell or curl.
echo Please download the firmware manually from:
echo %FIRMWARE_URL%
echo And place it in: %FIRMWARE_DIR%
goto :error

:verify
if exist "%FIRMWARE_FILE%" (
    echo.
    echo ========================================
    echo   Firmware downloaded successfully!
    echo ========================================
    echo.
    echo Location: %FIRMWARE_FILE%
    echo.
    echo The emulator will automatically use this firmware to decrypt games.
    goto :end
) else (
    echo Error: Failed to download firmware.
    goto :error
)

:error
exit /b 1

:end
echo.
pause
