[CmdletBinding()]
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)
$ErrorActionPreference = 'Stop'
$binary = Join-Path $PSScriptRoot 'ltools.exe'
if (-not (Test-Path $binary)) { throw "No se encontró ltools.exe junto a este lanzador." }
& $binary @Arguments
exit $LASTEXITCODE
