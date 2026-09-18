@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0ltools.ps1" %*
set "WTOOLS_EXIT=%ERRORLEVEL%"
if not "%WTOOLS_EXIT%"=="0" (
    echo.
    echo WTools no pudo ejecutarse. Codigo: %WTOOLS_EXIT%
    if /i not "%LTOOLS_NO_PAUSE%"=="1" pause
)
exit /b %WTOOLS_EXIT%
