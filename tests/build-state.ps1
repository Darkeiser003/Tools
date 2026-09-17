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
Assert-Impact 'cambio del generador de licencias' $licenseBundleChange @('Package', 'PublishTests') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$sshSigningChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/lib/ssh-signing.ps1' = 'old' }) ([ordered]@{ 'scripts/lib/ssh-signing.ps1' = 'new' })
Assert-Impact 'cambio de la firma SSH' $sshSigningChange @('Package', 'PublishTests') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$publishTestChange = Get-LToolsBuildImpact ([ordered]@{ 'tests/build-publish.ps1' = 'old' }) ([ordered]@{ 'tests/build-publish.ps1' = 'new' })
Assert-Impact 'cambio de la suite de publicación' $publishTestChange @('PublishTests') @('Package', 'RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$linuxOnlyChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/build.sh' = 'old' }) ([ordered]@{ 'scripts/build.sh' = 'new' })
Assert-Impact 'cambio de builder Linux' $linuxOnlyChange @() @('Package', 'RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E', 'PublishTests')

$rustTestChange = Get-LToolsBuildImpact ([ordered]@{ 'rust/tests/cli.rs' = 'old' }) ([ordered]@{ 'rust/tests/cli.rs' = 'new' })
Assert-Impact 'cambio de tests Rust' $rustTestChange @('CargoTests') @('RustCompile', 'Package', 'WindowsSmoke', 'WindowsE2E')

$rustTestDeletion = Get-LToolsBuildImpact ([ordered]@{ 'rust/tests/cli.rs' = 'old' }) ([ordered]@{})
Assert-Impact 'eliminación de tests Rust' $rustTestDeletion @('CargoTests') @('RustCompile', 'Package', 'WindowsSmoke', 'WindowsE2E')

$fuzzPolicyChange = Get-LToolsBuildImpact ([ordered]@{ 'fuzz/deny.toml' = 'old' }) ([ordered]@{ 'fuzz/deny.toml' = 'new' })
Assert-Impact 'cambio de la política del fuzzer' $fuzzPolicyChange @() @('RustCompile', 'Package', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$rustSharedLibraryChange = Get-LToolsBuildImpact ([ordered]@{ 'rust/crates/native-argv/src/lib.rs' = 'old' }) ([ordered]@{ 'rust/crates/native-argv/src/lib.rs' = 'new' })
Assert-Impact 'cambio de biblioteca Rust compartida' $rustSharedLibraryChange @('RustCompile', 'Package', 'CargoTests', 'WindowsSmoke', 'WindowsE2E') @()

$distributionChange = Get-LToolsBuildImpact ([ordered]@{ 'distribution/ltools-project.json' = 'old' }) ([ordered]@{ 'distribution/ltools-project.json' = 'new' })
Assert-Impact 'cambio del descriptor de distribución' $distributionChange @('Package') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$readmeChange = Get-LToolsBuildImpact ([ordered]@{ 'README.md' = 'old' }) ([ordered]@{ 'README.md' = 'new' })
Assert-Impact 'cambio del README empaquetado' $readmeChange @('Package') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$windowsE2EChange = Get-LToolsBuildImpact ([ordered]@{ 'windows/tests/e2e.ps1' = 'old' }) ([ordered]@{ 'windows/tests/e2e.ps1' = 'new' })
Assert-Impact 'cambio de E2E Windows' $windowsE2EChange @('WindowsE2E') @('RustCompile', 'Package', 'CargoTests', 'WindowsSmoke')

$nativeProcessHelperChange = Get-LToolsBuildImpact ([ordered]@{ 'windows/tests/native-process.ps1' = 'old' }) ([ordered]@{ 'windows/tests/native-process.ps1' = 'new' })
Assert-Impact 'cambio del helper de procesos Windows' $nativeProcessHelperChange @('WindowsSmoke', 'WindowsE2E', 'ReleaseTests') @('RustCompile', 'Package', 'CargoTests')

$builderChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/build.ps1' = 'old' }) ([ordered]@{ 'scripts/build.ps1' = 'new' })
Assert-Impact 'cambio de builder Windows' $builderChange @('Package', 'BuildStateTests', 'PublishTests') @('RustCompile', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$buildStateHelperChange = Get-LToolsBuildImpact ([ordered]@{ 'scripts/lib/build-state.ps1' = 'old' }) ([ordered]@{ 'scripts/lib/build-state.ps1' = 'new' })
Assert-Impact 'cambio del estado incremental' $buildStateHelperChange @('BuildStateTests') @('RustCompile', 'Package', 'CargoTests', 'WindowsSmoke', 'WindowsE2E')

$deletedChange = Get-LToolsBuildImpact ([ordered]@{ 'windows/ltools.ps1' = 'old' }) ([ordered]@{})
Assert-True ($deletedChange.Package -contains 'windows/ltools.ps1') 'La eliminación de una entrada de paquete no se detectó.'

$workspace = Join-Path ([IO.Path]::GetTempPath()) "ltools-build-state-test-$PID-$([guid]::NewGuid().ToString('N'))"
$output = Join-Path $workspace 'output'
$release = Join-Path $workspace 'release'
$dist = Join-Path $workspace 'dist'
$build = Join-Path $workspace 'build'
New-Item -ItemType Directory -Path $output, $release, $dist, $build -Force | Out-Null
try {
    $fingerprintA = Get-LToolsPublicKeyFingerprint $null "  AB CD`nEF  "
    $fingerprintEquivalent = Get-LToolsPublicKeyFingerprint $null 'abcdef'
    $fingerprintB = Get-LToolsPublicKeyFingerprint $null 'abcdef01'
    $sshFingerprintA = Get-LToolsPublicKeyFingerprint $null 'ssh-ed25519 AAAA same-comment'
    $sshFingerprintEquivalent = Get-LToolsPublicKeyFingerprint $null "  ssh-ed25519   AAAA other comment`n"
    $sshFingerprintB = Get-LToolsPublicKeyFingerprint $null 'ssh-ed25519 aAAA same-comment'
    Assert-True ($fingerprintA -eq $fingerprintEquivalent) 'La huella debe identificar la clave normalizada, ignorando espacios.'
    Assert-True ($fingerprintA -ne $fingerprintB) 'Cambiar la clave pública debe invalidar la identidad de firma.'
    Assert-True ($sshFingerprintA -eq $sshFingerprintEquivalent) 'Una huella SSH debe ignorar espacios y comentarios que no cambian la clave.'
    Assert-True ($sshFingerprintA -ne $sshFingerprintB) 'La codificación Base64 SSH distingue mayúsculas: claves diferentes no pueden colisionar al normalizar.'
    Assert-True ((Get-LToolsPublicKeyFingerprint $null 'OPAQUE') -ne (Get-LToolsPublicKeyFingerprint $null 'opaque')) 'Las claves no hexadecimales deben conservar su sensibilidad a mayúsculas.'
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
    $binary = Join-Path $build 'ltools.exe'
    $guiBinary = Join-Path $build 'ltools-gui.exe'
    $cliBinary = Join-Path $build 'ltools-cli.exe'
    Set-Content -LiteralPath $binary -Value 'known GUI binary' -NoNewline
    Set-Content -LiteralPath $guiBinary -Value 'known GUI binary' -NoNewline
    Set-Content -LiteralPath $cliBinary -Value 'known CLI binary' -NoNewline
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
    $firstBuildHashes = Get-LToolsBuildBinaryHashes $binary $guiBinary $cliBinary
    Assert-True (Test-LToolsBuildBinariesMatch $firstBuildHashes $firstBuildHashes) 'Los binarios compilados intactos no se reconocieron como actuales.'
    Assert-True (Test-LToolsBuildBinariesUntrusted $null $firstBuildHashes) 'La primera línea base debe limpiar binarios previos sin hashes confiables.'
    Assert-True (-not (Test-LToolsBuildBinariesUntrusted $null ([ordered]@{ gui = 'missing'; guiCopy = 'missing'; cli = 'missing' }))) 'Un target vacío no debe activar una limpieza redundante.'
    Assert-True (-not (Test-LToolsBuildBinariesUntrusted $firstBuildHashes $firstBuildHashes)) 'Una línea base existente no debe marcarse como no confiable.'
    Assert-True (Test-LToolsBuildBinariesUntrusted ([ordered]@{ gui = $firstBuildHashes.gui; cli = $firstBuildHashes.cli }) $firstBuildHashes) 'Un estado parcialmente migrado debe limpiar el target antes de volver a confiar en él.'
    Set-Content -LiteralPath $binary -Value 'tampered GUI binary' -NoNewline
    $tamperedBuildHashes = Get-LToolsBuildBinaryHashes $binary $guiBinary $cliBinary
    Assert-True (-not (Test-LToolsBuildBinariesMatch $firstBuildHashes $tamperedBuildHashes)) 'No se detectó un binario compilado alterado que debe forzar recompilación.'
    Assert-True (Test-LToolsBuildBinariesTampered $firstBuildHashes $tamperedBuildHashes) 'La alteración no se marcó para limpiar el target antes de invocar Cargo.'
    Set-Content -LiteralPath $binary -Value 'known GUI binary' -NoNewline
    $restoredBuildHashes = Get-LToolsBuildBinaryHashes $binary $guiBinary $cliBinary
    Assert-True (Test-LToolsBuildBinariesMatch $firstBuildHashes $restoredBuildHashes) 'Restaurar el binario compilado debe recuperar su hash esperado.'
    Assert-True (-not (Test-LToolsBuildBinariesTampered $firstBuildHashes $restoredBuildHashes)) 'Un binario restaurado no debe forzar una limpieza del target.'
    Assert-True ($firstHashes['output/THIRD-PARTY-LICENSES-windows.zip'] -ne 'missing' -and
        $firstHashes['portable/THIRD-PARTY-LICENSES/INDEX.txt'] -ne 'missing' -and
        $firstHashes['release/THIRD-PARTY-LICENSES-windows.zip'] -ne 'missing') 'No se incluyeron las licencias en las huellas del estado incremental.'
    $signing = [ordered]@{ required = $false; publicKeyFingerprint = $null; signed = $false; sshPublicKeyFingerprint = 'ssh-fingerprint'; sshIdentity = 'test@example.invalid'; sshSigned = $true }
    Assert-True (Test-LToolsBuildArtifactsMatch $firstHashes $firstHashes $signing $false $null 'ssh-fingerprint' 'test@example.invalid') 'Artefactos idénticos no se reconocieron como actuales.'

    Set-Content -LiteralPath $artifact -Value 'tampered release' -NoNewline
    $tamperedHashes = Get-LToolsBuildArtifactHashes $output $release $dist '1.2.3' 'x86_64'
    Assert-True (-not (Test-LToolsBuildArtifactsMatch $firstHashes $tamperedHashes $signing $false $null 'ssh-fingerprint' 'test@example.invalid')) 'No se detectó un artefacto alterado.'
    Assert-True (Test-LToolsBuildBinariesMatch $firstBuildHashes (Get-LToolsBuildBinaryHashes $binary $guiBinary $cliBinary)) 'Alterar solo un artefacto publicado no debe invalidar los binarios compilados.'
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
