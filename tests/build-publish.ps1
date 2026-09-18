$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $root 'scripts\lib\publish.ps1')
. (Join-Path $root 'scripts\lib\ssh-signing.ps1')

$sshKeygen = Get-Command ssh-keygen.exe, ssh-keygen -ErrorAction SilentlyContinue | Select-Object -First 1
if ($sshKeygen) {
    $savedSshEnvironment = @{}
    foreach ($name in @(
        'LTOOLS_SSH_SIGNING_KEY_FILE', 'LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE', 'LTOOLS_SSH_SIGNING_IDENTITY',
        'LTERMINAL_SSH_SIGNING_KEY_FILE', 'LTERMINAL_SSH_SIGNING_PUBLIC_KEY_FILE', 'LTERMINAL_SSH_SIGNING_IDENTITY'
    )) { $savedSshEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
    $sshTestDirectory = Join-Path ([IO.Path]::GetTempPath()) "ltools-powershell-ssh-test-$PID-$([guid]::NewGuid().ToString('N'))"
    New-Item -ItemType Directory -Path $sshTestDirectory | Out-Null
    try {
        $sshPrivateKey = Join-Path $sshTestDirectory 'isolated-key'
        & $sshKeygen.Source -q -t ed25519 -N '' -C 'ltools-test@example.invalid' -f $sshPrivateKey
        if ($LASTEXITCODE -ne 0) { throw 'No se pudo crear una clave SSH temporal para la prueba PowerShell.' }
        $env:LTOOLS_SSH_SIGNING_KEY_FILE = $sshPrivateKey
        $env:LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE = "$sshPrivateKey.pub"
        $env:LTOOLS_SSH_SIGNING_IDENTITY = 'ltools-test@example.invalid'
        $sshConfiguration = Get-LToolsSshSigningConfiguration $root
        if (-not $sshConfiguration.CanSign -or -not $sshConfiguration.CanVerify) {
            throw 'La configuración de firma SSH aislada no detectó ambas capacidades.'
        }
        $sshManifest = Join-Path $sshTestDirectory 'SHA256SUMS.txt'
        $sshSignature = Join-Path $sshTestDirectory 'SHA256SUMS.txt.sshsig'
        [IO.File]::WriteAllText($sshManifest, "0123456789abcdef package.zip" + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
        Invoke-LToolsSshManifestSigning $sshManifest $sshSignature $sshConfiguration | Out-Null
        Test-LToolsSshManifestSignature $sshManifest $sshSignature $sshConfiguration
        [IO.File]::AppendAllText($sshManifest, "tampered" + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
        $tamperingRejected = $false
        try { Test-LToolsSshManifestSignature $sshManifest $sshSignature $sshConfiguration }
        catch { $tamperingRejected = $true }
        if (-not $tamperingRejected) { throw 'La verificación SSH PowerShell aceptó un manifiesto alterado.' }
    } finally {
        foreach ($name in $savedSshEnvironment.Keys) {
            [Environment]::SetEnvironmentVariable($name, $savedSshEnvironment[$name], 'Process')
        }
        if (Test-Path -LiteralPath $sshTestDirectory -PathType Container) {
            Remove-Item -LiteralPath $sshTestDirectory -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

$workspace = Join-Path ([IO.Path]::GetTempPath()) "ltools-publish-test-$PID-$([guid]::NewGuid().ToString('N'))"
$output = Join-Path $workspace 'output'
$staging = Join-Path $workspace 'staging'
New-Item -ItemType Directory -Path $output, $staging -Force | Out-Null
try {
    $fileTarget = Join-Path $output 'artifact.zip'
    $fileSource = Join-Path $staging 'artifact.zip'
    Set-Content -LiteralPath $fileTarget -Value 'old artifact' -NoNewline
    Set-Content -LiteralPath $fileSource -Value 'new artifact' -NoNewline
    Publish-StagedFile $fileSource $fileTarget
    if ((Get-Content -Raw -LiteralPath $fileTarget) -ne 'new artifact' -or
        (Test-Path -LiteralPath $fileSource)) {
        throw 'La promoción atómica del archivo no reemplazó exactamente el destino.'
    }

    $jsonPath = Join-Path $output 'machine.json'
    Write-Utf8NoBom $jsonPath '{"schema":"test-v1"}'
    $jsonBytes = [IO.File]::ReadAllBytes($jsonPath)
    if ($jsonBytes.Length -lt 3 -or
        ($jsonBytes[0] -eq 0xEF -and $jsonBytes[1] -eq 0xBB -and $jsonBytes[2] -eq 0xBF)) {
        throw 'El JSON para distribución incluye BOM UTF-8.'
    }

    $directoryTarget = Join-Path $output 'portable'
    $directorySource = Join-Path $staging 'portable'
    New-Item -ItemType Directory -Path $directoryTarget, $directorySource -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $directoryTarget 'old.txt') -Value 'old' -NoNewline
    Set-Content -LiteralPath (Join-Path $directorySource 'new.txt') -Value 'new' -NoNewline
    Publish-StagedDirectory $directorySource $directoryTarget
    if (-not (Test-Path -LiteralPath (Join-Path $directoryTarget 'new.txt')) -or
        (Test-Path -LiteralPath (Join-Path $directoryTarget 'old.txt')) -or
        (Test-Path -LiteralPath $directorySource)) {
        throw 'La promoción de la carpeta portable dejó un estado incorrecto.'
    }

    $releaseTarget = Join-Path $output 'release'
    New-Item -ItemType Directory -Path (Join-Path $releaseTarget 'user-data') -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $releaseTarget 'user-data\keep.txt') -Value 'keep' -NoNewline
    Set-Content -LiteralPath (Join-Path $releaseTarget '.user-note') -Value 'hidden' -NoNewline
    $releaseStage = New-StagedDirectory $releaseTarget
    if ((Get-Content -Raw -LiteralPath (Join-Path $releaseTarget '.user-note')) -ne 'hidden' -or
        (Get-Content -Raw -LiteralPath (Join-Path $releaseStage 'user-data\keep.txt')) -ne 'keep') {
        throw 'El staging de release no conserva datos previos o altera el destino durante la preparación.'
    }
    Set-Content -LiteralPath (Join-Path $releaseStage 'new-release.txt') -Value 'ready' -NoNewline
    Publish-StagedDirectory $releaseStage $releaseTarget
    if ((Get-Content -Raw -LiteralPath (Join-Path $releaseTarget '.user-note')) -ne 'hidden' -or
        -not (Test-Path -LiteralPath (Join-Path $releaseTarget 'new-release.txt'))) {
        throw 'El intercambio de release no preservó el contenido ajeno y el nuevo paquete.'
    }

    $releaseFixture = Join-Path $output 'release-e2e-fixture'
    New-Item -ItemType Directory -Path $releaseFixture -Force | Out-Null
    $fixtureVersion = '9.8.7'
    $fixtureNames = @(
        "ltools-$fixtureVersion-windows-x86_64.exe",
        "ltools-$fixtureVersion-windows-x86_64-cli.exe",
        "ltools-$fixtureVersion-windows-x86_64.zip"
    )
    foreach ($name in $fixtureNames) {
        [IO.File]::WriteAllBytes((Join-Path $releaseFixture $name), [byte[]]@(1, 2, 3, 4))
    }
    $licenseFixture = Join-Path $releaseFixture 'THIRD-PARTY-LICENSES'
    New-Item -ItemType Directory -Path $licenseFixture | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $releaseFixture
    $licenseIndexText = @(
        "ring$([char]9)0.17.14$([char]9)Apache-2.0 AND ISC",
        "webpki-roots$([char]9)1.0.9$([char]9)CDLA-Permissive-2.0"
    ) -join [Environment]::NewLine
    Write-Utf8NoBom (Join-Path $licenseFixture 'INDEX.txt') $licenseIndexText
    $licenseArchive = Join-Path $releaseFixture 'THIRD-PARTY-LICENSES-windows.zip'
    Compress-Archive -Path @((Join-Path $releaseFixture 'LICENSE'), $licenseFixture) -DestinationPath $licenseArchive
    $fixtureArtifacts = @()
    foreach ($index in 0..($fixtureNames.Count - 1)) {
        $name = $fixtureNames[$index]
        $file = Join-Path $releaseFixture $name
        $kind = @('exe', 'exe-cli', 'portable-zip')[$index]
        $fixtureArtifacts += [ordered]@{
            platform = 'windows'; architecture = 'x86_64'; kind = $kind; filename = $name
            download_url = "https://example.invalid/$name"; size_bytes = (Get-Item -LiteralPath $file).Length
            sha256 = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
            executable = ($index -lt 2)
        }
    }
    $fixtureActions = @(@{
        id = 'windows.fixture.check'; legacyId = 'fixture-check'; actionId = 'windows.fixture.check'; actionKey = 'fixture.check'; qualifiedActionKey = 'windows.fixture.check'; canonicalKey = 'windows.fixture.check'; scope = 'fixture'; operation = 'fixture-check'
        label = 'Fixture check'; shortLabel = 'Fixture'; displayName = 'Windows · Fixture · Fixture check'; menuPath = @('Fixture', 'Fixture check'); group = 'Fixture'; description = 'Acción de prueba'
        category = 'fixture'; command = 'fixture'; args = @(); target = 'none'; targetPolicy = 'none'
        invocation = @{ executable = 'ltools.exe'; args = @('actions', 'run', 'windows.fixture.check'); target = 'none' }
        mutating = $false; confirmation = 'none'; safe = $true; profile = 'safe-default'; aliases = @(); supports = @('dry-run','plan')
    })
    $fixtureLinuxActions = @($fixtureActions | ForEach-Object {
        $copy = $_.Clone()
        $copy.qualifiedActionKey = 'linux.fixture.check'
        $copy.canonicalKey = 'linux.fixture.check'
        $copy.actionId = 'linux.fixture.check'
        $copy.id = 'linux.fixture.check'
        $copy.displayName = 'Linux · Fixture · Fixture check'
        $copy.invocation = @{ executable = 'ltools'; args = @('actions', 'run', 'linux.fixture.check'); target = 'none' }
        $copy
    })
    $fixtureDescriptorActions = @($fixtureActions | ForEach-Object {
        $copy = $_.Clone()
        $copy.id = $copy.actionId
        $copy
    })
    $fixtureLinuxDescriptorActions = @($fixtureDescriptorActions | ForEach-Object {
        $copy = $_.Clone()
        $copy.actionId = 'linux.fixture.check'
        $copy.qualifiedActionKey = 'linux.fixture.check'
        $copy.canonicalKey = 'linux.fixture.check'
        $copy.id = $copy.actionId
        $copy.displayName = 'Linux · Fixture · Fixture check'
        $copy
    })
    $fixtureJson = @{
        # Una release combinada puede conservar LTools como identidad global
        # aunque el descriptor nativo Windows sea WTools; la E2E debe validar
        # cada descriptor por plataforma, no compararlo ciegamente con esto.
        'ltools-release.json' = [ordered]@{ schema = 'ltools-release-v1'; application = 'LTools'; version = $fixtureVersion; hash_algorithm = 'sha256'; artifacts = $fixtureArtifacts }
        'ltools-capabilities.json' = @{ schema = 'ltools-capabilities-v1'; application = 'LTools'; version = $fixtureVersion; platform = 'linux'; actions = $fixtureLinuxDescriptorActions }
        'ltools-capabilities-windows.json' = @{ schema = 'ltools-capabilities-v1'; application = 'WTools'; version = $fixtureVersion; platform = 'windows'; actions = $fixtureDescriptorActions }
        'ltools-actions.json' = @{ schema = 'ltools-actions-v1'; platform = 'linux'; safety = @{ excluded_defaults = @('/'); target_selection = 'explicit-only' }; actions = $fixtureLinuxActions }
        'ltools-actions-windows.json' = @{ schema = 'ltools-actions-v1'; platform = 'windows'; safety = @{ excluded_defaults = @('C:'); target_selection = 'explicit-only' }; actions = $fixtureActions }
        'ltools-terminal.json' = @{ schema = 'ltools-terminal-integration-v1'; application = 'LTools'; version = $fixtureVersion; platform = 'linux'; entrypoint = @{ command = 'ltools' }; integration = @{ optional = $true; standalone_releases_require_it = $false }; host = @{ id = 'lterminal'; family = 'lterminal'; product = 'LTerminal'; known_products = @('LTerminal','WTools') }; actions = $fixtureLinuxDescriptorActions }
        'ltools-terminal-windows.json' = @{ schema = 'ltools-terminal-integration-v1'; application = 'WTools'; version = $fixtureVersion; platform = 'windows'; entrypoint = @{ command = 'ltools.exe' }; integration = @{ optional = $true; standalone_releases_require_it = $false }; host = @{ id = 'wtools'; family = 'lterminal'; product = 'WTools'; known_products = @('LTerminal','WTools') }; actions = $fixtureDescriptorActions }
        'ltools-project.json' = @{ schema = 'ltools-project-v1'; repository = 'example/project'; action_catalog = @{ catalog = 'ltools-actions.json'; schema = 'ltools-actions.schema.json'; descriptors = @{ linux = 'ltools-actions.json'; windows = 'ltools-actions-windows.json' } }; platforms = @{ windows = @{ install = @{ entrypoint = 'ltools.exe' } } } }
        'ltools-capabilities.schema.json' = @{ '$id' = 'https://example.invalid/capabilities.json' }
        'ltools-actions.schema.json' = @{ '$id' = 'https://example.invalid/actions.json' }
        'ltools-terminal.schema.json' = @{ '$id' = 'https://example.invalid/terminal.json' }
        'ltools-project.schema.json' = @{ '$id' = 'https://example.invalid/project.json' }
        'ltools-release.schema.json' = @{ '$id' = 'https://example.invalid/release.json' }
    }
    foreach ($name in $fixtureJson.Keys) {
        $json = $fixtureJson[$name] | ConvertTo-Json -Depth 8 -Compress
        Write-Utf8NoBom (Join-Path $releaseFixture $name) $json
    }
    Set-Content -LiteralPath (Join-Path $releaseFixture 'build-fragment.tmp') -Value 'ignored temp' -NoNewline
    Set-Content -LiteralPath (Join-Path $releaseFixture 'previous.bak') -Value 'ignored backup' -NoNewline
    Set-Content -LiteralPath (Join-Path $releaseFixture 'case-sensitive.TMP') -Value 'included uppercase extension' -NoNewline
    $checksumLines = @(Get-ChildItem -LiteralPath $releaseFixture -File |
        Where-Object { $_.Name -cnotlike '*.tmp' -and $_.Name -cnotlike '*.bak' } | Sort-Object Name | ForEach-Object {
        "$( (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() )  $($_.Name)"
    })
    [IO.File]::WriteAllLines((Join-Path $releaseFixture 'SHA256SUMS.txt'), $checksumLines, [Text.UTF8Encoding]::new($false))
    $releaseE2E = Join-Path $root 'windows\tests\release-e2e.ps1'
    & $releaseE2E -ReleaseDirectory $releaseFixture -Version $fixtureVersion -Architecture 'x86_64' -Binary 'unused-because-fixture-is-unsigned.exe' | Out-Null
    Add-Content -LiteralPath (Join-Path $releaseFixture $fixtureNames[0]) -Value 'tampered' -NoNewline
    $tamperRejected = $false
    try { & $releaseE2E -ReleaseDirectory $releaseFixture -Version $fixtureVersion -Architecture 'x86_64' -Binary 'unused-because-fixture-is-unsigned.exe' | Out-Null } catch { $tamperRejected = $true }
    if (-not $tamperRejected) { throw 'La E2E de release no detectó un binario alterado después del manifiesto/checksum.' }

    $blockedSource = Join-Path $staging 'blocked'
    New-Item -ItemType Directory -Path $blockedSource -Force | Out-Null
    $blockedDestination = Join-Path $output 'not-a-directory'
    Set-Content -LiteralPath $blockedDestination -Value 'preserve' -NoNewline
    $rejected = $false
    try { Publish-StagedDirectory $blockedSource $blockedDestination } catch { $rejected = $true }
    if (-not $rejected -or (Get-Content -Raw -LiteralPath $blockedDestination) -ne 'preserve') {
        throw 'El publicador no rechazó el tipo de destino incorrecto de forma segura.'
    }

    Write-Output 'Promoción de archivos/carpetas y firma SSH PowerShell validadas; destinos previos conservados ante error.'
} finally {
    Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
}
