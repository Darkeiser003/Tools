@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1" %*
set "LTOOLS_BUILD_EXIT=%ERRORLEVEL%"
if not "%LTOOLS_BUILD_EXIT%"=="0" (
    echo.
    echo La build de WinSlim-Tools fallo. Codigo: %LTOOLS_BUILD_EXIT%
    echo Revisa el mensaje anterior para conocer la causa.
    if /i not "%LTOOLS_NO_PAUSE%"=="1" pause
)
exit /b %LTOOLS_BUILD_EXIT%
