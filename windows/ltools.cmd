@echo off
setlocal
"%~dp0ltools.exe" %*
exit /b %ERRORLEVEL%
