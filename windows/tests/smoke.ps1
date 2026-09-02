# Smoke seguro del ejecutable Windows. No instala, borra ni modifica el sistema.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Binary,
    [string]$CliBinary,
    [string]$Version
)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Binary)) { throw "No existe el ejecutable: $Binary" }
if (-not $Version) {
    $cargoManifest = Join-Path $PSScriptRoot '..\..\rust\Cargo.toml'
    $Version = ((Select-String -Path $cargoManifest -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value)
}
if (-not $Version) { throw 'No se pudo resolver la versión desde rust/Cargo.toml.' }
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
        return $output.Trim()
    }
    Run @('--version')
    Run @('--help')
    $germanHelp = Run @('--lang', 'de', '--help')
    if ($germanHelp -notmatch 'Verwendung:' -or $germanHelp -notmatch 'Datenträger') {
        throw 'Las traducciones alemanas del help Windows no se aplicaron completamente.'
    }
    Run @('doctor')
    Run @('defaults')
    Run @('storage', 'tools')
    Run @('registry', 'status')
    if ($CliBinary -and (Test-Path -LiteralPath $CliBinary)) {
        $cliOutput = & $CliBinary 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0 -or $cliOutput -notmatch 'Uso: ltools') {
            throw 'El ejecutable CLI Windows sin argumentos no muestra la ayuda.'
        }
    }
    $oldCliProfile = $env:LTOOLS_CLI
    $env:LTOOLS_CLI = '1'
    try {
        $cliOutput = & $Binary 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0 -or $cliOutput -notmatch 'Uso: ltools') {
            throw 'El perfil CLI Windows no muestra la ayuda sin argumentos.'
        }
    } finally {
        if ($null -eq $oldCliProfile) { Remove-Item Env:LTOOLS_CLI -ErrorAction SilentlyContinue }
        else { $env:LTOOLS_CLI = $oldCliProfile }
    }
    $capabilityOutput = Run @('capabilities', '--format', 'json')
    $capabilityJson = $capabilityOutput | ConvertFrom-Json
    if ($capabilityJson.host_tools.Count -lt 6 -or
        -not ($capabilityJson.host_tools | Where-Object { $_.category -eq 'system' }) -or
        -not ($capabilityJson.host_tools | Where-Object { $_.command -eq 'sc.exe' })) {
        throw 'El catálogo Windows de herramientas del anfitrión está incompleto.'
    }
    if ($capabilityJson.host_tools | Where-Object { $_.category -in @('games', 'virtualization', 'development') }) {
        throw 'El catálogo Windows incluye una categoría fuera de alcance.'
    }
    if ($capabilityJson.host_tools | Where-Object {
            $_.category -in @('games', 'virtualization', 'development') -or
            $_.command -in @('steam', 'wine', 'git', 'pacman', 'apt-get', 'systemctl')
        }) {
        throw 'El catálogo Windows incluye una herramienta fuera de alcance.'
    }
    $compose = @($capabilityJson.host_tools | Where-Object { $_.id -eq 'docker-compose' })
    $kubectl = @($capabilityJson.host_tools | Where-Object { $_.id -eq 'kubectl' })
    if ($compose.Count -ne 1 -or -not $compose[0].installable -or
        $kubectl.Count -ne 1 -or -not $kubectl[0].installable) {
        throw 'Windows no declara Compose y kubectl como instaladores principales.'
    }
    $alternatives = @($capabilityJson.host_tools | Where-Object {
            $_.id -in @('docker', 'podman', 'podman-compose', 'helm', 'kind', 'minikube', 'k3d', 'k9s')
        })
    if ($alternatives | Where-Object { $_.installable }) {
        throw 'Windows ofrece instalación automática para una alternativa no principal.'
    }
    if ($capabilityJson.host_tools | Where-Object { $_.available -and $null -eq $_.version }) {
        throw 'El catálogo Windows no incluye el campo version en herramientas detectadas.'
    }
    if ($capabilityJson.features -contains 'wine-prefixes' -or
        $capabilityJson.features -contains 'lutris' -or
        $capabilityJson.features -contains 'heroic' -or
        $capabilityJson.features -contains 'umu') {
        throw 'El contrato de capacidades Windows anuncia funciones Linux/Wine.'
    }
    $releaseAssets = Join-Path $temp 'release-assets'
    New-Item -ItemType Directory -Force -Path $releaseAssets | Out-Null
    $releaseFixture = Join-Path $releaseAssets "ltools-$Version-windows-x86_64.exe"
    Set-Content -Encoding UTF8 $releaseFixture 'synthetic-windows-release'
    $releaseManifest = Join-Path $temp 'ltools-release.json'
    Run @('release-manifest', '--output', $releaseManifest, '--repository', 'Darkeiser003/Tools', '--tag', "v$Version", '--artifacts-dir', $releaseAssets)
    $releaseJson = Get-Content -Raw $releaseManifest | ConvertFrom-Json
    if ($releaseJson.schema -ne 'ltools-release-v1' -or $releaseJson.artifacts.Count -ne 1 -or
        $releaseJson.artifacts[0].platform -ne 'windows' -or
        $releaseJson.artifacts[0].sha256 -notmatch '^[a-f0-9]{64}$') {
        throw 'El manifiesto de release Windows no contiene los metadatos esperados.'
    }
    Run @('audit', '--root', $temp, '--no-mounts', '--out', (Join-Path $temp 'report'))
    if (-not (Test-Path (Join-Path $temp 'report\summary.txt'))) { throw 'El resumen del informe Windows no se generó.' }
    $prefixReport = Join-Path $temp 'report\wine-prefixes.tsv'
    if (-not (Test-Path $prefixReport)) { throw 'El informe de compatibilidad Windows no se generó.' }
    if ((Get-Content $prefixReport | Select-Object -Skip 1 | Where-Object { $_.Trim() }).Count -ne 0) {
        throw 'La auditoría Windows encontró prefijos Wine; no debe escanearlos en esta plataforma.'
    }
    New-Item -ItemType Directory -Force -Path (Join-Path $temp 'Epic Games\ExampleGame') | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $temp 'Steam\steamapps') | Out-Null
    Set-Content -Encoding UTF8 (Join-Path $temp 'Steam\steamapps\appmanifest_123.acf') '"name" "Native Example"'
    $gamesReport = Join-Path $temp 'games-report'
    $gamesOutput = Run @('games', '--root', $temp, '--out', $gamesReport)
    if (-not (Test-Path (Join-Path $gamesReport 'windows-game-launchers.tsv'))) {
        throw 'El inventario nativo Windows de juegos no se generó.'
    }
    $launcherRows = Get-Content (Join-Path $gamesReport 'windows-game-launchers.tsv') | Select-Object -Skip 1
    if ($launcherRows -match 'Wine|Lutris|Heroic|UMU') {
        throw 'Las filas del inventario Windows incluyen detectores Linux/Wine.'
    }
    if ($launcherRows -notmatch 'Steam\tmanifest') {
        throw 'El inventario Windows no recogió el manifiesto nativo de Steam.'
    }
    if ($gamesOutput -notmatch 'Auditoría nativa de juegos Windows') {
        throw 'El inventario Windows no se identificó como nativo.'
    }
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
