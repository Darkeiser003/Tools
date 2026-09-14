$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $root 'scripts\lib\build-state.ps1')

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-Impact([string]$Name, $Impact, [string[]]$Required, [string[]]$Forbidden) {
    foreach ($property in $Required) {
        if (@($Impact.$property).Count -eq 0) { throw "$Name no activó $property" }
    }
    foreach ($property in $Forbidden) {
        if (@($Impact.$property).Count -ne 0) { throw "$Name activó suites no relacionadas: $property" }
    }
}

$publishChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/lib/publish.ps1' = 'old' }) ([ordered]@{ 'scripts/lib/publish.ps1' = 'new' })
Assert-Impact 'cambio del publicador' $publishChange @('Package', 'PublishTests') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$linuxOnlyChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/build.sh' = 'old' }) ([ordered]@{ 'scripts/build.sh' = 'new' })
Assert-Impact 'cambio de builder Linux' $linuxOnlyChange @() @('Package', 'RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E', 'PublishTests')

$rustTestChange = Get-LToolsBuildImpact ([ordered]@{ 'rust/tests/cli.rs' = 'old' }) ([ordered]@{ 'rust/tests/cli.rs' = 'new' })
Assert-Impact 'cambio de tests Rust' $rustTestChange @('CargoTests') @('RustCompile', 'Package', 'WindowsSmoke', 'WindowsE2E')

$windowsE2EChange = Get-LToolsBuildImpact ([ordered]@{ 'windows/tests/e2e.ps1' = 'old' }) ([ordered]@{ 'windows/tests/e2e.ps1' = 'new' })
Assert-Impact 'cambio de E2E Windows' $windowsE2EChange @('WindowsE2E') @('RustCompile', 'Package', 'CargoTests', 'WindowsSmoke')

$builderChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/build.ps1' = 'old' }) ([ordered]@{ 'scripts/build.ps1' = 'new' })
Assert-Impact 'cambio de builder Windows' $builderChange @('Package', 'BuildStateTests') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$deletedChange = Get-LToolsBuildImpact ([ordered]@{ 'windows/ltools.ps1' = 'old' }) ([ordered]@{})
Assert-True ($deletedChange.Package -contains 'windows/ltools.ps1') 'La eliminación de una entrada de paquete no se detectó.'

$workspace = Join-Path ([IO.Path]::GetTempPath()) "ltools-build-state-test-$PID-$([guid]::NewGuid().ToString('N'))"
$output = Join-Path $workspace 'output'
$release = Join-Path $workspace 'release'
$dist = Join-Path $workspace 'dist'
New-Item -ItemType Directory -Path $output, $release, $dist -Force | Out-Null
try {
    $fingerprintA = Get-LToolsPublicKeyFingerprint $null "  AB CD`nEF  "
    $fingerprintEquivalent = Get-LToolsPublicKeyFingerprint $null 'abcdef'
    $fingerprintB = Get-LToolsPublicKeyFingerprint $null 'abcdef01'
    Assert-True ($fingerprintA -eq $fingerprintEquivalent) 'La huella debe identificar la clave normalizada, ignorando espacios.'
    Assert-True ($fingerprintA -ne $fingerprintB) 'Cambiar la clave pública debe invalidar la identidad de firma.'

    $artifact = Join-Path $release 'ltools-1.2.3-windows-x86_64.exe'
    Set-Content -LiteralPath $artifact -Value 'known release' -NoNewline
    $firstHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    $signing = [ordered]@{ required = $false; publicKeyFingerprint = $null; signed = $false }
    Assert-True (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $false $null) 'Artefactos idénticos no se reconocieron como actuales.'

    Set-Content -LiteralPath $artifact -Value 'tampered release' -NoNewline
    $tamperedHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $tamperedHashes $signing $false $null)) 'No se detectó un artefacto alterado.'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $false $fingerprintB)) 'No se detectó el cambio de clave pública.'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $true $null)) 'No se detectó el cambio entre modo firmado y no firmado.'
    Assert-True (Get-LToolsPackagePending $true $false) '-NoPackage no dejó la publicación pendiente.'
    Assert-True (-not (Get-LToolsPackagePending $true $true)) 'Una publicación completada quedó pendiente.'

    Remove-Item -LiteralPath $artifact -Force
    $missingHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $tamperedHashes $missingHashes $signing $false $null)) 'No se detectó un artefacto ausente.'
} finally {
    Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Output 'Matriz de suites, hashes de artefactos, identidad de firma y paquete aplazado validados.'
