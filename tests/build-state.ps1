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

$licenseBundleChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/lib/third-party-licenses.ps1' = 'old' }) ([ordered]@{ 'scripts/lib/third-party-licenses.ps1' = 'new' })
Assert-Impact 'cambio del generador de licencias' $licenseBundleChange @('Package') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$linuxOnlyChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/build.sh' = 'old' }) ([ordered]@{ 'scripts/build.sh' = 'new' })
Assert-Impact 'cambio de builder Linux' $linuxOnlyChange @() @('Package', 'RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E', 'PublishTests')

$rustTestChange = Get-LToolsBuildImpact ([ordered]@{ 'rust/tests/cli.rs' = 'old' }) ([ordered]@{ 'rust/tests/cli.rs' = 'new' })
Assert-Impact 'cambio de tests Rust' $rustTestChange @('CargoTests') @('RustCompile', 'Package', 'WindowsSmoke', 'WindowsE2E')

$rustTestDeletion = Get-LToolsBuildImpact ([ordered]@{ 'rust/tests/cli.rs' = 'old' }) ([ordered]@{})
Assert-Impact 'eliminación de tests Rust' $rustTestDeletion @('CargoTests') @('RustCompile', 'Package', 'WindowsSmoke', 'WindowsE2E')

$distributionChange = Get-LToolsBuildImpact ([ordered]@{ 'distribution/ltools-project.json' = 'old' }) ([ordered]@{ 'distribution/ltools-project.json' = 'new' })
Assert-Impact 'cambio del descriptor de distribución' $distributionChange @('Package') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$readmeChange = Get-LToolsBuildImpact ([ordered]@{ 'README.md' = 'old' }) ([ordered]@{ 'README.md' = 'new' })
Assert-Impact 'cambio del README empaquetado' $readmeChange @('Package') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

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
    $publicKeyFile = Join-Path $workspace 'release-public.hex'
    Set-Content -LiteralPath $publicKeyFile -Value 'FILEKEY' -NoNewline
    $environmentWins = Get-LToolsPublicKeyFingerprint $publicKeyFile 'ENVKEY'
    $environmentExpected = Get-LToolsPublicKeyFingerprint $null 'ENVKEY'
    Assert-True ($environmentWins -eq $environmentExpected) 'La clave de entorno efectiva debe prevalecer sobre el archivo también en la firma incremental.'
    $blankEnvironment = Get-LToolsPublicKeyFingerprint $publicKeyFile " `n `t"
    $fileExpected = Get-LToolsPublicKeyFingerprint $publicKeyFile $null
    Assert-True ($blankEnvironment -eq $fileExpected) 'Una clave de entorno vacía no debe ocultar la clave de archivo efectiva.'
    Assert-True (Test-LToolsSigningKeyChanged ([ordered]@{ publicKeyFingerprint = $fingerprintA }) $fingerprintB) 'Cambiar la clave pública debe invalidar también la compilación que la fija.'
    Assert-True (-not (Test-LToolsSigningKeyChanged ([ordered]@{ publicKeyFingerprint = $fingerprintA }) $fingerprintA)) 'Una clave pública sin cambios no debe forzar una recompilación.'

    $artifact = Join-Path $release 'ltools-1.2.3-windows-x86_64.exe'
    Set-Content -LiteralPath $artifact -Value 'known release' -NoNewline
    Set-Content -LiteralPath (Join-Path $release 'LICENSE') -Value 'MIT License' -NoNewline
    Set-Content -LiteralPath (Join-Path $release 'SHA256SUMS.txt.sshsig') -Value 'known SSH signature' -NoNewline
    $licenseZipName = 'THIRD-PARTY-LICENSES-windows.zip'
    Set-Content -LiteralPath (Join-Path $output $licenseZipName) -Value 'known licenses archive' -NoNewline
    Set-Content -LiteralPath (Join-Path $release $licenseZipName) -Value 'known licenses archive' -NoNewline
    $portableLicenses = Join-Path $output 'ltools-1.2.3-windows-x86_64\THIRD-PARTY-LICENSES'
    New-Item -ItemType Directory -Path $portableLicenses -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $portableLicenses 'INDEX.txt') -Value 'known license inventory' -NoNewline
    $firstHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    Assert-True ($firstHashes['output/THIRD-PARTY-LICENSES-windows.zip'] -ne 'missing' -and
        $firstHashes['portable/THIRD-PARTY-LICENSES/INDEX.txt'] -ne 'missing' -and
        $firstHashes['release/THIRD-PARTY-LICENSES-windows.zip'] -ne 'missing') 'No se incluyeron las licencias en las huellas del estado incremental.'
    $signing = [ordered]@{ required = $false; publicKeyFingerprint = $null; signed = $false; sshPublicKeyFingerprint = 'ssh-fingerprint'; sshIdentity = 'test@example.invalid'; sshSigned = $true }
    Assert-True (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $false $null 'ssh-fingerprint' 'test@example.invalid') 'Artefactos idénticos no se reconocieron como actuales.'

    Set-Content -LiteralPath $artifact -Value 'tampered release' -NoNewline
    $tamperedHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $tamperedHashes $signing $false $null 'ssh-fingerprint' 'test@example.invalid')) 'No se detectó un artefacto alterado.'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $false $fingerprintB 'ssh-fingerprint' 'test@example.invalid')) 'No se detectó el cambio de clave pública.'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $false $null 'other-ssh-fingerprint' 'test@example.invalid')) 'No se detectó el cambio de clave SSH.'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $false $null 'ssh-fingerprint' 'other@example.invalid')) 'No se detectó el cambio de identidad SSH.'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $true $null 'ssh-fingerprint' 'test@example.invalid')) 'No se detectó el cambio entre modo firmado y no firmado.'
    Assert-True (Get-LToolsPackagePending $true $false) '-NoPackage no dejó la publicación pendiente.'
    Assert-True (-not (Get-LToolsPackagePending $true $true)) 'Una publicación completada quedó pendiente.'

    Set-Content -LiteralPath (Join-Path $portableLicenses 'INDEX.txt') -Value 'tampered license inventory' -NoNewline
    $tamperedLicenseHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $tamperedLicenseHashes $signing $false $null 'ssh-fingerprint' 'test@example.invalid')) 'No se detectó la alteración de un aviso de terceros incluido en el ZIP.'

    Remove-Item -LiteralPath $artifact -Force
    $missingHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $tamperedHashes $missingHashes $signing $false $null 'ssh-fingerprint' 'test@example.invalid')) 'No se detectó un artefacto ausente.'
} finally {
    Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Output 'Matriz de suites, hashes de artefactos, identidad de firma y paquete aplazado validados.'
