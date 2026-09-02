# E2E Windows aislada: inventario nativo y operaciones del sistema.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Binary)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) { throw "No existe el ejecutable: $Binary" }
$temp = Join-Path ([IO.Path]::GetTempPath()) ("ltools-windows-e2e-" + [guid]::NewGuid().ToString('N'))
$fixtureRoot = Join-Path $temp 'fixture con espacios y ñ'
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Epic Games\ExampleGame') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Rockstar Games\ExampleGame') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Steam\steamapps') | Out-Null
Set-Content -Encoding UTF8 (Join-Path $fixtureRoot 'Steam\steamapps\appmanifest_123.acf') '"name" "Native Example"'
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot '.wine\drive_c') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Lutris\games') | Out-Null
$oldUserProfile = $env:USERPROFILE
$oldHome = $env:HOME
$oldAppData = $env:APPDATA
$oldLocalAppData = $env:LOCALAPPDATA
$oldLanguage = $env:LTOOLS_LANG
$oldCliMode = $env:LTOOLS_CLI
$oldNoClear = $env:LTOOLS_NO_CLEAR
$env:USERPROFILE = $temp
$env:HOME = $temp
$env:APPDATA = Join-Path $temp 'AppData\Roaming'
$env:LOCALAPPDATA = Join-Path $temp 'AppData\Local'
$env:LTOOLS_LANG = 'es'
Remove-Item Env:LTOOLS_CLI -ErrorAction SilentlyContinue
$env:LTOOLS_NO_CLEAR = '1'
$processHelper = Join-Path $PSScriptRoot 'native-process.ps1'
if (-not (Test-Path -LiteralPath $processHelper)) { throw "Falta el helper de procesos: $processHelper" }
. $processHelper

function Run([string[]]$Arguments) {
    Write-Host ("  [RUN] ltools.exe {0}" -f ($Arguments -join ' '))
    $result = Invoke-NativeProcess -FileName $Binary -Arguments $Arguments
    if ($result.ExitCode -ne 0) {
        throw (Format-NativeProcessFailure $result "ltools.exe $($Arguments -join ' ')")
    }
    $output = [string]$result.Stdout
    if ([string]::IsNullOrWhiteSpace($output) -and
        -not [string]::IsNullOrWhiteSpace($result.Stderr)) { $output = [string]$result.Stderr }
    return [string](([string]$output).Trim())
}
function Run-WithInput([string[]]$Arguments, [string]$InputText) {
    Write-Host ("  [RUN-INPUT] ltools.exe {0}" -f ($Arguments -join ' '))
    $result = Invoke-NativeProcess -FileName $Binary -Arguments $Arguments -InputText $InputText
    if ($result.ExitCode -ne 0) {
        throw (Format-NativeProcessFailure $result 'ltools.exe E2E')
    }
    Write-Host ([string](([string]$result.Stdout).Trim()))
}

try {
    Run-WithInput @() ("q" + [Environment]::NewLine)
    $output = Run @('prefix', 'list', '--root', $fixtureRoot)
    if ($output -notmatch 'no aplican') { throw 'Windows no bloqueó la lógica de prefijos Wine.' }
    $gamesReport = Join-Path $temp 'games report con espacios y ñ'
    $games = Run @('games', '--root', $fixtureRoot, '--out', $gamesReport)
    $launcherRows = Get-Content -LiteralPath (Join-Path $gamesReport 'windows-game-launchers.tsv') | Select-Object -Skip 1
    if ($launcherRows -match 'Wine|Lutris|Heroic|UMU' -or
        $games -notmatch 'Auditoría nativa de juegos Windows') {
        throw 'El inventario de juegos Windows mezcló detectores Linux/Wine.'
    }
    if ($launcherRows -notmatch 'Steam\tmanifest') {
        throw 'El inventario Windows no recogió el manifiesto nativo de Steam.'
    }
    $capabilities = Run @('capabilities', '--format', 'json')
    if ($capabilities -notmatch 'ltools-capabilities-v1') {
        throw 'El contrato JSON Windows anuncia o mezcla funciones Linux/Wine.'
    }
    $capabilityJson = $capabilities | ConvertFrom-Json
    if ($capabilityJson.application -ne 'WinSlim-Tools' -or $capabilityJson.platform -ne 'windows') {
        throw 'La identidad Windows del contrato no es WinSlim-Tools.'
    }
    if ($capabilityJson.features -contains 'wine-prefixes' -or
        $capabilities -match 'Heroic|Lutris|UMU') {
        throw 'El contrato JSON Windows anuncia o mezcla funciones Linux/Wine.'
    }
    Run-WithInput @('system', '--dry-run', 'service', 'restart', 'EventLog') ("y" + [Environment]::NewLine)
    $storageOutput = Run @('storage', 'tools')
    if ($storageOutput -notmatch 'diskpart') { throw 'El módulo Windows de almacenamiento falló.' }
    $storageStatus = Run @('storage', 'status')
    if ($storageStatus -notmatch 'Almacenamiento Windows') { throw 'El estado de almacenamiento Windows falló.' }
    $storagePartitions = Run @('storage', 'partitions')
    if ($storagePartitions -notmatch 'Discos y particiones Windows') { throw 'El inventario de particiones Windows falló.' }
    $registryOutput = Run @('registry', 'status')
    if ($registryOutput -notmatch 'Registro Windows') { throw 'El módulo Windows de registro falló.' }
    $cliResult = Invoke-NativeProcess -FileName $Binary -EnvironmentOverrides @{ LTOOLS_CLI = '1' } -TimeoutSeconds 15
    $cliOutput = [string]$cliResult.Stdout + [string]$cliResult.Stderr
    if ($cliResult.ExitCode -ne 0 -or $cliOutput -notmatch 'Uso: ltools|Usage: ltools') {
        throw (Format-NativeProcessFailure $cliResult 'El perfil CLI Windows falló')
    }
    $processOutput = Run @('system', 'processes', '--sort', 'memory', '--limit', '3')
    if ($processOutput -notmatch 'Procesos Windows') { throw 'El listado de procesos Windows falló.' }
    Write-Host 'Windows E2E completado correctamente.'
} finally {
    $env:USERPROFILE = $oldUserProfile
    $env:HOME = $oldHome
    $env:APPDATA = $oldAppData
    $env:LOCALAPPDATA = $oldLocalAppData
    if ($null -eq $oldLanguage) { Remove-Item Env:LTOOLS_LANG -ErrorAction SilentlyContinue }
    else { $env:LTOOLS_LANG = $oldLanguage }
    if ($null -eq $oldCliMode) { Remove-Item Env:LTOOLS_CLI -ErrorAction SilentlyContinue }
    else { $env:LTOOLS_CLI = $oldCliMode }
    if ($null -eq $oldNoClear) { Remove-Item Env:LTOOLS_NO_CLEAR -ErrorAction SilentlyContinue }
    else { $env:LTOOLS_NO_CLEAR = $oldNoClear }
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
