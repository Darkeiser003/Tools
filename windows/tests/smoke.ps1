# Smoke seguro del ejecutable Windows. No instala, borra ni modifica el sistema.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Binary)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Binary)) { throw "No existe el ejecutable: $Binary" }
$temp = Join-Path ([IO.Path]::GetTempPath()) ("ltools-windows-smoke-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $temp | Out-Null
$oldUserProfile = $env:USERPROFILE
$oldHome = $env:HOME
$oldAppData = $env:APPDATA
$oldLocalAppData = $env:LOCALAPPDATA
$env:USERPROFILE = $temp
$env:HOME = $temp
$env:APPDATA = Join-Path $temp 'AppData\Roaming'
$env:LOCALAPPDATA = Join-Path $temp 'AppData\Local'
try {
    function Run([string[]]$Args) {
        $output = & $Binary @Args 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) { throw "Falló ltools.exe $($Args -join ' '): $output" }
        if ([string]::IsNullOrWhiteSpace($output)) { throw "Salida vacía para ltools.exe $($Args -join ' ')" }
        Write-Host $output.Trim()
    }
    Run @('--version')
    Run @('--help')
    Run @('doctor')
    Run @('defaults')
    Run @('capabilities', '--format', 'json')
    Run @('audit', '--root', $temp, '--no-mounts', '--out', (Join-Path $temp 'report'))
    if (-not (Test-Path (Join-Path $temp 'report\summary.txt'))) { throw 'El resumen del informe Windows no se generó.' }
    if (-not (Test-Path (Join-Path $temp 'report\wine-prefixes.tsv'))) { throw 'El informe de prefijos Windows no se generó.' }
    Run @('system', 'status')
    Run @('system', 'services', '--filter', 'active', '--limit', '3')
    Run @('system', 'processes', '--sort', 'memory', '--limit', '3')
    Run @('system', 'journal', '--channel', 'System', '--limit', '3')
    $systemReport = Join-Path $temp 'system.json'
    Run @('system', 'export', '--format', 'json', '--out', $systemReport)
    if (-not (Test-Path $systemReport)) { throw 'El informe JSON de servicios Windows no se generó.' }
    if ((Get-Content -Raw $systemReport) -notmatch '^\s*\[') { throw 'El informe JSON Windows no tiene formato de lista.' }
    Write-Host 'Windows smoke completado correctamente.'
} finally {
    $env:USERPROFILE = $oldUserProfile
    $env:HOME = $oldHome
    $env:APPDATA = $oldAppData
    $env:LOCALAPPDATA = $oldLocalAppData
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
