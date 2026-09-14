function New-LToolsThirdPartyLicenseBundle {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$Destination,
        [Parameter(Mandatory = $true)][string]$ManifestPath,
        [Parameter(Mandatory = $true)][string]$Platform
    )

    if (Test-Path -LiteralPath $Destination) {
        throw "El destino de licencias ya existe; no se mezclan datos previos: $Destination"
    }
    $manifest = [IO.Path]::GetFullPath($ManifestPath)
    $metadataLines = & cargo metadata --offline --format-version 1 --manifest-path $manifest --filter-platform $Platform
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata falló; no se publica un paquete sin avisos verificables.' }
    try { $metadata = ($metadataLines -join [Environment]::NewLine) | ConvertFrom-Json }
    catch { throw "No se pudo interpretar cargo metadata para generar avisos: $($_.Exception.Message)" }
    if (-not $metadata.packages -or @($metadata.packages).Count -eq 0) {
        throw 'cargo metadata no devolvió paquetes para generar avisos.'
    }

    $rootManifest = [IO.Path]::GetFullPath($manifest)
    $rootComparison = [StringComparison]::OrdinalIgnoreCase
    $packages = @($metadata.packages | Where-Object {
        [IO.Path]::GetFullPath([string]$_.manifest_path) -ne $rootManifest
    } | Sort-Object name, version)
    if ($packages.Count -eq 0) { throw 'cargo metadata no devolvió dependencias de terceros.' }

    New-Item -ItemType Directory -Path $Destination | Out-Null
    $index = [Text.StringBuilder]::new()
    [void]$index.AppendLine('LTools - avisos de dependencias de terceros')
    [void]$index.AppendLine("Plataforma Cargo: $Platform")
    [void]$index.AppendLine('Se conservan los ficheros legales originales de cada paquete Cargo incluido.')
    [void]$index.AppendLine()
    [void]$index.AppendLine("Paquete`tVersión`tExpresión de licencia")
    $copied = 0
    foreach ($package in $packages) {
        $name = [string]$package.name
        $version = [string]$package.version
        $license = [string]$package.license
        if ($name -notmatch '^[A-Za-z0-9_.+-]+$' -or $version -notmatch '^[A-Za-z0-9_.+-]+$') {
            throw "Nombre/versión de paquete no seguros en cargo metadata: $name $version"
        }
        if ([string]::IsNullOrWhiteSpace($license)) { throw "El paquete $name $version no declara licencia." }
        $crateRoot = [IO.Path]::GetFullPath((Split-Path -Parent ([string]$package.manifest_path)))
        $packageDestination = Join-Path $Destination "$name-$version"
        $legalFiles = @(Get-ChildItem -LiteralPath $crateRoot -Recurse -File -Force | Where-Object {
            $_.Name -match '^(?i:LICENSE|COPYING|NOTICE|COPYRIGHT|PATENTS)(?:[._ -].*)?$' -and
                ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0
        })
        if ($legalFiles.Count -eq 0) {
            throw "$name $version no contiene ficheros LICENSE/COPYING/NOTICE verificables."
        }
        [void]$index.AppendLine("$name`t$version`t$license")
        foreach ($file in $legalFiles) {
            $fullFile = [IO.Path]::GetFullPath($file.FullName)
            $prefix = $crateRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
            if (-not $fullFile.StartsWith($prefix, $rootComparison)) {
                throw "Aviso de licencia fuera del directorio del paquete: $fullFile"
            }
            $relative = $fullFile.Substring($prefix.Length)
            $target = Join-Path $packageDestination $relative
            $parent = Split-Path -Parent $target
            New-Item -ItemType Directory -Force -Path $parent | Out-Null
            Copy-Item -LiteralPath $fullFile -Destination $target
            $copied++
        }
    }
    [void]$index.AppendLine("Ficheros legales copiados: $copied")
    [IO.File]::WriteAllText((Join-Path $Destination 'INDEX.txt'), $index.ToString(), [Text.UTF8Encoding]::new($false))
    Write-Output "Avisos de terceros preparados: $($packages.Count) paquetes, $copied ficheros legales."
}
