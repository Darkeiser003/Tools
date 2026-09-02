[CmdletBinding()]
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)
$ErrorActionPreference = 'Stop'

$candidates = @(
    (Join-Path $PSScriptRoot 'ltools-cli.exe'),
    (Join-Path $PSScriptRoot 'ltools.exe')
)
$root = Split-Path -Parent $PSScriptRoot
$cargoManifest = Join-Path $root 'rust\Cargo.toml'
if (Test-Path -LiteralPath $cargoManifest) {
    $match = Select-String -Path $cargoManifest -Pattern '^\s*version\s*=\s*"([^"]+)"' |
        Select-Object -First 1
    if ($match) {
        $version = $match.Matches.Groups[1].Value
        $output = Join-Path $root 'dist\windows'
        $release = Join-Path $root 'release'
        $candidates += Join-Path $output "ltools-$version-windows-x86_64-cli.exe"
        $candidates += Join-Path $output "ltools-$version-windows-x86_64\ltools-cli.exe"
        $candidates += Join-Path $release "ltools-$version-windows-x86_64-cli.exe"
    }
}
$release = Join-Path $root 'release'
if (Test-Path -LiteralPath $release) {
    $candidates += @(
        Get-ChildItem -LiteralPath $release -File -Filter 'ltools-*-windows-*-cli.exe' -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTimeUtc -Descending |
            ForEach-Object { $_.FullName }
    )
}
$binary = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
if (-not $binary) {
    throw 'No se encontró ltools-cli.exe. Ejecuta el builder Windows o coloca el perfil CLI junto a este lanzador.'
}
$env:LTOOLS_CLI = '1'
& $binary @Arguments
$exitCode = $LASTEXITCODE
if ($null -eq $exitCode) { $exitCode = 0 }
exit $exitCode
