@echo off
setlocal
echo Selah — Windows installer helper
echo ================================
echo.

REM Locate the installer in the same folder as this script
set "SCRIPT_DIR=%~dp0"
set "INSTALLER="

for %%F in ("%SCRIPT_DIR%Selah-Setup-*.exe") do (
    set "INSTALLER=%%F"
    goto :found
)

echo Could not find Selah-Setup-*.exe in the same folder as this script.
echo Please make sure both files are in the same folder.
pause
exit /b 1

:found
echo Found installer: %INSTALLER%
echo.

echo Removing the "downloaded from internet" flag...
powershell -NoProfile -Command "Unblock-File -Path '%INSTALLER%'"

echo Launching installer...
"%INSTALLER%"

echo.
echo If installation succeeded, Selah will be in your Start menu.
pause