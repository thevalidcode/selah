@echo off
echo Selah — Windows unblock helper
echo ===============================
echo.

set "TARGET=%LOCALAPPDATA%\Selah"

if not exist "%TARGET%" (
    echo Selah is not installed yet.
    echo Please run Selah-Setup.exe first, then run this again.
    pause
    exit /b 1
)

echo Unblocking files in %TARGET% ...
powershell -NoProfile -Command "Get-ChildItem -Path '%TARGET%' -Recurse -File | Unblock-File"

echo Done. You can now launch Selah normally.
pause