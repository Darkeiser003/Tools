# E2E Windows aislada: inventario nativo y operaciones del sistema.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Binary)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Binary)) { throw "No existe el ejecutable: $Binary" }
$temp = Join-Path ([IO.Path]::GetTempPath()) ("ltools-windows-e2e-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path (Join-Path $temp 'Epic Games\ExampleGame') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $temp 'Rockstar Games\ExampleGame') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $temp 'Steam\steamapps') | Out-Null
Set-Content -Encoding UTF8 (Join-Path $temp 'Steam\steamapps\appmanifest_123.acf') '"name" "Native Example"'
New-Item -ItemType Directory -Force -Path (Join-Path $temp '.wine\drive_c') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $temp 'Lutris\games') | Out-Null
$oldUserProfile = $env:USERPROFILE
$oldHome = $env:HOME
$oldAppData = $env:APPDATA
$oldLocalAppData = $env:LOCALAPPDATA
$env:USERPROFILE = $temp
$env:HOME = $temp
$env:APPDATA = Join-Path $temp 'AppData\Roaming'
$env:LOCALAPPDATA = Join-Path $temp 'AppData\Local'

function Quote-Arg([string]$Value) {
    return '"' + $Value.Replace('"', '\"') + '"'
}
function Run-WithInput([string[]]$Arguments, [string]$InputText) {
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $Binary
    $info.Arguments = (($Arguments | ForEach-Object { Quote-Arg $_ }) -join ' ')
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $info
    [void]$process.Start()
    $process.StandardInput.Write($InputText)
    $process.StandardInput.Close()
    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw "Falló ltools.exe E2E: $stdout $stderr" }
    Write-Host $stdout.Trim()
}

try {
    Run-WithInput @() ("q" + [Environment]::NewLine)
    $output = & $Binary 'prefix' 'list' '--root' $temp 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $output -notmatch 'no aplican') { throw 'Windows no bloqueó la lógica de prefijos Wine.' }
    $games = & $Binary 'games' '--root' $temp '--out' (Join-Path $temp 'games-report') 2>&1 | Out-String
    $launcherRows = Get-Content (Join-Path $temp 'games-report\windows-game-launchers.tsv') | Select-Object -Skip 1
    if ($LASTEXITCODE -ne 0 -or $launcherRows -match 'Wine|Lutris|Heroic|UMU' -or
        $games -notmatch 'Auditoría nativa de juegos Windows') {
        throw 'El inventario de juegos Windows mezcló detectores Linux/Wine.'
    }
    if ($launcherRows -notmatch 'Steam\tmanifest') {
        throw 'El inventario Windows no recogió el manifiesto nativo de Steam.'
    }
    $capabilities = & $Binary 'capabilities' '--format' 'json' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $capabilities -notmatch 'ltools-capabilities-v1') {
        throw 'El contrato JSON Windows anuncia o mezcla funciones Linux/Wine.'
    }
    $capabilityJson = $capabilities | ConvertFrom-Json
    if ($capabilityJson.features -contains 'wine-prefixes' -or
        $capabilities -match 'Heroic|Lutris|UMU') {
        throw 'El contrato JSON Windows anuncia o mezcla funciones Linux/Wine.'
    }
    Run-WithInput @('system', '--dry-run', 'service', 'restart', 'EventLog') ("y" + [Environment]::NewLine)
    $storageOutput = & $Binary 'storage' 'tools' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $storageOutput -notmatch 'diskpart') { throw 'El módulo Windows de almacenamiento falló.' }
    $storageStatus = & $Binary 'storage' 'status' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $storageStatus -notmatch 'Almacenamiento Windows') { throw 'El estado de almacenamiento Windows falló.' }
    $storagePartitions = & $Binary 'storage' 'partitions' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $storagePartitions -notmatch 'Discos y particiones Windows') { throw 'El inventario de particiones Windows falló.' }
    $registryOutput = & $Binary 'registry' 'status' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $registryOutput -notmatch 'Registro Windows') { throw 'El módulo Windows de registro falló.' }
    $oldCliProfile = $env:LTOOLS_CLI
    $env:LTOOLS_CLI = '1'
    try {
        $cliOutput = & $Binary 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0 -or $cliOutput -notmatch 'Uso: ltools') { throw 'El perfil CLI Windows falló.' }
    } finally {
        if ($null -eq $oldCliProfile) { Remove-Item Env:LTOOLS_CLI -ErrorAction SilentlyContinue }
        else { $env:LTOOLS_CLI = $oldCliProfile }
    }
    $processOutput = & $Binary 'system' 'processes' '--sort' 'memory' '--limit' '3' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $processOutput -notmatch 'Procesos Windows') { throw 'El listado de procesos Windows falló.' }
    Write-Host 'Windows E2E completado correctamente.'
} finally {
    $env:USERPROFILE = $oldUserProfile
    $env:HOME = $oldHome
    $env:APPDATA = $oldAppData
    $env:LOCALAPPDATA = $oldLocalAppData
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
