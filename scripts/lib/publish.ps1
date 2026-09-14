function Write-Utf8NoBom([string]$Path, [string]$Content) {
    [IO.File]::WriteAllText($Path, $Content, [Text.UTF8Encoding]::new($false))
}

function Copy-PublishTree([string]$Source, [string]$Destination) {
    foreach ($child in Get-ChildItem -LiteralPath $Source -Force) {
        $target = Join-Path $Destination $child.Name
        if (($child.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "La release existente contiene un enlace/punto de reanálisis; no se reemplaza: $($child.FullName)"
        }
        if ($child.PSIsContainer) {
            New-Item -ItemType Directory -Path $target | Out-Null
            Copy-PublishTree $child.FullName $target
        } else {
            Copy-Item -LiteralPath $child.FullName -Destination $target
        }
    }
}

function New-StagedDirectory([string]$Destination) {
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent -PathType Container)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    $stage = Join-Path $parent ".$(Split-Path -Leaf $Destination).ltools-stage.$([guid]::NewGuid().ToString('N'))"
    if (Test-Path -LiteralPath $stage) { throw "Ya existe la carpeta de staging: $stage" }
    New-Item -ItemType Directory -Path $stage | Out-Null
    try {
        if (Test-Path -LiteralPath $Destination) {
            $destinationItem = Get-Item -LiteralPath $Destination -Force
            if (-not $destinationItem.PSIsContainer -or
                ($destinationItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "La publicación existente no es una carpeta normal: $Destination"
            }
            Copy-PublishTree $Destination $stage
        }
        return $stage
    } catch {
        Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction SilentlyContinue
        throw
    }
}

function Publish-StagedFile([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
        throw "El archivo temporal no existe: $Source"
    }
    $sourceItem = Get-Item -LiteralPath $Source -Force
    if (($sourceItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "No se publica un archivo temporal que sea enlace: $Source"
    }

    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent -PathType Container)) {
        throw "No existe la carpeta de destino: $parent"
    }
    if (Test-Path -LiteralPath $Destination) {
        $destinationItem = Get-Item -LiteralPath $Destination -Force
        if ($destinationItem.PSIsContainer -or
            ($destinationItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "El destino no es un archivo normal: $Destination"
        }
        try {
            [IO.File]::Replace($Source, $Destination, $null)
            return
        } catch {
            # Algunos sistemas de archivos no implementan ReplaceFile. En ese
            # caso conserva una copia vecina y restaura si Move falla.
            $backup = Join-Path $parent ".$(Split-Path -Leaf $Destination).previous.$([guid]::NewGuid().ToString('N'))"
            [IO.File]::Move($Destination, $backup)
            try {
                [IO.File]::Move($Source, $Destination)
            } catch {
                if (-not (Test-Path -LiteralPath $Destination) -and
                    (Test-Path -LiteralPath $backup -PathType Leaf)) {
                    [IO.File]::Move($backup, $Destination)
                }
                throw
            }
            Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue
        }
    } else {
        [IO.File]::Move($Source, $Destination)
    }
}

function Publish-StagedDirectory([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Container)) {
        throw "La carpeta temporal no existe: $Source"
    }
    $sourceItem = Get-Item -LiteralPath $Source -Force
    if (($sourceItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "No se publica una carpeta temporal que sea enlace: $Source"
    }
    if (Test-Path -LiteralPath $Destination) {
        $destinationItem = Get-Item -LiteralPath $Destination -Force
        if (-not $destinationItem.PSIsContainer -or
            ($destinationItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "El destino no es una carpeta normal: $Destination"
        }
        $backup = "$Destination.previous.$([guid]::NewGuid().ToString('N'))"
        [IO.Directory]::Move($Destination, $backup)
        try {
            [IO.Directory]::Move($Source, $Destination)
        } catch {
            if (-not (Test-Path -LiteralPath $Destination -PathType Container) -and
                (Test-Path -LiteralPath $backup -PathType Container)) {
                [IO.Directory]::Move($backup, $Destination)
            }
            throw
        }
        Remove-Item -LiteralPath $backup -Recurse -Force -ErrorAction SilentlyContinue
    } else {
        [IO.Directory]::Move($Source, $Destination)
    }
}
