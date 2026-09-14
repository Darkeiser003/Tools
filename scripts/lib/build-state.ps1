function Get-LToolsMapValue($Map, [string]$Key) {
    if ($null -eq $Map) { return $null }
    if ($Map -is [System.Collections.IDictionary]) {
        if ($Map.Contains($Key)) { return $Map[$Key] }
        return $null
    }
    $property = $Map.PSObject.Properties[$Key]
    if ($property) { return $property.Value }
    return $null
}

function Get-LToolsMapKeys($Map) {
    if ($null -eq $Map) { return @() }
    if ($Map -is [System.Collections.IDictionary]) { return @($Map.Keys) }
    return @($Map.PSObject.Properties | ForEach-Object Name)
}

function Get-LToolsBuildImpact($Old, $New) {
    $changed = [System.Collections.Generic.List[string]]::new()
    foreach ($key in (Get-LToolsMapKeys $New)) {
        if ($null -eq $Old -or (Get-LToolsMapValue $Old $key) -ne (Get-LToolsMapValue $New $key)) {
            $changed.Add([string]$key)
        }
    }
    foreach ($key in (Get-LToolsMapKeys $Old)) {
        if ($null -eq (Get-LToolsMapValue $New $key)) { $changed.Add([string]$key) }
    }

    $impact = [ordered]@{
        Changed = @($changed | Sort-Object -Unique)
        RustCompile = [System.Collections.Generic.List[string]]::new()
        Package = [System.Collections.Generic.List[string]]::new()
        CargoTests = [System.Collections.Generic.List[string]]::new()
        WindowsSmoke = [System.Collections.Generic.List[string]]::new()
        WindowsE2E = [System.Collections.Generic.List[string]]::new()
        BuildStateTests = [System.Collections.Generic.List[string]]::new()
        PublishTests = [System.Collections.Generic.List[string]]::new()
        ReleaseTests = [System.Collections.Generic.List[string]]::new()
    }

    foreach ($path in $impact.Changed) {
        $rustBuild = $path -match '^rust/(src/|Cargo\.(toml|lock)$|build\.rs$|rust-toolchain(\.toml)?$|\.cargo/)'
        $rustProduct = $path -match '^rust/src/' -or $path -match '^rust/(Cargo\.(toml|lock)$|build\.rs$|rust-toolchain(\.toml)?$|\.cargo/)'
        $windowsProduct = $path -match '^windows/(?!tests/)'
        $packageInput = $rustProduct -or $windowsProduct -or $path -eq 'windows/tests/release-e2e.ps1' -or
            $path -match '^scripts/build\.ps1$' -or
            $path -match '^scripts/lib/publish\.ps1$' -or
            $path -match '^scripts/lib/third-party-licenses\.ps1$' -or
            $path -match '^scripts/lib/ssh-signing\.ps1$' -or
            $path -match '^appimage/ltools-(capabilities|terminal)\.schema\.json$' -or
            $path -match '^distribution/' -or $path -in @('README.md', 'LICENSE')

        if ($rustBuild) { $impact.RustCompile.Add($path) }
        if ($packageInput) { $impact.Package.Add($path) }
        if ($path -match '^rust/(src/|tests/|Cargo\.(toml|lock)$|build\.rs$|rust-toolchain(\.toml)?$|\.cargo/)') {
            $impact.CargoTests.Add($path)
        }
        if ($rustProduct -or $windowsProduct -or $path -eq 'windows/tests/smoke.ps1') {
            $impact.WindowsSmoke.Add($path)
        }
        if ($rustProduct -or $windowsProduct -or $path -eq 'windows/tests/e2e.ps1') {
            $impact.WindowsE2E.Add($path)
        }
        if ($path -match '^scripts/lib/publish\.ps1$' -or $path -eq 'tests/build-publish.ps1') {
            $impact.PublishTests.Add($path)
        }
        if ($path -match '^scripts/build\.ps1$' -or $path -match '^scripts/lib/build-state\.ps1$' -or $path -eq 'tests/build-state.ps1') {
            $impact.BuildStateTests.Add($path)
        }
        if ($path -eq 'windows/tests/release-e2e.ps1') {
            $impact.ReleaseTests.Add($path)
        }
    }

    foreach ($name in @('Changed', 'RustCompile', 'Package', 'CargoTests', 'WindowsSmoke', 'WindowsE2E', 'BuildStateTests', 'PublishTests', 'ReleaseTests')) {
        $impact[$name] = @($impact[$name] | Sort-Object -Unique)
    }
    return [pscustomobject]$impact
}

function Get-LToolsFileHash([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { return 'missing' }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-LToolsBuildArtifactHashes(
    [string]$OutputDirectory,
    [string]$PublishDirectory,
    [string]$DistDirectory,
    [string]$Version,
    [string]$Architecture
) {
    $prefix = "ltools-$Version-windows-$Architecture"
    $paths = [ordered]@{}
    foreach ($name in @(
        "$prefix.exe", "$prefix-cli.exe", "$prefix.zip", 'THIRD-PARTY-LICENSES-windows.zip',
        'ltools-capabilities.json', 'ltools-capabilities-windows.json',
        'ltools-terminal.json', 'ltools-terminal-windows.json',
        'ltools-capabilities.schema.json', 'ltools-terminal.schema.json'
    )) {
        $paths["output/$name"] = Join-Path $OutputDirectory $name
    }
    $portableNames = @(
        'ltools.exe', 'ltools-cli.exe', 'ltools.ps1', 'ltools.cmd',
        'ltools-cli.ps1', 'ltools-cli.cmd', 'README.md', 'LICENSE', 'ltools-capabilities.json',
        'ltools-capabilities-windows.json', 'ltools-terminal.json',
        'ltools-terminal-windows.json', 'ltools-capabilities.schema.json',
        'ltools-terminal.schema.json', 'BUILD-INFO.txt', 'THIRD-PARTY-LICENSES/INDEX.txt'
    )
    $portable = Join-Path $OutputDirectory $prefix
    foreach ($name in $portableNames) { $paths["portable/$name"] = Join-Path $portable $name }
    $portableLicenses = Join-Path $portable 'THIRD-PARTY-LICENSES'
    if (Test-Path -LiteralPath $portableLicenses -PathType Container) {
        foreach ($licenseFile in Get-ChildItem -LiteralPath $portableLicenses -Recurse -File -Force | Sort-Object FullName) {
            $relative = $licenseFile.FullName.Substring($portableLicenses.Length).TrimStart([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
            $paths["portable/THIRD-PARTY-LICENSES/$relative"] = $licenseFile.FullName
        }
    }

    foreach ($name in @(
        "$prefix.exe", "$prefix-cli.exe", "$prefix.zip", 'THIRD-PARTY-LICENSES-windows.zip',
        'ltools-capabilities.json', 'ltools-capabilities-windows.json',
        'ltools-terminal.json', 'ltools-terminal-windows.json',
        'ltools-capabilities.schema.json', 'ltools-terminal.schema.json',
        'ltools-project.json', 'ltools-project.schema.json', 'ltools-release.schema.json',
        'ltools-release.json', 'LICENSE', 'SHA256SUMS.txt', 'SHA256SUMS.txt.sig', 'SHA256SUMS.txt.sshsig'
    )) {
        $paths["release/$name"] = Join-Path $PublishDirectory $name
    }
    foreach ($name in @(
        'ltools-release.json', 'ltools-project.json', 'ltools-project.schema.json',
        'ltools-release.schema.json', 'LICENSE', 'SHA256SUMS.txt', 'SHA256SUMS.txt.sig', 'SHA256SUMS.txt.sshsig'
    )) {
        $paths["dist/$name"] = Join-Path $DistDirectory $name
    }

    $hashes = [ordered]@{}
    foreach ($key in $paths.Keys) { $hashes[$key] = Get-LToolsFileHash $paths[$key] }
    return $hashes
}

function Get-LToolsPublicKeyFingerprint(
    [string]$PublicKeyFile,
    [string]$PublicKeyEnvironment
) {
    $keyText = $null
    if (-not [string]::IsNullOrWhiteSpace($PublicKeyEnvironment)) {
        $keyText = $PublicKeyEnvironment
    } elseif ($PublicKeyFile -and (Test-Path -LiteralPath $PublicKeyFile -PathType Leaf)) {
        $keyText = [IO.File]::ReadAllText($PublicKeyFile)
    }
    if ($null -eq $keyText) { return $null }
    $canonicalKey = ($keyText -replace '\s', '').ToLowerInvariant()
    $bytes = [Text.Encoding]::UTF8.GetBytes($canonicalKey)
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($bytes))).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Test-LToolsSigningKeyChanged($OldSigning, [string]$CurrentFingerprint) {
    return (Get-LToolsMapValue $OldSigning 'publicKeyFingerprint') -ne $CurrentFingerprint
}

function Test-LToolsBuildArtifactsMatch(
    $PreviousHashes,
    $CurrentHashes,
    $PreviousSigning,
    [bool]$SigningRequired,
    [string]$CurrentKeyFingerprint,
    [string]$CurrentSshKeyFingerprint,
    [string]$CurrentSshIdentity
) {
    if ($null -eq $PreviousHashes -or $null -eq $PreviousSigning) { return $false }
    if ((Get-LToolsMapValue $PreviousSigning 'required') -ne $SigningRequired) { return $false }
    $currentSignatureHash = Get-LToolsMapValue $CurrentHashes 'release/SHA256SUMS.txt.sig'
    $currentSigned = $currentSignatureHash -and $currentSignatureHash -ne 'missing'
    if ((Get-LToolsMapValue $PreviousSigning 'signed') -ne [bool]$currentSigned) { return $false }
    if ([string](Get-LToolsMapValue $PreviousSigning 'publicKeyFingerprint') -ne [string]$CurrentKeyFingerprint) { return $false }
    $currentSshSignatureHash = Get-LToolsMapValue $CurrentHashes 'release/SHA256SUMS.txt.sshsig'
    $currentSshSigned = $currentSshSignatureHash -and $currentSshSignatureHash -ne 'missing'
    if ((Get-LToolsMapValue $PreviousSigning 'sshSigned') -ne [bool]$currentSshSigned) { return $false }
    if ([string](Get-LToolsMapValue $PreviousSigning 'sshPublicKeyFingerprint') -ne [string]$CurrentSshKeyFingerprint) { return $false }
    if ([string](Get-LToolsMapValue $PreviousSigning 'sshIdentity') -ne [string]$CurrentSshIdentity) { return $false }
    foreach ($key in (Get-LToolsMapKeys $CurrentHashes)) {
        if ((Get-LToolsMapValue $PreviousHashes $key) -ne (Get-LToolsMapValue $CurrentHashes $key)) { return $false }
    }
    foreach ($key in (Get-LToolsMapKeys $PreviousHashes)) {
        if ((Get-LToolsMapValue $CurrentHashes $key) -ne (Get-LToolsMapValue $PreviousHashes $key)) { return $false }
    }
    return $true
}

function Get-LToolsPackagePending([bool]$NeedPackage, [bool]$PackageCompleted) {
    return ($NeedPackage -and -not $PackageCompleted)
}
