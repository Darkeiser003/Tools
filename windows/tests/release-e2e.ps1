[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$ReleaseDirectory,
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Architecture,
    [Parameter(Mandatory = $true)][string]$Binary,
    [switch]$RequireSignature,
    [string]$PublicKeyFile,
    [switch]$RequireSshSignature,
    [string]$SshPublicKeyFile,
    [string]$SshIdentity
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-process.ps1')
. (Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) 'scripts\lib\ssh-signing.ps1')

function Assert-Release([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw "RELEASE E2E ERROR: $Message" }
}

$ReleaseDirectory = [IO.Path]::GetFullPath($ReleaseDirectory)
Assert-Release (Test-Path -LiteralPath $ReleaseDirectory -PathType Container) "No existe la carpeta release: $ReleaseDirectory"
Assert-Release ($Version -match '^\d+\.\d+\.\d+([.-][0-9A-Za-z.-]+)?$') "Versión inválida: $Version"
Assert-Release ($Architecture -match '^(x86_64|arm64|x86)$') "Arquitectura inválida: $Architecture"
$projectLicense = Join-Path $ReleaseDirectory 'LICENSE'
Assert-Release ((Test-Path -LiteralPath $projectLicense -PathType Leaf) -and
    (Select-String -LiteralPath $projectLicense -SimpleMatch 'MIT License' -Quiet) -and
    (Select-String -LiteralPath $projectLicense -SimpleMatch 'Darkeiser003' -Quiet) -and
    (Select-String -LiteralPath $projectLicense -SimpleMatch 'https://github.com/Darkeiser003' -Quiet)) 'Falta la licencia MIT del proyecto o no identifica correctamente al titular.'

$requiredArtifacts = @(
    "ltools-$Version-windows-$Architecture.exe",
    "ltools-$Version-windows-$Architecture-cli.exe",
    "ltools-$Version-windows-$Architecture.zip"
)
foreach ($name in $requiredArtifacts) {
    $path = Join-Path $ReleaseDirectory $name
    Assert-Release ((Test-Path -LiteralPath $path -PathType Leaf) -and (Get-Item -LiteralPath $path).Length -gt 0) "Falta o está vacío el artefacto requerido: $name"
}
$licenseArchive = Join-Path $ReleaseDirectory 'THIRD-PARTY-LICENSES-windows.zip'
Assert-Release ((Test-Path -LiteralPath $licenseArchive -PathType Leaf) -and (Get-Item -LiteralPath $licenseArchive).Length -gt 0) 'Falta el ZIP independiente de licencias Windows.'
$licenseCheckDir = Join-Path ([IO.Path]::GetTempPath()) ("ltools-release-licenses-" + [guid]::NewGuid().ToString('N'))
try {
    Expand-Archive -LiteralPath $licenseArchive -DestinationPath $licenseCheckDir
    $licenseIndex = Join-Path $licenseCheckDir 'THIRD-PARTY-LICENSES\INDEX.txt'
    $bundledProjectLicense = Join-Path $licenseCheckDir 'LICENSE'
    Assert-Release (Test-Path -LiteralPath $licenseIndex -PathType Leaf) 'El ZIP independiente omite el índice de licencias.'
    Assert-Release ((Test-Path -LiteralPath $bundledProjectLicense -PathType Leaf) -and
        (Select-String -LiteralPath $bundledProjectLicense -SimpleMatch 'MIT License' -Quiet) -and
        (Get-FileHash -LiteralPath $bundledProjectLicense -Algorithm SHA256).Hash -eq
            (Get-FileHash -LiteralPath $projectLicense -Algorithm SHA256).Hash) 'El ZIP independiente omite la licencia MIT del proyecto o contiene otra versión.'
    $licenseText = Get-Content -Raw -LiteralPath $licenseIndex
    Assert-Release ($licenseText.Contains('ISC') -and $licenseText.Contains('CDLA-Permissive-2.0')) 'El ZIP independiente omite avisos ISC/CDLA.'
} finally {
    if (Test-Path -LiteralPath $licenseCheckDir -PathType Container) {
        Remove-Item -LiteralPath $licenseCheckDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

$requiredJson = @(
    'ltools-release.json', 'ltools-capabilities.json', 'ltools-capabilities-windows.json',
    'ltools-actions.json', 'ltools-actions-windows.json', 'ltools-terminal.json', 'ltools-terminal-windows.json', 'ltools-project.json',
    'ltools-capabilities.schema.json', 'ltools-actions.schema.json', 'ltools-terminal.schema.json',
    'ltools-project.schema.json', 'ltools-release.schema.json'
)
$jsonValues = @{}
foreach ($name in $requiredJson) {
    $path = Join-Path $ReleaseDirectory $name
    Assert-Release (Test-Path -LiteralPath $path -PathType Leaf) "Falta el descriptor/esquema $name"
    try { $parsed = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json } catch { throw "RELEASE E2E ERROR: JSON inválido en $name`: $($_.Exception.Message)" }
    $jsonValues[$name] = $parsed
    if ($name -eq 'ltools-release.json') { $manifest = $parsed }
}
Assert-Release ($manifest.schema -eq 'ltools-release-v1' -and
    $manifest.application -in @('LTools', 'WTools') -and
    $manifest.version -eq $Version -and
    $manifest.hash_algorithm -eq 'sha256' -and @($manifest.artifacts).Count -gt 0) 'El manifiesto no cumple el contrato de release.'
foreach ($name in @('ltools-capabilities.schema.json', 'ltools-actions.schema.json', 'ltools-terminal.schema.json', 'ltools-project.schema.json', 'ltools-release.schema.json')) {
    Assert-Release (-not [string]::IsNullOrWhiteSpace([string]$jsonValues[$name].'$id')) "El esquema $name no declara un identificador."
}
foreach ($name in @('ltools-capabilities.json', 'ltools-actions.json', 'ltools-actions-windows.json', 'ltools-terminal.json', 'ltools-capabilities-windows.json', 'ltools-terminal-windows.json')) {
    if (-not $jsonValues.ContainsKey($name)) { continue }
    $entries = @($jsonValues[$name].actions)
    Assert-Release ($entries.Count -gt 0) "$name no publica acciones."
    $keys = @($entries | ForEach-Object { [string]$_.actionKey })
    $isBackendCatalog = $name -in @('ltools-actions.json', 'ltools-actions-windows.json')
    $qualifiedKeys = if ($isBackendCatalog) { @($entries | ForEach-Object { [string]$_.qualifiedActionKey }) } else { @() }
    $operations = @($entries | ForEach-Object { [string]$_.operation })
    $ids = @($entries | ForEach-Object { [string]$_.id })
    $actionIds = @($entries | ForEach-Object { [string]$_.actionId })
    $labels = @($entries | ForEach-Object { [string]$_.label })
    $shortLabels = @($entries | ForEach-Object { [string]$_.shortLabel })
    $displayNames = @($entries | ForEach-Object { [string]$_.displayName })
    Assert-Release (($ids | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
        (@($ids | Sort-Object -Unique).Count -eq $ids.Count)) "$name contiene id vacíos o duplicados."
    Assert-Release (($actionIds | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
        (@($actionIds | Sort-Object -Unique).Count -eq $actionIds.Count)) "$name contiene actionId vacíos o duplicados."
    Assert-Release (($keys | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
        (@($keys | Sort-Object -Unique).Count -eq $keys.Count)) "$name contiene actionKey vacíos o duplicados."
    if ($isBackendCatalog) {
        Assert-Release (($qualifiedKeys | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
            (@($qualifiedKeys | Sort-Object -Unique).Count -eq $qualifiedKeys.Count)) "$name contiene qualifiedActionKey vacíos o duplicados."
    }
    Assert-Release (($labels | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
        (@($labels | Sort-Object -Unique).Count -eq $labels.Count)) "$name contiene labels vacíos o duplicados."
    Assert-Release (($shortLabels | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
        (@($shortLabels | Sort-Object -Unique).Count -eq $shortLabels.Count)) "$name contiene shortLabel vacíos o duplicados."
    Assert-Release (($operations | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
        (@($operations | Sort-Object -Unique).Count -eq $operations.Count)) "$name contiene operation vacíos o duplicados."
    if ($isBackendCatalog) {
        $canonicalKeys = @($entries | ForEach-Object { [string]$_.canonicalKey })
        Assert-Release (($canonicalKeys | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
            (@($canonicalKeys | Sort-Object -Unique).Count -eq $canonicalKeys.Count)) "$name contiene canonicalKey vacíos o duplicados."
        Assert-Release (($displayNames | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0 -and
            (@($displayNames | Sort-Object -Unique).Count -eq $displayNames.Count)) "$name contiene displayName vacíos o duplicados."
    }
    foreach ($action in $entries) {
        $key = [string]$action.actionKey
        if ($isBackendCatalog) {
            Assert-Release ([string]$action.id -ceq [string]$action.actionId -and
                -not [string]::IsNullOrWhiteSpace([string]$action.legacyId)) "$name contiene una identidad estable o legacyId inconsistente."
            Assert-Release ([string]$action.qualifiedActionKey -ceq (([string]$jsonValues[$name].platform) + '.' + $key)) "$name contiene una qualifiedActionKey inconsistente."
            Assert-Release ([string]$action.actionId -ceq [string]$action.canonicalKey -and
                [string]$action.canonicalKey -ceq [string]$action.qualifiedActionKey -and
                @($action.menuPath).Count -eq 2 -and
                -not [string]::IsNullOrWhiteSpace([string]$action.displayName)) "$name contiene una identidad descriptiva incompleta."
            Assert-Release ([string]$action.invocation.executable -in @('ltools', 'ltools.exe') -and
                (@($action.invocation.args) -join '|') -ceq ('actions|run|' + [string]$action.actionId) -and
                [string]$action.invocation.target -ceq [string]$action.target) "$name contiene una invocación declarativa inconsistente."
        } else {
            Assert-Release ([string]$action.id -ceq [string]$action.actionId -and
                -not [string]::IsNullOrWhiteSpace([string]$action.legacyId) -and
                [string]$action.actionId -ceq [string]$action.canonicalKey -and
                [string]$action.canonicalKey -ceq [string]$action.qualifiedActionKey) "$name contiene una identidad declarativa incompleta."
        }
        Assert-Release ($key -match '^[a-z0-9]+(?:[.-][a-z0-9]+)+$' -and
            -not [string]::IsNullOrWhiteSpace([string]$action.scope) -and
            -not [string]::IsNullOrWhiteSpace([string]$action.operation) -and
            $key.Split('.')[0] -ceq [string]$action.scope -and
            [string]$action.operation -ceq ($key -replace '\.', '-') -and
            -not [string]::IsNullOrWhiteSpace([string]$action.group) -and
            -not [string]::IsNullOrWhiteSpace([string]$action.description) -and
            [string]$action.confirmation -in @('none', 'before-run', 'always', 'backend') -and
            ([bool]$action.safe -eq ([string]$action.confirmation -ceq 'none'))) "$name contiene una acción con identidad o política inconsistente."
    }
}
Assert-Release ($jsonValues['ltools-capabilities.json'].schema -eq 'ltools-capabilities-v1' -and
    $jsonValues['ltools-capabilities.json'].platform -in @('linux', 'windows') -and
    $jsonValues['ltools-capabilities.json'].version -eq $Version -and
    $jsonValues['ltools-capabilities.json'].application -eq
        $(if ($jsonValues['ltools-capabilities.json'].platform -eq 'windows') { 'WTools' } else { 'LTools' }) -and
    $jsonValues['ltools-capabilities-windows.json'].platform -eq 'windows' -and
    $jsonValues['ltools-capabilities-windows.json'].version -eq $Version -and
    $jsonValues['ltools-capabilities-windows.json'].application -eq 'WTools') 'El descriptor de capacidades Windows no coincide con su plataforma.'
Assert-Release ($jsonValues['ltools-terminal.json'].schema -eq 'ltools-terminal-integration-v1' -and
    $jsonValues['ltools-terminal.json'].platform -in @('linux', 'windows') -and
    $jsonValues['ltools-terminal.json'].version -eq $Version -and
    $jsonValues['ltools-terminal.json'].application -eq
        $(if ($jsonValues['ltools-terminal.json'].platform -eq 'windows') { 'WTools' } else { 'LTools' }) -and
    $jsonValues['ltools-terminal.json'].entrypoint.command -eq
        $(if ($jsonValues['ltools-terminal.json'].platform -eq 'windows') { 'ltools.exe' } else { 'ltools' }) -and
    $jsonValues['ltools-terminal.json'].integration.optional -eq $true -and
    $jsonValues['ltools-terminal.json'].integration.standalone_releases_require_it -eq $false -and
    $jsonValues['ltools-terminal.json'].host.product -eq
        $(if ($jsonValues['ltools-terminal.json'].platform -eq 'windows') { 'WTools' } else { 'LTerminal' }) -and
    $jsonValues['ltools-terminal.json'].host.id -eq
        $(if ($jsonValues['ltools-terminal.json'].platform -eq 'windows') { 'wtools' } else { 'lterminal' }) -and
    $jsonValues['ltools-terminal-windows.json'].version -eq $Version -and
    $jsonValues['ltools-terminal-windows.json'].application -eq 'WTools' -and
    $jsonValues['ltools-terminal-windows.json'].host.id -eq 'wtools') 'El descriptor de integración Windows no cumple su contrato.'

foreach ($pair in @(
        @('ltools-capabilities.json', 'ltools-capabilities-windows.json'),
        @('ltools-terminal.json', 'ltools-terminal-windows.json'),
        @('ltools-actions.json', 'ltools-actions-windows.json')
    )) {
    $linuxEntries = @($jsonValues[$pair[0]].actions)
    $windowsEntries = @($jsonValues[$pair[1]].actions)
    $canonicalKeys = @($linuxEntries + $windowsEntries | ForEach-Object { [string]$_.canonicalKey })
    $qualifiedKeys = @($linuxEntries + $windowsEntries | ForEach-Object { [string]$_.qualifiedActionKey })
    $displayNames = @($linuxEntries + $windowsEntries | ForEach-Object { [string]$_.displayName })
    Assert-Release (@($canonicalKeys | Sort-Object -Unique).Count -eq $canonicalKeys.Count -and
        @($qualifiedKeys | Sort-Object -Unique).Count -eq $qualifiedKeys.Count -and
        @($displayNames | Sort-Object -Unique).Count -eq $displayNames.Count) "$($pair[0]) y $($pair[1]) contienen identidades globales duplicadas."
}
Assert-Release ($jsonValues['ltools-project.json'].schema -eq 'ltools-project-v1' -and
    -not [string]::IsNullOrWhiteSpace([string]$jsonValues['ltools-project.json'].repository) -and
    $jsonValues['ltools-project.json'].action_catalog.catalog -eq 'ltools-actions.json' -and
    $jsonValues['ltools-project.json'].action_catalog.schema -eq 'ltools-actions.schema.json' -and
    $jsonValues['ltools-project.json'].action_catalog.descriptors.linux -eq 'ltools-actions.json' -and
    $jsonValues['ltools-project.json'].action_catalog.descriptors.windows -eq 'ltools-actions-windows.json' -and
    $jsonValues['ltools-project.json'].platforms.windows.install.entrypoint -eq 'ltools.exe') 'El descriptor del proyecto omite la instalación Windows.'

$manifestNames = @{}
foreach ($artifact in @($manifest.artifacts)) {
    $name = [string]$artifact.filename
    Assert-Release (-not [string]::IsNullOrWhiteSpace($name) -and
        [IO.Path]::GetFileName($name) -ceq $name -and $name -notmatch '[\\/]') "Nombre de artefacto inseguro en el manifiesto: $name"
    Assert-Release (-not $manifestNames.ContainsKey($name)) "Artefacto duplicado en el manifiesto: $name"
    $manifestNames[$name] = $true
    $path = Join-Path $ReleaseDirectory $name
    Assert-Release (Test-Path -LiteralPath $path -PathType Leaf) "El manifiesto refiere un archivo ausente: $name"
    $item = Get-Item -LiteralPath $path
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    Assert-Release ([int64]$artifact.size_bytes -eq $item.Length -and [string]$artifact.sha256 -eq $hash) "Tamaño/hash del manifiesto incorrecto: $name"
}
foreach ($name in $requiredArtifacts) {
    Assert-Release ($manifestNames.ContainsKey($name)) "El manifiesto omite $name"
}

$checksumPath = Join-Path $ReleaseDirectory 'SHA256SUMS.txt'
Assert-Release (Test-Path -LiteralPath $checksumPath -PathType Leaf) 'Falta SHA256SUMS.txt'
$checksums = @{}
foreach ($line in [IO.File]::ReadAllLines($checksumPath)) {
    if ($line -notmatch '^(?<hash>[A-Fa-f0-9]{64})  (?<name>[^/\\]+)$' -or $Matches.name -in @('.', '..')) {
        throw "RELEASE E2E ERROR: Línea insegura o malformada en SHA256SUMS.txt: $line"
    }
    $name = $Matches.name
    Assert-Release (-not $checksums.ContainsKey($name)) "SHA256SUMS.txt contiene duplicados: $name"
    $path = Join-Path $ReleaseDirectory $name
    Assert-Release (Test-Path -LiteralPath $path -PathType Leaf) "SHA256SUMS.txt refiere un archivo ausente: $name"
    $actual = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
    Assert-Release ($actual -ieq $Matches.hash) "SHA256SUMS.txt no coincide para $name"
    $checksums[$name] = $true
}
$expectedFiles = @(Get-ChildItem -LiteralPath $ReleaseDirectory -File -Force |
    Where-Object { $_.Name -notin @('SHA256SUMS.txt', 'SHA256SUMS.txt.sig', 'SHA256SUMS.txt.sshsig') -and
        $_.Name -cnotlike '*.tmp' -and $_.Name -cnotlike '*.bak' })
Assert-Release ($checksums.Count -eq $expectedFiles.Count) 'SHA256SUMS.txt no cubre exactamente todos los archivos planos de release.'
foreach ($file in $expectedFiles) {
    Assert-Release ($checksums.ContainsKey($file.Name)) "SHA256SUMS.txt omite $($file.Name)"
}

$signaturePath = Join-Path $ReleaseDirectory 'SHA256SUMS.txt.sig'
if ($RequireSignature) { Assert-Release (Test-Path -LiteralPath $signaturePath -PathType Leaf) 'Se requiere firma Ed25519, pero falta SHA256SUMS.txt.sig.' }
if (Test-Path -LiteralPath $signaturePath -PathType Leaf) {
    $hasPublicKey = ($PublicKeyFile -and (Test-Path -LiteralPath $PublicKeyFile -PathType Leaf)) -or
        $env:LTOOLS_UPDATE_PUBLIC_KEY -or $env:LTERMINAL_UPDATE_PUBLIC_KEY
    Assert-Release ([bool]$hasPublicKey) 'Hay una firma, pero no se ha configurado una clave pública de verificación.'
    $arguments = @('release-signature', '--manifest', $checksumPath, '--signature', $signaturePath)
    if ($PublicKeyFile -and (Test-Path -LiteralPath $PublicKeyFile -PathType Leaf)) { $arguments += @('--public-key-file', $PublicKeyFile) }
    $verification = Invoke-NativeProcess -FileName $Binary -Arguments ($arguments + @('--verify')) -TimeoutSeconds 60
    Assert-Release ($verification.ExitCode -eq 0) (Format-NativeProcessFailure $verification 'Firma Ed25519 de release')
}
$sshSignaturePath = Join-Path $ReleaseDirectory 'SHA256SUMS.txt.sshsig'
if ($RequireSshSignature) { Assert-Release (Test-Path -LiteralPath $sshSignaturePath -PathType Leaf) 'Se requiere firma OpenSSH, pero falta SHA256SUMS.txt.sshsig.' }
if (Test-Path -LiteralPath $sshSignaturePath -PathType Leaf) {
    if ($SshPublicKeyFile) { $env:LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE = $SshPublicKeyFile }
    if ($SshIdentity) { $env:LTOOLS_SSH_SIGNING_IDENTITY = $SshIdentity }
    $root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    $sshConfiguration = Get-LToolsSshSigningConfiguration $root
    try { Test-LToolsSshManifestSignature $checksumPath $sshSignaturePath $sshConfiguration }
    catch { throw "RELEASE E2E ERROR: Firma OpenSSH inválida: $($_.Exception.Message)" }
}

Write-Output "E2E de release Windows correcto: $ReleaseDirectory"
