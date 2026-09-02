@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0ltools.ps1" %*
set "LTOOLS_EXIT=%ERRORLEVEL%"
if not "%LTOOLS_EXIT%"=="0" (
    echo.
    echo LTools no pudo ejecutarse. Codigo: %LTOOLS_EXIT%
    if /i not "%LTOOLS_NO_PAUSE%"=="1" pause
)
exit /b %LTOOLS_EXIT%
