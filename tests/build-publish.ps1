$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $root 'scripts\lib\publish.ps1')

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
    $fixtureJson = @{
        'ltools-release.json' = [ordered]@{ schema = 'ltools-release-v1'; application = 'WinSlim-Tools'; version = $fixtureVersion; hash_algorithm = 'sha256'; artifacts = $fixtureArtifacts }
        'ltools-capabilities.json' = @{ schema = 'ltools-capabilities-v1'; application = 'WinSlim-Tools'; version = $fixtureVersion; platform = 'windows' }
        'ltools-capabilities-windows.json' = @{ schema = 'ltools-capabilities-v1'; application = 'WinSlim-Tools'; version = $fixtureVersion; platform = 'windows' }
        'ltools-terminal.json' = @{ schema = 'ltools-terminal-integration-v1'; application = 'WinSlim-Tools'; version = $fixtureVersion; platform = 'windows'; entrypoint = @{ command = 'ltools.exe' }; integration = @{ optional = $true; standalone_releases_require_it = $false }; host = @{ product = 'WinSlim Terminal' } }
        'ltools-terminal-windows.json' = @{ schema = 'ltools-terminal-integration-v1'; application = 'WinSlim-Tools'; version = $fixtureVersion; platform = 'windows'; entrypoint = @{ command = 'ltools.exe' }; integration = @{ optional = $true; standalone_releases_require_it = $false }; host = @{ product = 'WinSlim Terminal' } }
        'ltools-project.json' = @{ schema = 'ltools-project-v1'; repository = 'example/project'; platforms = @{ windows = @{ install = @{ entrypoint = 'ltools.exe' } } } }
        'ltools-capabilities.schema.json' = @{ '$id' = 'https://example.invalid/capabilities.json' }
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

    Write-Output 'Promoción de archivos/carpetas validada; destinos previos conservados ante error.'
} finally {
    Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
}
