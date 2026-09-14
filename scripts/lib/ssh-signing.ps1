#requires -Version 5.1

function Get-LToolsSshSigningConfiguration([string]$Root = (Get-Location).Path) {
    $privateKey = if ($env:LTOOLS_SSH_SIGNING_KEY_FILE) { $env:LTOOLS_SSH_SIGNING_KEY_FILE } elseif ($env:LTERMINAL_SSH_SIGNING_KEY_FILE) { $env:LTERMINAL_SSH_SIGNING_KEY_FILE } else { $null }
    $configuredKey = $null
    $publicKey = $null
    if (-not $privateKey) {
        $git = Get-Command git -ErrorAction SilentlyContinue
        if ($git) {
            $configuredKey = (& $git.Source -C $Root config --get user.signingkey 2>$null | Select-Object -First 1)
            if ($LASTEXITCODE -ne 0 -or $configuredKey -like 'key::*') { $configuredKey = $null }
        }
        if ($configuredKey) {
            $configuredKey = [string]$configuredKey
            if ($configuredKey.StartsWith('~/')) { $configuredKey = Join-Path $HOME $configuredKey.Substring(2) }
            if ($configuredKey.EndsWith('.pub', [StringComparison]::OrdinalIgnoreCase)) {
                $publicKey = $configuredKey
                $privateKey = $configuredKey.Substring(0, $configuredKey.Length - 4)
            } else { $privateKey = $configuredKey }
        } else {
            $privateKey = Join-Path $HOME '.ssh\id_ed25519'
        }
    }
    if ($privateKey -and $privateKey.StartsWith('~/')) { $privateKey = Join-Path $HOME $privateKey.Substring(2) }
    if ($env:LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE) { $publicKey = $env:LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE }
    elseif ($env:LTERMINAL_SSH_SIGNING_PUBLIC_KEY_FILE) { $publicKey = $env:LTERMINAL_SSH_SIGNING_PUBLIC_KEY_FILE }
    elseif (-not $publicKey) { $publicKey = "$privateKey.pub" }
    if ($publicKey -and $publicKey.StartsWith('~/')) { $publicKey = Join-Path $HOME $publicKey.Substring(2) }
    $identity = if ($env:LTOOLS_SSH_SIGNING_IDENTITY) { $env:LTOOLS_SSH_SIGNING_IDENTITY } elseif ($env:LTERMINAL_SSH_SIGNING_IDENTITY) { $env:LTERMINAL_SSH_SIGNING_IDENTITY } else { $null }
    if (-not $identity) {
        $git = Get-Command git -ErrorAction SilentlyContinue
        if ($git) {
            $identity = (& $git.Source -C $Root config --get user.email 2>$null | Select-Object -First 1)
            if ($LASTEXITCODE -ne 0) { $identity = $null }
        }
    }
    $sshKeygen = Get-Command ssh-keygen.exe, ssh-keygen -ErrorAction SilentlyContinue | Select-Object -First 1
    return [pscustomobject]@{
        PrivateKey = $privateKey
        PublicKey = $publicKey
        Identity = [string]$identity
        SshKeygen = if ($sshKeygen) { $sshKeygen.Source } else { $null }
        CanSign = [bool]($sshKeygen -and $privateKey -and (Test-Path -LiteralPath $privateKey -PathType Leaf))
        CanVerify = [bool]($sshKeygen -and $publicKey -and (Test-Path -LiteralPath $publicKey -PathType Leaf) -and $identity -match '^[A-Za-z0-9._@+-]+$')
    }
}

function New-LToolsAllowedSignersFile([string]$Path, $Configuration) {
    if (-not $Configuration.CanVerify) { throw 'Falta ssh-keygen, la clave pública SSH o la identidad para verificar la firma.' }
    $keyFields = [IO.File]::ReadAllText($Configuration.PublicKey).Trim() -split '\s+'
    if ($keyFields.Count -lt 2 -or $keyFields[0] -notmatch '^(ssh-|sk-ssh-|ecdsa-)') {
        throw 'La clave pública no tiene un formato OpenSSH admitido.'
    }
    $allowed = '{0} {1} {2}{3}' -f $Configuration.Identity, $keyFields[0], $keyFields[1], [Environment]::NewLine
    [IO.File]::WriteAllText($Path, $allowed, [Text.UTF8Encoding]::new($false))
}

function Test-LToolsSshManifestSignature([string]$Manifest, [string]$Signature, $Configuration) {
    if (-not $Configuration.CanVerify) { throw 'No se puede validar la firma SSH: falta ssh-keygen, la clave pública o la identidad GitHub.' }
    if (-not (Test-Path -LiteralPath $Manifest -PathType Leaf) -or -not (Test-Path -LiteralPath $Signature -PathType Leaf)) {
        throw 'Falta el manifiesto o la firma OpenSSH de la release.'
    }
    $temporary = Join-Path ([IO.Path]::GetTempPath()) ('ltools-ssh-verify-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $temporary | Out-Null
    try {
        $allowed = Join-Path $temporary 'allowed_signers'
        New-LToolsAllowedSignersFile $allowed $Configuration
        $arguments = @('-Y', 'verify', '-f', $allowed, '-I', $Configuration.Identity, '-n', 'ltools-release', '-s', $Signature)
        $start = [Diagnostics.ProcessStartInfo]::new()
        $start.FileName = $Configuration.SshKeygen
        $start.Arguments = (($arguments | ForEach-Object { '"' + ([string]$_).Replace('"', '\"') + '"' }) -join ' ')
        $start.UseShellExecute = $false
        $start.RedirectStandardInput = $true
        $process = [Diagnostics.Process]::new()
        $process.StartInfo = $start
        if (-not $process.Start()) { throw 'No se pudo iniciar ssh-keygen para verificar la firma.' }
        $contents = [IO.File]::ReadAllBytes($Manifest)
        $process.StandardInput.BaseStream.Write($contents, 0, $contents.Length)
        $process.StandardInput.Close()
        if (-not $process.WaitForExit(60000)) { $process.Kill(); throw 'ssh-keygen agotó el tiempo al verificar la firma.' }
        if ($process.ExitCode -ne 0) { throw 'La firma OpenSSH de SHA256SUMS.txt no es válida para la clave configurada.' }
    } finally {
        if (Test-Path -LiteralPath $temporary -PathType Container) { Remove-Item -LiteralPath $temporary -Recurse -Force -ErrorAction SilentlyContinue }
    }
}

function Invoke-LToolsSshManifestSigning([string]$Manifest, [string]$Signature, $Configuration) {
    if (-not $Configuration.CanSign -or -not $Configuration.CanVerify) {
        throw 'Falta la clave privada/pública OpenSSH, ssh-keygen o la identidad GitHub para firmar releases.'
    }
    if (-not (Test-Path -LiteralPath $Manifest -PathType Leaf) -or (Get-Item -LiteralPath $Manifest).Length -eq 0) {
        throw 'No se puede firmar un SHA256SUMS.txt ausente o vacío.'
    }
    $temporary = Join-Path ([IO.Path]::GetTempPath()) ('ltools-ssh-sign-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $temporary | Out-Null
    try {
        $stagedManifest = Join-Path $temporary 'SHA256SUMS.txt'
        $stagedSignature = "$stagedManifest.sig"
        $allowed = Join-Path $temporary 'allowed_signers'
        Copy-Item -LiteralPath $Manifest -Destination $stagedManifest
        New-LToolsAllowedSignersFile $allowed $Configuration
        $output = & $Configuration.SshKeygen -Y sign -f $Configuration.PrivateKey -n ltools-release $stagedManifest 2>&1
        if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $stagedSignature -PathType Leaf)) {
            throw "ssh-keygen no pudo firmar SHA256SUMS.txt: $($output -join ' ')"
        }
        Test-LToolsSshManifestSignature $stagedManifest $stagedSignature $Configuration
        $parent = Split-Path -Parent $Signature
        if (-not (Test-Path -LiteralPath $parent -PathType Container)) { New-Item -ItemType Directory -Path $parent | Out-Null }
        $temporarySignature = "$Signature.tmp-$PID"
        Copy-Item -LiteralPath $stagedSignature -Destination $temporarySignature -Force
        Move-Item -LiteralPath $temporarySignature -Destination $Signature -Force
        Write-Output "Manifiesto firmado y verificado con OpenSSH: $Signature"
    } finally {
        if (Test-Path -LiteralPath $temporary -PathType Container) { Remove-Item -LiteralPath $temporary -Recurse -Force -ErrorAction SilentlyContinue }
    }
}
