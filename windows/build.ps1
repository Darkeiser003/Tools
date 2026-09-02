#requires -Version 5.1
<#
    Builder nativo de LTools para Windows.

    La release oficial de Windows se produce aquí, no mediante el builder
    AppImage de Linux. Compila únicamente cuando han cambiado fuentes Rust o
    Cargo; cambios de documentación/recursos solo vuelven a empaquetar.
    Los artefactos publicables se sincronizan en la carpeta release común.
#>

[CmdletBinding()]
param(
    [switch]$Help,
    [switch]$Clean,
    [switch]$Force,
    [switch]$Fast,
    [switch]$NoTests,
    [switch]$NoPackage,
    [switch]$NoRun,
    [switch]$NonInteractive,
    [switch]$NoLog,
    [string]$Log,
    [string]$Output,
    [string]$ReleaseOutput,
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$CargoManifest = Join-Path $Root "rust\Cargo.toml"
$DefaultOutput = Join-Path $Root "dist\windows"
$OutputDir = if ($Output) { [IO.Path]::GetFullPath($Output) } else { $DefaultOutput }
$PublishDir = if ($ReleaseOutput) { [IO.Path]::GetFullPath($ReleaseOutput) } else { Join-Path $Root "release" }
$Version = ((Select-String -Path $CargoManifest -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value)
if (-not $Version) { throw "No se pudo leer la versión desde rust/Cargo.toml" }
$TargetDir = Join-Path $Root "rust\target\windows"
$env:CARGO_TARGET_DIR = $TargetDir
$CargoReleaseDir = Join-Path $TargetDir "$Target\release"
$Binary = Join-Path $CargoReleaseDir "ltools.exe"
$PackageArch = if ($Target -match '^aarch64') { 'arm64' } elseif ($Target -match '^i686') { 'x86' } else { 'x86_64' }
$StatePath = Join-Path $OutputDir ".build-state.json"
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"

function Show-Help {
    @"
Uso: powershell -ExecutionPolicy Bypass -File windows\build.ps1 [opciones]

  -Force          Fuerza compilación y empaquetado.
  -Clean          Limpia el target Windows antes de compilar.
  -Fast           Release incremental, sin LTO (desarrollo).
  -NoTests        Omite cargo test y el smoke Windows.
  -NoPackage      Compila pero no crea el ZIP.
  -NoRun          No lanza el smoke al terminar.
  -Target T       Target Rust (por defecto x86_64-pc-windows-msvc).
  -Output RUTA    Carpeta de salida.
  -ReleaseOutput RUTA
                  Carpeta canónica de publicación (por defecto ..\release).
  -Log FICHERO    Fichero de log; -NoLog desactiva logs.
  -NonInteractive No solicita confirmaciones.

Salida: ltools-VERSION-windows-ARQUITECTURA.zip, perfiles exe/CLI y una carpeta
portable. Los artefactos publicables se copian también a release\.
"@
}
if ($Help) { Show-Help; exit 0 }

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$LogPath = if ($NoLog) { $null } elseif ($Log) { [IO.Path]::GetFullPath($Log) } else { Join-Path $OutputDir "build-windows-$Stamp-$PID.log" }
if ($LogPath) {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $LogPath) | Out-Null
    "WinSlim-Tools Windows build $Version - $(Get-Date -Format o)" | Set-Content -Encoding UTF8 $LogPath
}
$TimingPath = if ($LogPath) { [IO.Path]::ChangeExtension($LogPath, $null) + "-timings.tsv" } else { $null }
if ($TimingPath) { ("step" + [char]9 + "seconds" + [char]9 + "status") | Set-Content -Encoding UTF8 $TimingPath }
$script:BuildStart = [Diagnostics.Stopwatch]::StartNew()

function Write-Log([string]$Line) {
    Write-Host $Line
    if ($LogPath) { $Line | Add-Content -Encoding UTF8 $LogPath }
}
function Invoke-Step([string]$Name, [scriptblock]$Action) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    Write-Log "==> $Name"
    try {
        $global:LASTEXITCODE = 0
        & $Action 2>&1 | ForEach-Object { Write-Log ([string]$_) }
        if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) { throw "código de salida $LASTEXITCODE" }
        $watch.Stop()
        if ($TimingPath) { ( $Name + [char]9 + [math]::Round($watch.Elapsed.TotalSeconds, 3) + [char]9 + "ok" ) | Add-Content -Encoding UTF8 $TimingPath }
        Write-Log ("    OK ({0:N2}s)" -f $watch.Elapsed.TotalSeconds)
    } catch {
        $watch.Stop()
        if ($TimingPath) { ( $Name + [char]9 + [math]::Round($watch.Elapsed.TotalSeconds, 3) + [char]9 + "failed" ) | Add-Content -Encoding UTF8 $TimingPath }
        Write-Log "    ERROR: $($_.Exception.Message)"
        throw
    }
}
function Get-Relative([string]$Path) {
    $rootUri = New-Object Uri(($Root.TrimEnd('\') + '\'))
    $pathUri = New-Object Uri($Path)
    return [Uri]::UnescapeDataString($rootUri.MakeRelativeUri($pathUri).ToString()).Replace('\', '/')
}
function Get-Inputs {
    $files = @()
    foreach ($base in @("rust", "windows", "appimage", "distribution", "tests", "docs")) {
        $dir = Join-Path $Root $base
        if (Test-Path $dir) { $files += Get-ChildItem -LiteralPath $dir -Recurse -File }
    }
    $readme = Join-Path $Root "README.md"
    if (Test-Path $readme) { $files += Get-Item $readme }
    return $files | Where-Object {
        $relative = Get-Relative $_.FullName
        $relative -notmatch '^(rust/target|dist)/'
    } | Sort-Object FullName -Unique
}
function Get-Signatures {
    $result = [ordered]@{}
    foreach ($file in Get-Inputs) {
        $relative = Get-Relative $file.FullName
        $result[$relative] = "$($file.Length):$($file.LastWriteTimeUtc.Ticks)"
    }
    return $result
}
function Get-MapValue($Map, [string]$Key) {
    if ($null -eq $Map) { return $null }
    $property = $Map.PSObject.Properties[$Key]
    if ($property) { return $property.Value }
    return $null
}
function Get-ChangeClass($Old, $New) {
    $rust = @(); $package = @(); $tests = @()
    foreach ($key in $New.Keys) {
        if ($null -eq $Old -or (Get-MapValue $Old $key) -ne $New[$key]) {
            if ($key -match '^(rust/|windows/|appimage/|distribution/|README\.md$)') { $package += $key }
            if ($key -match '^rust/(src/|Cargo\.)') { $rust += $key }
            if ($key -match '(^tests/|^windows/tests/|\.md$)') { $tests += $key }
        }
    }
    if ($Old) {
        foreach ($property in $Old.PSObject.Properties) {
            if (-not $New.Contains($property.Name)) {
                $package += $property.Name
                if ($property.Name -match '^rust/(src/|Cargo\.)') { $rust += $property.Name }
            }
        }
    }
    [pscustomobject]@{ Rust = $rust; Package = $package; Tests = $tests }
}
function Invoke-NativeCommand([string]$Executable, [string[]]$Arguments) {
    # Windows PowerShell 5.1 convierte stderr de procesos nativos en
    # ErrorRecord. Cargo escribe su progreso en stderr incluso cuando todo
    # termina correctamente; usar ErrorAction=Stop aquí provocaba falsos
    # fallos durante líneas como «Compiling version_check».
    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & $Executable @Arguments 2>&1 | ForEach-Object { Write-Log ([string]$_) }
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
    return [int]$exitCode
}
function Invoke-Cargo([string[]]$Arguments) {
    $exitCode = Invoke-NativeCommand 'cargo' $Arguments
    if ($exitCode -ne 0) { throw "cargo terminó con código $exitCode" }
}
function Ensure-Target {
    $rustup = Get-Command rustup -ErrorAction SilentlyContinue
    if (-not $rustup) {
        Write-Log "AVISO: rustup no está disponible; Cargo comprobará el target al compilar."
        return
    }
    $installed = @(& $rustup.Source target list --installed 2>$null)
    if ($installed -contains $Target) { return }
    if ($NonInteractive) { throw "Falta el target Rust $Target. Ejecuta: rustup target add $Target" }
    $answer = Read-Host "Falta el target Rust $Target. ¿Instalarlo con rustup? [S/n]"
    if ($answer -and $answer.Trim().ToLowerInvariant() -notin @('s', 'si', 'sí', 'y', 'yes')) {
        throw "No se puede compilar sin el target $Target"
    }
    Invoke-Step "Instalando target Rust $Target" {
        $exitCode = Invoke-NativeCommand $rustup.Source @('target', 'add', $Target)
        if ($exitCode -ne 0) { throw "rustup terminó con código $exitCode" }
    }
}

Write-Log "WinSlim-Tools Windows build $Version"
Write-Log "Target: $Target"
Write-Log "Salida: $OutputDir"
Write-Log "Publicación: $PublishDir"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw "No se encontró cargo. Instala Rust mediante rustup." }
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) { throw "No se encontró rustc." }
Ensure-Target

$oldState = $null
if (Test-Path $StatePath) {
    try { $oldState = Get-Content $StatePath -Raw | ConvertFrom-Json } catch { Write-Log "AVISO: estado incremental inválido; se reconstruirá." }
}
$newSignatures = Get-Signatures
$oldSignatures = if ($oldState) { $oldState.files } else { $null }
$classes = Get-ChangeClass $oldSignatures $newSignatures
$needCompile = $Force -or $Clean -or -not (Test-Path $Binary) -or $classes.Rust.Count -gt 0
$existingZip = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch.zip"
$existingCli = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch-cli.exe"
$publishedExe = Join-Path $PublishDir "ltools-$Version-windows-$PackageArch.exe"
$publishedCli = Join-Path $PublishDir "ltools-$Version-windows-$PackageArch-cli.exe"
$publishedZip = Join-Path $PublishDir "ltools-$Version-windows-$PackageArch.zip"
$needPackage = $Force -or $needCompile -or $classes.Package.Count -gt 0 -or -not (Test-Path $existingZip) -or -not (Test-Path $existingCli)
$needPackage = $needPackage -or -not (Test-Path $publishedExe) -or -not (Test-Path $publishedCli) -or -not (Test-Path $publishedZip)
$needTests = -not $NoTests -and ($Force -or $needCompile -or $classes.Tests.Count -gt 0)
Write-Log ("Cambios: Rust={0}, paquete={1}, pruebas={2}" -f $classes.Rust.Count, $classes.Package.Count, $classes.Tests.Count)
Write-Log ("Plan incremental: compilar={0}, empaquetar={1}, probar={2}" -f $needCompile, $needPackage, $needTests)

if ($Clean) {
    Invoke-Step "Limpiando target Windows" { Remove-Item -LiteralPath $TargetDir -Recurse -Force -ErrorAction SilentlyContinue }
    $needCompile = $true
}
if ($needCompile) {
    $cargoArgs = @('build', '--manifest-path', $CargoManifest, '--release', '--target', $Target)
    if ($Fast) { $env:CARGO_PROFILE_RELEASE_LTO = 'false'; $env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '256'; $env:CARGO_PROFILE_RELEASE_INCREMENTAL = 'true' }
    Invoke-Step "Compilando backend Rust Windows" { Invoke-Cargo $cargoArgs }
} else { Write-Log "    SKIP: backend Rust sin cambios relevantes." }

if ($needTests) {
    Invoke-Step "Ejecutando tests Rust" { Invoke-Cargo @('test', '--manifest-path', $CargoManifest, '--target', $Target) }
    if (-not $NoRun -and $Target -match 'windows') {
        $smoke = Join-Path $Root 'windows\tests\smoke.ps1'
        if (Test-Path $smoke) {
        Invoke-Step "Ejecutando smoke Windows" {
                $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $smoke, '-Binary', $Binary, '-Version', $Version)
                if ($exitCode -ne 0) { throw "smoke Windows terminó con código $exitCode" }
            }
        }
        $e2e = Join-Path $Root 'windows\tests\e2e.ps1'
        if (Test-Path $e2e) {
            Invoke-Step "Ejecutando E2E Windows" {
                $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $e2e, '-Binary', $Binary)
                if ($exitCode -ne 0) { throw "E2E Windows terminó con código $exitCode" }
            }
        }
    }
} else { Write-Log "    SKIP: tests sin cambios relevantes o desactivados." }

if ($needPackage -and -not $NoPackage) {
    if (-not (Test-Path $Binary)) { throw "No existe el ejecutable Windows: $Binary" }
    # La carpeta de salida contiene únicamente resultados regenerables. Se
    # eliminan solo artefactos LTools previos para no mezclar versiones en el
    # manifiesto de release al cambiar Cargo.toml.
    Get-ChildItem -LiteralPath $OutputDir -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like 'ltools-*.exe' -or $_.Name -like 'ltools-*.zip' } |
        Remove-Item -Force
    Get-ChildItem -LiteralPath $OutputDir -Directory -Filter 'ltools-*windows-*' -ErrorAction SilentlyContinue |
        Remove-Item -Recurse -Force
    $ExecutableArtifact = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch.exe"
    Copy-Item -LiteralPath $Binary -Destination $ExecutableArtifact -Force
    $CliExecutableArtifact = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch-cli.exe"
    Copy-Item -LiteralPath $Binary -Destination $CliExecutableArtifact -Force
    $portable = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch"
    Remove-Item -LiteralPath $portable -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $portable | Out-Null
    Copy-Item -LiteralPath $Binary -Destination (Join-Path $portable 'ltools.exe')
    Copy-Item -LiteralPath $Binary -Destination (Join-Path $portable 'ltools-cli.exe')
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'ltools.ps1') -Destination $portable
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'ltools.cmd') -Destination $portable
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'ltools-cli.ps1') -Destination $portable
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'ltools-cli.cmd') -Destination $portable
    Copy-Item -LiteralPath (Join-Path $Root 'README.md') -Destination $portable
    $capabilities = & $Binary capabilities --format json 2>&1
    if ($LASTEXITCODE -ne 0) { throw "No se pudo generar ltools-capabilities.json: $capabilities" }
    $capabilities | Set-Content -Encoding UTF8 (Join-Path $portable 'ltools-capabilities.json')
    Copy-Item -LiteralPath (Join-Path $portable 'ltools-capabilities.json') -Destination $OutputDir -Force
    $terminalDescriptor = & $Binary capabilities --format terminal-json 2>&1
    if ($LASTEXITCODE -ne 0) { throw "No se pudo generar ltools-terminal.json: $terminalDescriptor" }
    $terminalDescriptor | Set-Content -Encoding UTF8 (Join-Path $portable 'ltools-terminal.json')
    Copy-Item -LiteralPath (Join-Path $portable 'ltools-terminal.json') -Destination $OutputDir -Force
    Copy-Item -LiteralPath (Join-Path $Root 'appimage\ltools-capabilities.schema.json') -Destination $portable
    Copy-Item -LiteralPath (Join-Path $Root 'appimage\ltools-terminal.schema.json') -Destination $portable
    Copy-Item -LiteralPath (Join-Path $Root 'appimage\ltools-terminal.schema.json') -Destination $OutputDir -Force
    if (-not (Select-String -Path (Join-Path $portable 'ltools-capabilities.json') -Pattern '"schema": "ltools-capabilities-v1"' -Quiet)) {
        throw 'El descriptor JSON de capacidades Windows no es válido.'
    }
    $terminalJson = Get-Content -Raw -LiteralPath (Join-Path $portable 'ltools-terminal.json') | ConvertFrom-Json
    if ($terminalJson.schema -ne 'ltools-terminal-integration-v1' -or
        $terminalJson.platform -ne 'windows' -or
        $terminalJson.entrypoint.command -ne 'ltools.exe' -or
        $terminalJson.integration.optional -ne $true -or
        $terminalJson.integration.standalone_releases_require_it -ne $false -or
        $terminalJson.integration.exclusive_host_family -ne 'lterminal' -or
        $terminalJson.host.product -ne 'WinSlim Terminal') {
        throw 'El descriptor JSON de integración Windows no declara WinSlim Terminal correctamente.'
    }
    @("WinSlim-Tools $Version", "Platform: Windows", "Target: $Target", "Backend: ltools.exe", "CLI backend: ltools-cli.exe (no arguments prints help)", "Linux-only Bash modules and AppImage assets are not included.") |
        Set-Content -Encoding UTF8 (Join-Path $portable 'BUILD-INFO.txt')
    $zip = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch.zip"
    Remove-Item -LiteralPath $zip -Force -ErrorAction SilentlyContinue
    Invoke-Step "Empaquetando ZIP portable Windows" { Compress-Archive -Path (Join-Path $portable '*') -DestinationPath $zip -CompressionLevel Optimal }
    Write-Log "Ejecutable Windows: $ExecutableArtifact"

    # release/ es la carpeta común que se puede subir a GitHub. Solo se
    # reemplazan los artefactos Windows de esta versión; los AppImage Linux
    # que ya haya publicado el builder Linux se conservan.
    New-Item -ItemType Directory -Force -Path $PublishDir | Out-Null
    Get-ChildItem -LiteralPath $PublishDir -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "ltools-$Version-windows-*" } |
        Remove-Item -Force
    foreach ($file in @(
        $ExecutableArtifact,
        $CliExecutableArtifact,
        $zip,
        (Join-Path $OutputDir 'ltools-capabilities.json'),
        (Join-Path $OutputDir 'ltools-terminal.json'),
        (Join-Path $OutputDir 'ltools-terminal.schema.json')
    )) {
        if (Test-Path -LiteralPath $file -PathType Leaf) {
            Copy-Item -LiteralPath $file -Destination $PublishDir -Force
        }
    }
    foreach ($file in @(
        (Join-Path $Root 'distribution\ltools-project.json'),
        (Join-Path $Root 'distribution\ltools-project.schema.json'),
        (Join-Path $Root 'distribution\ltools-release.schema.json')
    )) {
        Copy-Item -LiteralPath $file -Destination $PublishDir -Force
    }
    Write-Log "Carpeta release Windows preparada: $PublishDir"
} elseif (-not $NoPackage) { Write-Log "    SKIP: paquete Windows ya actualizado." }

if ($needPackage -and -not $NoPackage) {
    $releaseManifestOutput = Join-Path $PublishDir "ltools-release.json"
    $releaseRepository = if ($env:LTOOLS_GITHUB_REPOSITORY) { $env:LTOOLS_GITHUB_REPOSITORY } else { 'Darkeiser003/Tools' }
    $releaseTag = if ($env:LTOOLS_GITHUB_TAG) { $env:LTOOLS_GITHUB_TAG } else { "v$Version" }
    Invoke-Step "Generando manifiesto verificable de release" {
        & $Binary release-manifest --output $releaseManifestOutput --repository $releaseRepository --tag $releaseTag --artifacts-dir $PublishDir 2>&1 |
            ForEach-Object { Write-Log ([string]$_) }
        if ($LASTEXITCODE -ne 0) { throw "no se pudo generar ltools-release.json" }
    }
    $distribution = Join-Path $Root 'distribution'
    New-Item -ItemType Directory -Force -Path (Join-Path $Root 'dist') | Out-Null
    Copy-Item -LiteralPath $releaseManifestOutput -Destination (Join-Path $Root 'dist\ltools-release.json') -Force
    Copy-Item -LiteralPath (Join-Path $distribution 'ltools-project.json') -Destination (Join-Path $Root 'dist\ltools-project.json') -Force
    Copy-Item -LiteralPath (Join-Path $distribution 'ltools-project.schema.json') -Destination (Join-Path $Root 'dist\ltools-project.schema.json') -Force
    Copy-Item -LiteralPath (Join-Path $distribution 'ltools-release.schema.json') -Destination (Join-Path $Root 'dist\ltools-release.schema.json') -Force
    $manifest = Get-Content -Raw -LiteralPath $releaseManifestOutput | ConvertFrom-Json
    if ($manifest.schema -ne 'ltools-release-v1' -or $manifest.application -ne 'WinSlim-Tools' -or
        $manifest.hash_algorithm -ne 'sha256' -or @($manifest.artifacts).Count -lt 1) {
        throw 'El manifiesto de release Windows no supera la validación estructural.'
    }
    Write-Log "Manifiesto de release: $releaseManifestOutput"
}

$state = [ordered]@{ version = $Version; target = $Target; builtAt = (Get-Date).ToUniversalTime().ToString('o'); files = $newSignatures }
$state | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 $StatePath
if ($TimingPath) {
    $total = [math]::Round($script:BuildStart.Elapsed.TotalSeconds, 3)
    ( "total" + [char]9 + $total + [char]9 + "ok" ) | Add-Content -Encoding UTF8 $TimingPath
    Write-Log "Tiempo total: $total s"
}
Write-Log "Build Windows terminada correctamente."
