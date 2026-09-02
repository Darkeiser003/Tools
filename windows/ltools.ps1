[CmdletBinding()]
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$candidates = New-Object System.Collections.Generic.List[string]
$candidates.Add((Join-Path $PSScriptRoot 'ltools.exe'))

# Permite ejecutar windows\ltools.ps1 directamente desde el repositorio,
# después de una build, o desde la carpeta portable ya empaquetada.
$cargoManifest = Join-Path $root 'rust\Cargo.toml'
$versionMatch = $null
if (Test-Path -LiteralPath $cargoManifest) {
    $versionMatch = Select-String -Path $cargoManifest -Pattern '^\s*version\s*=\s*"([^"]+)"' |
        Select-Object -First 1
}
$version = if ($versionMatch) { $versionMatch.Matches.Groups[1].Value } else { $null }
$windowsOutput = Join-Path $root 'dist\windows'
$releaseOutput = Join-Path $root 'release'
if ($version) {
    $candidates.Add((Join-Path $windowsOutput "ltools-$version-windows-x86_64.exe"))
    $candidates.Add((Join-Path $windowsOutput "ltools-$version-windows-x86_64\ltools.exe"))
    $candidates.Add((Join-Path $releaseOutput "ltools-$version-windows-x86_64.exe"))
}
foreach ($target in @('x86_64-pc-windows-msvc', 'x86_64-pc-windows-gnu')) {
    $candidates.Add((Join-Path $root "rust\target\windows\$target\release\ltools.exe"))
}
if (Test-Path -LiteralPath $windowsOutput) {
    Get-ChildItem -LiteralPath $windowsOutput -File -Filter 'ltools-*.exe' -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTimeUtc -Descending |
        ForEach-Object { $candidates.Add($_.FullName) }
    Get-ChildItem -LiteralPath $windowsOutput -Directory -Filter 'ltools-*windows-*' -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTimeUtc -Descending |
        ForEach-Object {
            $portableBinary = Join-Path $_.FullName 'ltools.exe'
            if (Test-Path -LiteralPath $portableBinary) { $candidates.Add($portableBinary) }
        }
}
$binary = $null
foreach ($candidate in $candidates) {
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
        $binary = (Get-Item -LiteralPath $candidate).FullName
        break
    }
}
if (Test-Path -LiteralPath $releaseOutput) {
    Get-ChildItem -LiteralPath $releaseOutput -File -Filter 'ltools-*-windows-*.exe' -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -notlike '*-cli.exe' } |
        Sort-Object LastWriteTimeUtc -Descending |
        ForEach-Object { $candidates.Add($_.FullName) }
}
if (-not $binary) {
    throw "No se encontró ltools.exe. Ejecuta build.cmd o windows\build.ps1 para generarlo, o coloca el ejecutable junto a este lanzador."
}
& $binary @Arguments
$exitCode = $LASTEXITCODE
if ($null -eq $exitCode) { $exitCode = 0 }
exit $exitCode
