# Smoke seguro del ejecutable Windows. No instala, borra ni modifica el sistema.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Binary,
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
        Write-Host $output.Trim()
    }
    Run @('--version')
    Run @('--help')
    Run @('doctor')
    Run @('defaults')
    $capabilityOutput = Run @('capabilities', '--format', 'json')
    $capabilityJson = $capabilityOutput | ConvertFrom-Json
    if ($capabilityJson.host_tools.Count -lt 5 -or
        -not ($capabilityJson.host_tools | Where-Object { $_.category -eq 'system' }) -or
        -not ($capabilityJson.host_tools | Where-Object { $_.command -eq 'sc.exe' }) -or
        -not ($capabilityJson.host_tools | Where-Object { $_.installable -eq $true })) {
        throw 'El catálogo Windows de herramientas del anfitrión está incompleto.'
    }
    if ($capabilityJson.host_tools | Where-Object { $_.category -in @('games', 'virtualization', 'development') }) {
        throw 'El catálogo Windows incluye una categoría fuera de alcance.'
    }
    if ($capabilityJson.host_tools | Where-Object {
            $_.category -in @('games', 'virtualization', 'development') -or
            $_.command -in @('steam', 'wine', 'docker', 'git')
        }) {
        throw 'El catálogo Windows incluye una herramienta fuera de alcance.'
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
