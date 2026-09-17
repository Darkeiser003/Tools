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
    [switch]$Menu,
    [switch]$Clean,
    [switch]$Force,
    [switch]$Fast,
    [switch]$StrictSecurity,
    [switch]$SecurityReview,
    [switch]$AutoFix,
    [switch]$NoTests,
    [switch]$NoSmoke,
    [switch]$NoE2E,
    [switch]$NoPackage,
    [switch]$NoRun,
    [switch]$NonInteractive,
    [switch]$NoLog,
    [ValidateSet('all', 'backend', 'frontend', 'cli', 'package', 'exe')]
    [string]$Component = 'all',
    [switch]$Preview,
    [string]$TestExisting,
    [switch]$RequireSigning,
    [switch]$AllowUnsigned,
    [string]$Log,
    [string]$Output,
    [string]$ReleaseOutput,
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"
$Root = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$WindowsDir = Join-Path $Root 'windows'
$PublishHelpers = Join-Path $PSScriptRoot 'lib\publish.ps1'
. $PublishHelpers
. (Join-Path $PSScriptRoot 'lib\third-party-licenses.ps1')
. (Join-Path $PSScriptRoot 'lib\ssh-signing.ps1')
. (Join-Path $PSScriptRoot 'lib\build-state.ps1')
$CargoManifest = Join-Path $Root "rust\Cargo.toml"
$DefaultOutput = Join-Path $Root "dist\windows"
$OutputDir = [IO.Path]::GetFullPath($(if ($Output) { $Output } else { $DefaultOutput }))
$PublishDir = [IO.Path]::GetFullPath($(if ($ReleaseOutput) { $ReleaseOutput } else { Join-Path $Root "release" }))
$BuildProfile = if ($Fast) { 'fast' } else { 'release' }

function Get-ComparablePath([string]$Path) {
    return [IO.Path]::GetFullPath($Path).TrimEnd([char[]]@(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar
    ))
}
function Assert-SafeOutputPath([string]$Path, [string]$Description) {
    $candidate = Get-ComparablePath $Path
    $root = Get-ComparablePath $Root
    $volume = Get-ComparablePath ([IO.Path]::GetPathRoot($Path))
    $comparison = [StringComparison]::OrdinalIgnoreCase
    if ($candidate -eq $volume) {
        throw "$Description no puede ser la raíz de una unidad: $Path"
    }
    if ($candidate -eq $root -or $root.StartsWith($candidate + [IO.Path]::DirectorySeparatorChar, $comparison)) {
        throw "$Description no puede ser la raíz del proyecto ni una carpeta que la contenga: $Path"
    }
    $cursor = $candidate
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "$Description no puede atravesar un enlace o punto de reanálisis: $cursor"
            }
        }
        $parent = Split-Path -Parent $cursor
        if (-not $parent -or $parent -eq $cursor) { break }
        $cursor = $parent
    }
}
function Assert-DisjointOutputPaths([string]$OutputPath, [string]$PublishPath) {
    $output = Get-ComparablePath $OutputPath
    $publish = Get-ComparablePath $PublishPath
    $separator = [IO.Path]::DirectorySeparatorChar
    $comparison = [StringComparison]::OrdinalIgnoreCase
    if ($output.Equals($publish, $comparison) -or
        $output.StartsWith($publish + $separator, $comparison) -or
        $publish.StartsWith($output + $separator, $comparison)) {
        throw "Output y ReleaseOutput no pueden coincidir ni contenerse entre sí: Output=$OutputPath ReleaseOutput=$PublishPath"
    }
}
function Test-PowerShellSyntax {
    $scriptRoots = @(
        (Join-Path $Root 'scripts'),
        (Join-Path $Root 'windows'),
        (Join-Path $Root 'tests')
    )
    $files = @($scriptRoots | Where-Object { Test-Path -LiteralPath $_ -PathType Container } |
        ForEach-Object { Get-ChildItem -LiteralPath $_ -Recurse -File -Filter '*.ps1' })
    foreach ($file in $files) {
        $tokens = $null
        $parseErrors = $null
        [void][System.Management.Automation.Language.Parser]::ParseFile(
            $file.FullName, [ref]$tokens, [ref]$parseErrors)
        if ($parseErrors.Count -gt 0) {
            $details = ($parseErrors | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
            throw "Sintaxis PowerShell inválida en $($file.FullName):$([Environment]::NewLine)$details"
        }
    }
    Write-Host "Sintaxis PowerShell correcta: $($files.Count) archivo(s)."
}
$Version = ((Select-String -Path $CargoManifest -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value)
if (-not $Version) { throw "No se pudo leer la versión desde rust/Cargo.toml" }
$TargetDir = Join-Path $Root "rust\target\windows"
$env:CARGO_TARGET_DIR = $TargetDir
$CargoReleaseDir = Join-Path $TargetDir "$Target\release"
$Binary = Join-Path $CargoReleaseDir "ltools.exe"
$GuiBinary = Join-Path $CargoReleaseDir "ltools-gui.exe"
$CliBinary = Join-Path $CargoReleaseDir "ltools-cli.exe"
$LivePreviewScript = Join-Path $PSScriptRoot 'live-preview.ps1'
$PackageArch = if ($Target -match '^aarch64') { 'arm64' } elseif ($Target -match '^i686') { 'x86' } else { 'x86_64' }
$StatePath = Join-Path $CargoReleaseDir ".build-state.json"
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$SigningPrivateKeyFile = $null
$SigningPublicKeyFile = $null
$SigningRequired = $true
$PublicKeyEnvironmentActive = $false

function Show-Help {
    @"
Uso: powershell -ExecutionPolicy Bypass -File scripts\build.ps1 [opciones]

Sin opciones abre el menú. Usa -Menu para volver a abrirlo desde una llamada
con parámetros y -Help para mostrar esta ayuda.

  -Force          Fuerza compilación y empaquetado.
  -Clean          Limpia el target Windows antes de compilar.
  -Fast           Release incremental, sin LTO (desarrollo).
  -StrictSecurity Exige cargo-audit, cargo-deny, ShellCheck y actionlint.
  -SecurityReview Exige ShellCheck y actionlint para scripts y workflows.
  -AutoFix       Corrige formato Rust y sugerencias mecánicas de Clippy; luego reescanea.
  -NoTests        Omite cargo test, el smoke y la E2E Windows.
  -NoPackage      Compila pero no crea el ZIP.
  -NoSmoke        Omite solo el smoke Windows posterior a cargo test.
  -NoE2E          Omite solo la E2E Windows posterior a cargo test.
  -NoRun          Alias compatible: omite smoke y E2E, pero conserva cargo test.
  -Component C    Ejecuta solo all|backend|frontend|cli|package|exe.
  -Preview        Abre el preview GUI vigilado en un target debug aislado.
  -TestExisting R Ejecuta smoke y E2E contra un ejecutable ya existente.
  -Target T       Target Windows: x86_64, aarch64 o i686; MSVC o GNU.
  -Output RUTA    Carpeta de salida.
  -ReleaseOutput RUTA
                  Carpeta canónica de publicación (por defecto ..\release).
  -Log FICHERO    Fichero de log; -NoLog desactiva logs.
  -RequireSigning Exige firmas Ed25519 y OpenSSH válidas para release (predeterminado).
  -AllowUnsigned  Excepción explícita para una release local sin firma.
  -NonInteractive No solicita confirmaciones.

Salida: ltools-VERSION-windows-ARQUITECTURA.zip, perfiles exe/CLI y una carpeta
portable. Los artefactos publicables se copian también a release\.
"@
}
function Wait-Menu {
    if ([Console]::IsInputRedirected) { return }
    [void](Read-Host 'Pulsa Enter para volver al menú')
}
function Invoke-MenuTask([string]$Title, [scriptblock]$Task) {
    Write-Host "`n$Title"
    try {
        $global:LASTEXITCODE = 0
        & $Task
        if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) { throw "código de salida $LASTEXITCODE" }
        Write-Host 'Tarea terminada correctamente.'
    } catch {
        Write-Host "La tarea terminó con error: $($_.Exception.Message)" -ForegroundColor Red
    }
    Wait-Menu
}
function Show-Menu {
    if ([Console]::IsInputRedirected) {
        Write-Host 'El menú requiere una terminal interactiva. Usa -Help para las opciones de línea de comandos.'
        return
    }
    $binary = Join-Path $Root "rust\target\windows\$Target\release\ltools.exe"
    $cliBinary = Join-Path $Root "rust\target\windows\$Target\release\ltools-cli.exe"
    $localStaging = Join-Path $Root 'dist\windows'
    $unsignedStaging = Join-Path $Root 'dist\windows-local'
    $unsignedRelease = Join-Path $Root 'dist\windows-local-release'
    $smoke = Join-Path $WindowsDir 'tests\smoke.ps1'
    $e2e = Join-Path $WindowsDir 'tests\e2e.ps1'
    $livePreview = Join-Path $PSScriptRoot 'live-preview.ps1'
    while ($true) {
        Write-Host "`nWinSlim-Tools — desarrollo y distribución"
        Write-Host '  1) Preview y ejecución'
        Write-Host '  2) Pruebas con binarios existentes (sin recompilar)'
        Write-Host '  3) Build y distribución'
        Write-Host '  4) Limpiar artefactos Windows'
        Write-Host '  5) Ayuda de las opciones avanzadas'
        Write-Host '  0) Salir'
        $choice = Read-Host 'Selecciona una opción'
        switch ($choice) {
            '1' {
                while ($true) {
                    Write-Host "`nPreview y ejecución"
                    Write-Host '  1) Abrir la GUI release existente'
                    Write-Host '  2) Mostrar la ayuda CLI existente'
                    Write-Host '  3) Preview GUI vigilado (target debug aislado; Ctrl+C para volver)'
                    Write-Host '  0) Volver'
                    $preview = Read-Host 'Selecciona una opción'
                    if ($preview -eq '0') { break }
                    if ($preview -eq '1') {
                        if (Test-Path -LiteralPath $binary) { Invoke-MenuTask 'Abriendo la GUI' { & $binary } }
                        else { Write-Host 'No existe el ejecutable release. Usa Build → Compilar backend.'; Wait-Menu }
                    } elseif ($preview -eq '2') {
                        if (Test-Path -LiteralPath $cliBinary) { Invoke-MenuTask 'Ayuda CLI' { & $cliBinary --help } }
                        elseif (Test-Path -LiteralPath $binary) {
                            Invoke-MenuTask 'Ayuda CLI' {
                                $previousCliMode = $env:LTOOLS_CLI
                                try {
                                    $env:LTOOLS_CLI = '1'
                                    & $binary --help
                                } finally {
                                    if ($null -eq $previousCliMode) {
                                        Remove-Item Env:LTOOLS_CLI -ErrorAction SilentlyContinue
                                    } else {
                                        $env:LTOOLS_CLI = $previousCliMode
                                    }
                                }
                            }
                        }
                        else { Write-Host 'No hay ejecutable release disponible.'; Wait-Menu }
                    } elseif ($preview -eq '3') {
                        Invoke-MenuTask 'Preview GUI vigilado' { & $livePreview -Target $Target }
                    } else { Write-Host 'Opción no válida.'; Wait-Menu }
                }
            }
            '2' {
                while ($true) {
                    Write-Host "`nPruebas Windows sin recompilar"
                    Write-Host '  1) Smoke del ejecutable'
                    Write-Host '  2) E2E de acciones y detección Windows'
                    Write-Host '  3) Smoke y E2E'
                    Write-Host '  4) Validar sintaxis de scripts PowerShell'
                    Write-Host '  0) Volver'
                    $testChoice = Read-Host 'Selecciona una opción'
                    if ($testChoice -eq '0') { break }
                    if ($testChoice -in @('1', '2', '3') -and -not (Test-Path -LiteralPath $binary)) {
                        Write-Host 'No existe un backend release; estas pruebas no compilan.'
                        Wait-Menu
                        continue
                    }
                    if ($testChoice -in @('1', '3') -and -not (Test-Path -LiteralPath $cliBinary)) {
                        Write-Host 'El smoke también necesita ltools-cli.exe junto al backend GUI.'
                        Wait-Menu
                        continue
                    }
                    if ($testChoice -in @('1', '3')) {
                        Invoke-MenuTask 'Smoke Windows' { & $smoke -Binary $binary -CliBinary $cliBinary -Version $Version }
                    }
                    if ($testChoice -in @('2', '3')) {
                        Invoke-MenuTask 'E2E Windows' { & $e2e -Binary $binary }
                    }
                    if ($testChoice -eq '4') { Invoke-MenuTask 'Sintaxis PowerShell' { Test-PowerShellSyntax } }
                    if ($testChoice -notin @('1', '2', '3', '4')) { Write-Host 'Opción no válida.'; Wait-Menu }
                }
            }
            '3' {
                while ($true) {
                    Write-Host "`nBuild Windows"
                    Write-Host '  1) Compilar solo backend (sin ZIP ni pruebas)'
                    Write-Host '  2) Build portable de desarrollo (perfil rápido, conserva las pruebas)'
                    Write-Host '  3) Build portable completa con smoke y E2E'
                    Write-Host '  4) Build local sin firma (no publicable)'
                    Write-Host '  5) Exigir auditoría estricta de seguridad y continuar la build incremental'
                    Write-Host '  6) Revisar scripts y workflows con ShellCheck, actionlint y zizmor si está disponible'
                    Write-Host '  0) Volver'
                    $buildChoice = Read-Host 'Selecciona una opción'
                    if ($buildChoice -eq '0') { break }
                    switch ($buildChoice) {
                        '1' { Invoke-MenuTask 'Compilando backend Windows' { & $PSCommandPath -Force -NoPackage -NoTests -NoRun } }
                        '2' { Invoke-MenuTask 'Build portable rápida local' { & $PSCommandPath -Force -Fast -AllowUnsigned -Output $unsignedStaging -ReleaseOutput $unsignedRelease } }
                        '3' { Invoke-MenuTask 'Build portable y validaciones' { & $PSCommandPath -Force } }
                        '4' { Invoke-MenuTask 'Build local sin firma' { & $PSCommandPath -Force -AllowUnsigned -Output $unsignedStaging -ReleaseOutput $unsignedRelease } }
                        '5' { Invoke-MenuTask 'Auditoría estricta de dependencias y build incremental' { & $PSCommandPath -StrictSecurity } }
                        '6' { Invoke-MenuTask 'Revisión estática estricta de scripts y workflows' { & $PSCommandPath -SecurityReview } }
                        default { Write-Host 'Opción no válida.'; Wait-Menu }
                    }
                }
            }
            '4' {
                Write-Host "`nTarget de compilación: $TargetDir"
                Write-Host "Staging/publicación local Windows: $localStaging, $unsignedStaging y $unsignedRelease"
                Write-Host '  1) Simular limpieza de target y staging'
                Write-Host '  2) Limpiar solo el target Windows'
                Write-Host '  3) Limpiar target y staging Windows'
                Write-Host '  0) Volver'
                $cleanChoice = Read-Host 'Selecciona una opción'
                if ($cleanChoice -eq '1') {
                    foreach ($path in @($TargetDir, $localStaging, $unsignedStaging, $unsignedRelease)) { if (Test-Path -LiteralPath $path) { Write-Host "Se retiraría: $path" } }
                    Write-Host 'Simulación: no se modificó ningún archivo.'
                    Wait-Menu
                } elseif ($cleanChoice -in @('2', '3')) {
                    $paths = if ($cleanChoice -eq '2') { @($TargetDir) } else { @($TargetDir, $localStaging, $unsignedStaging, $unsignedRelease) }
                    $paths = @($paths | Where-Object { Test-Path -LiteralPath $_ })
                    if ($paths.Count -eq 0) { Write-Host 'No hay salidas para limpiar.'; Wait-Menu; continue }
                    $paths | ForEach-Object { Write-Host "Se retirará: $_" }
                    if ((Read-Host '¿Continuar? Escribe SI para confirmar') -ceq 'SI') {
                        foreach ($path in $paths) {
                            Assert-SafeOutputPath $path 'Limpieza Windows'
                        }
                        foreach ($path in $paths) { Remove-Item -LiteralPath $path -Recurse -Force }
                        Write-Host 'Limpieza terminada.'
                    } else { Write-Host 'Cancelado.' }
                    Wait-Menu
                } elseif ($cleanChoice -ne '0') { Write-Host 'Opción no válida.'; Wait-Menu }
            }
            '5' { Invoke-MenuTask 'Opciones avanzadas del builder' { Show-Help } }
            '0' { return }
            '' { return }
            default { Write-Host 'Opción no válida.'; Wait-Menu }
        }
    }
}
if ($Help) { Show-Help; exit 0 }
if ($Menu -or ($PSBoundParameters.Count -eq 0 -and $args.Count -eq 0)) { Show-Menu; exit 0 }

if ($Target -notmatch '^(x86_64|aarch64|i686)-pc-windows-(msvc|gnu)$') {
    throw "Target Windows no compatible: $Target. Usa x86_64, aarch64 o i686 con msvc o gnu."
}
Assert-SafeOutputPath $OutputDir 'Output'
Assert-SafeOutputPath $PublishDir 'ReleaseOutput'
Assert-SafeOutputPath $TargetDir 'Cargo target Windows'
Assert-DisjointOutputPaths $OutputDir $PublishDir

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
    foreach ($base in @("rust/src", "rust/crates", "rust/tests", "rust/.cargo", "windows", "appimage", "distribution", "tests", "docs", "scripts", ".cargo")) {
        $dir = Join-Path $Root $base
        if (Test-Path -LiteralPath $dir -PathType Container) {
            $files += Get-ChildItem -LiteralPath $dir -Recurse -File -Force
        }
    }
    foreach ($relative in @(
        "rust\Cargo.toml", "rust\Cargo.lock", "rust\build.rs",
        "rust\rust-toolchain", "rust\rust-toolchain.toml",
        ".cargo\config", ".cargo\config.toml", "README.md",
        "deny.toml", "fuzz\Cargo.toml", "fuzz\Cargo.lock", "fuzz\deny.toml"
    )) {
        $path = Join-Path $Root $relative
        if (Test-Path -LiteralPath $path -PathType Leaf) { $files += Get-Item -LiteralPath $path -Force }
    }
    return $files | Where-Object {
        $relative = Get-Relative $_.FullName
        $relative -notmatch '^(rust/target|windows/(target|bin|obj)|dist)/'
    } | Sort-Object FullName -Unique
}
function Get-Signatures {
    $result = [ordered]@{}
    foreach ($file in Get-Inputs) {
        $relative = Get-Relative $file.FullName
        $result[$relative] = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    }
    return $result
}
function Get-MapValue($Map, [string]$Key) {
    return Get-LToolsMapValue $Map $Key
}
function Test-BuildProfileChanged($State, [string]$Profile) {
    if ($null -eq $State) { return $true }
    return (Get-MapValue $State 'profile') -ne $Profile
}
function Invoke-NativeCommand([string]$Executable, [string[]]$Arguments) {
    # Windows PowerShell 5.1 convierte stderr de procesos nativos en
    # ErrorRecord. Cargo escribe su progreso en stderr incluso cuando todo
    # termina correctamente; usar ErrorAction=Stop aquí provocaba falsos
    # fallos durante líneas como «Compiling version_check».
    $previousErrorActionPreference = $ErrorActionPreference
    $previousCargoIncremental = $null
    $restoreCargoIncremental = $false
    if ($Executable -ieq 'cargo') {
        $previousCargoIncremental = [Environment]::GetEnvironmentVariable('CARGO_PROFILE_RELEASE_INCREMENTAL', 'Process')
        if ($null -ne $previousCargoIncremental -and $previousCargoIncremental -cnotin @('true', 'false')) {
            # Cargo solo acepta booleanos minúsculos aquí. CI o shells de
            # Windows pueden heredar 0/1/True/False y hacer fallar incluso
            # fmt/clippy antes de llegar a la compilación.
            [Environment]::SetEnvironmentVariable('CARGO_PROFILE_RELEASE_INCREMENTAL', $null, 'Process')
            $restoreCargoIncremental = $true
            if (-not $script:CargoIncrementalWarningLogged) {
                Write-Log 'AVISO: se ignora temporalmente CARGO_PROFILE_RELEASE_INCREMENTAL heredado con formato no válido para Cargo.'
                $script:CargoIncrementalWarningLogged = $true
            }
        }
    }
    try {
        $ErrorActionPreference = 'Continue'
        & $Executable @Arguments 2>&1 | ForEach-Object { Write-Log ([string]$_) }
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
        if ($restoreCargoIncremental) {
            [Environment]::SetEnvironmentVariable('CARGO_PROFILE_RELEASE_INCREMENTAL', $previousCargoIncremental, 'Process')
        }
    }
    return [int]$exitCode
}
function Invoke-Cargo([string[]]$Arguments) {
    $exitCode = Invoke-NativeCommand 'cargo' $Arguments
    if ($exitCode -ne 0) { throw "cargo terminó con código $exitCode" }
}
function Get-RustSourceHashes {
    $hashes = @{}
    $rustRoot = Join-Path $Root 'rust'
    foreach ($file in Get-ChildItem -LiteralPath $rustRoot -Filter '*.rs' -File -Recurse |
        Where-Object { -not $_.FullName.StartsWith((Join-Path $rustRoot 'target') + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) }) {
        $hashes[$file.FullName] = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    }
    return ,$hashes
}
function Write-RustAutofixChanges([hashtable]$Before) {
    $after = Get-RustSourceHashes
    $paths = @($Before.Keys) + @($after.Keys) | Sort-Object -Unique
    $reported = $false
    foreach ($path in $paths) {
        if ($Before[$path] -ne $after[$path]) {
            $relative = $path.Substring($Root.Length).TrimStart([char[]]@(92, 47))
            Write-Log "[AUTO-FIXED] $relative"
            $reported = $true
        }
    }
    if (-not $reported) { Write-Log '[AUTO-FIX] no persistieron cambios de archivo.' }
}
function Invoke-RustQualityChecks {
    $formatCheck = @('fmt', '--manifest-path', $CargoManifest, '--all', '--', '--check')
    $formatExit = Invoke-NativeCommand 'cargo' $formatCheck
    if ($formatExit -ne 0) {
        if (-not $AutoFix) {
            throw 'rustfmt detectó diferencias; corrígelas o vuelve a ejecutar con -AutoFix para aplicar formato y reescanear.'
        }
        $rustFixBefore = Get-RustSourceHashes
        Invoke-Cargo @('fmt', '--manifest-path', $CargoManifest, '--all')
        Write-Log '[AUTO-FIX] rustfmt aplicó el formato; se repite el escaneo antes de continuar.'
        Write-RustAutofixChanges $rustFixBefore
        Invoke-Cargo $formatCheck
    }

    $clippyCheck = @('clippy', '--all-features', '--locked', '--manifest-path', $CargoManifest, '--workspace', '--all-targets', '--target', $Target, '--', '-D', 'warnings')
    $clippyExit = Invoke-NativeCommand 'cargo' $clippyCheck
    if ($clippyExit -ne 0) {
        if (-not $AutoFix) {
            throw 'Clippy encontró avisos; corrígelos o vuelve a ejecutar con -AutoFix para aplicar solo sugerencias mecánicas y reescanear.'
        }
        $rustFixBefore = Get-RustSourceHashes
        Invoke-Cargo @('clippy', '--fix', '--allow-dirty', '--allow-staged', '--all-features', '--locked', '--manifest-path', $CargoManifest, '--workspace', '--all-targets', '--target', $Target)
        Write-Log '[AUTO-FIX] Clippy aplicó sugerencias mecánicas; se volverán a ejecutar formato y Clippy estricto.'
        Invoke-Cargo @('fmt', '--manifest-path', $CargoManifest, '--all')
        Write-Log '[AUTO-FIX] rustfmt normalizó el resultado de Clippy antes del reescaneo.'
        Write-RustAutofixChanges $rustFixBefore
        Invoke-Cargo $formatCheck
        Invoke-Cargo $clippyCheck
    }
}
function Invoke-RustSecurityAudit {
    $auditAvailable = $null -ne (Get-Command 'cargo-audit' -ErrorAction SilentlyContinue)
    $denyAvailable = $null -ne (Get-Command 'cargo-deny' -ErrorAction SilentlyContinue)
    if ($StrictSecurity -and (-not $auditAvailable -or -not $denyAvailable)) {
        $missing = @()
        if (-not $auditAvailable) { $missing += 'cargo-audit' }
        if (-not $denyAvailable) { $missing += 'cargo-deny' }
        throw "La auditoría estricta requiere las herramientas que faltan: $($missing -join ', ')."
    }
    if (-not $auditAvailable) { Write-Log 'AVISO: cargo-audit no está instalado; se omite la auditoría de vulnerabilidades.' }
    else {
        Invoke-Step 'Actualizando y auditando advisories Rust' {
            Invoke-Cargo @('audit', '--file', (Join-Path $Root 'rust\Cargo.lock'))
            Invoke-Cargo @('audit', '--file', (Join-Path $Root 'fuzz\Cargo.lock'))
        }
    }
    if (-not $denyAvailable) { Write-Log 'AVISO: cargo-deny no está instalado; se omite la revisión de licencias y fuentes.' }
    else {
        Invoke-Step 'Revisando advisories, licencias y fuentes Rust' {
            Invoke-Cargo @('deny', '--manifest-path', $CargoManifest, '--config', (Join-Path $Root 'deny.toml'), 'check')
            Invoke-Cargo @('deny', '--manifest-path', (Join-Path $Root 'fuzz\Cargo.toml'), '--config', (Join-Path $Root 'fuzz\deny.toml'), 'check')
        }
    }
}
function Invoke-StaticSecurityReview {
    $shellCheck = Get-Command 'shellcheck' -ErrorAction SilentlyContinue
    $actionLint = Get-Command 'actionlint' -ErrorAction SilentlyContinue
    $zizmor = Get-Command 'zizmor' -ErrorAction SilentlyContinue
    $requireTools = $SecurityReview -or $StrictSecurity
    $missing = @()
    if (-not $shellCheck) { $missing += 'shellcheck' }
    if (-not $actionLint) { $missing += 'actionlint' }
    if ($requireTools -and $missing.Count -gt 0) {
        throw "La revisión estática solicitada requiere estas herramientas ausentes: $($missing -join ', ')."
    }

    if ($shellCheck) {
        $shellFiles = @()
        $shellFiles += @(Get-ChildItem -LiteralPath $Root -File -Filter '*.sh' -ErrorAction SilentlyContinue)
        foreach ($directory in @('scripts', 'tests', 'appimage')) {
            $path = Join-Path $Root $directory
            if (Test-Path -LiteralPath $path -PathType Container) {
                $shellFiles += @(Get-ChildItem -LiteralPath $path -File -Filter '*.sh' -Recurse)
            }
        }
        if ($shellFiles.Count -gt 0) {
            Invoke-Step 'ShellCheck de scripts Bash' {
                $arguments = @('--severity=error') + @($shellFiles | ForEach-Object { $_.FullName })
                $exitCode = Invoke-NativeCommand 'shellcheck' $arguments
                if ($exitCode -ne 0) { throw "ShellCheck terminó con código $exitCode." }
            }
        } else { Write-Log '[REVIEW][SKIP] No se encontraron scripts Bash.' }
    } else { Write-Log '[REVIEW][SKIP] shellcheck no está instalado; no se considera superado.' }

    if ($actionLint) {
        $workflowDirectory = Join-Path $Root '.github\workflows'
        $workflows = if (Test-Path -LiteralPath $workflowDirectory -PathType Container) {
            @(Get-ChildItem -LiteralPath $workflowDirectory -File | Where-Object { $_.Extension -in @('.yml', '.yaml') })
        } else { @() }
        if ($workflows.Count -gt 0) {
            Invoke-Step 'actionlint de workflows GitHub Actions' {
                $arguments = @('-color') + @($workflows | ForEach-Object { $_.FullName })
                $exitCode = Invoke-NativeCommand 'actionlint' $arguments
                if ($exitCode -ne 0) { throw "actionlint terminó con código $exitCode." }
            }
        } else { Write-Log '[REVIEW][SKIP] No se encontraron workflows GitHub Actions.' }
    } else { Write-Log '[REVIEW][SKIP] actionlint no está instalado; no se considera superado.' }
    if ($zizmor) {
        Invoke-Step 'zizmor de workflows y automatizaciones GitHub' {
            $zizmorInput = Join-Path $Root '.github'
            $hasGitHubToken = $env:GH_TOKEN -or $env:GITHUB_TOKEN -or $env:ZIZMOR_GITHUB_TOKEN
            if ($hasGitHubToken) {
                Write-Log '[REVIEW] zizmor online: se verifican también referencias remotas de acciones.'
                $exitCode = Invoke-NativeCommand 'zizmor' @($zizmorInput)
            } else {
                Write-Log '[REVIEW][PARTIAL] zizmor offline: no se verifican referencias remotas; el workflow de GitHub ejecuta la auditoría online.'
                $exitCode = Invoke-NativeCommand 'zizmor' @('--offline', $zizmorInput)
            }
            if ($exitCode -ne 0) { throw "zizmor terminó con código $exitCode." }
        }
    } else { Write-Log '[REVIEW][SKIP] zizmor no está instalado; GitHub Actions lo ejecuta en el workflow de seguridad.' }
    if ($missing.Count -eq 0) { Write-Log '[REVIEW][PASS] ShellCheck y actionlint superados.' }
}
function Initialize-Signing {
    $configHome = if ($env:LTOOLS_CONFIG_HOME) { $env:LTOOLS_CONFIG_HOME } elseif ($env:XDG_CONFIG_HOME) { $env:XDG_CONFIG_HOME } elseif ($env:USERPROFILE) { Join-Path $env:USERPROFILE '.config' } else { $null }
    if ($env:LTOOLS_SIGNING_PRIVATE_KEY_FILE) { $script:SigningPrivateKeyFile = $env:LTOOLS_SIGNING_PRIVATE_KEY_FILE }
    elseif ($env:LTERMINAL_SIGNING_PRIVATE_KEY_FILE) { $script:SigningPrivateKeyFile = $env:LTERMINAL_SIGNING_PRIVATE_KEY_FILE }
    elseif ($configHome) { $script:SigningPrivateKeyFile = Join-Path $configHome 'lterminal\release-signing-private.pem' }
    if ($env:LTOOLS_UPDATE_PUBLIC_KEY_FILE) { $script:SigningPublicKeyFile = $env:LTOOLS_UPDATE_PUBLIC_KEY_FILE }
    elseif ($env:LTERMINAL_UPDATE_PUBLIC_KEY_FILE) { $script:SigningPublicKeyFile = $env:LTERMINAL_UPDATE_PUBLIC_KEY_FILE }
    elseif ($configHome) { $script:SigningPublicKeyFile = Join-Path $configHome 'lterminal\release-signing-public.hex' }
    # La clave pública se fija en el binario distribuible para permitir que
    # el actualizador verifique releases sin pedir configuración al usuario.
    $publicKeyEnvironment = if (-not [string]::IsNullOrWhiteSpace($env:LTOOLS_UPDATE_PUBLIC_KEY)) { $env:LTOOLS_UPDATE_PUBLIC_KEY } elseif (-not [string]::IsNullOrWhiteSpace($env:LTERMINAL_UPDATE_PUBLIC_KEY)) { $env:LTERMINAL_UPDATE_PUBLIC_KEY } else { $null }
    $script:PublicKeyEnvironmentActive = $null -ne $publicKeyEnvironment
    $publicKeyText = if ($script:PublicKeyEnvironmentActive) { $publicKeyEnvironment } elseif ($script:SigningPublicKeyFile -and (Test-Path -LiteralPath $script:SigningPublicKeyFile -PathType Leaf)) { [IO.File]::ReadAllText($script:SigningPublicKeyFile) } else { $null }
    $canonicalPublicKey = if ($null -ne $publicKeyText) { $publicKeyText -replace '\s', '' } else { $null }
    if (-not [string]::IsNullOrWhiteSpace($canonicalPublicKey)) { $env:LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY = $canonicalPublicKey }
    else { Remove-Item Env:LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY -ErrorAction SilentlyContinue }
    # Las releases son firmadas por defecto. Los switches/variables de
    # exigencia se conservan como compatibilidad explícita; la única vía
    # normal para una build local sin firma es -AllowUnsigned.
    $script:SigningRequired = $true
    if ($RequireSigning -or ($env:LTOOLS_REQUIRE_SIGNING -match '^(1|true|yes)$') -or ($env:LTERMINAL_REQUIRE_SIGNING -match '^(1|true|yes)$') -or ($env:CI -match '^(1|true|yes)$')) {
        $script:SigningRequired = $true
    }
    if ($AllowUnsigned -or ($env:LTOOLS_ALLOW_UNSIGNED -match '^(1|true|yes)$')) { $script:SigningRequired = $false }
    $script:SshSigningConfiguration = Get-LToolsSshSigningConfiguration $Root
    if ($script:SshSigningConfiguration.PrivateKey) { $env:LTOOLS_SSH_SIGNING_KEY_FILE = $script:SshSigningConfiguration.PrivateKey }
    if ($script:SshSigningConfiguration.PublicKey) { $env:LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE = $script:SshSigningConfiguration.PublicKey }
    if ($script:SshSigningConfiguration.Identity) { $env:LTOOLS_SSH_SIGNING_IDENTITY = $script:SshSigningConfiguration.Identity }
}
function Invoke-ReleaseSigning([string]$ReleaseDirectory) {
    $checksums = Join-Path $ReleaseDirectory 'SHA256SUMS.txt'
    $signature = Join-Path $ReleaseDirectory 'SHA256SUMS.txt.sig'
    $sshSignature = Join-Path $ReleaseDirectory 'SHA256SUMS.txt.sshsig'
    Invoke-Step 'Generando SHA256SUMS.txt' {
        $exitCode = Invoke-NativeCommand $Binary @('release-checksums', '--output', $checksums, '--artifacts-dir', $ReleaseDirectory)
        if ($exitCode -ne 0) { throw "no se pudo generar SHA256SUMS.txt (código $exitCode)" }
    }
    $privateAvailable = ($SigningPrivateKeyFile -and (Test-Path -LiteralPath $SigningPrivateKeyFile -PathType Leaf)) -or $env:LTOOLS_SIGNING_PRIVATE_KEY -or $env:LTERMINAL_SIGNING_PRIVATE_KEY
    $publicAvailable = ($SigningPublicKeyFile -and (Test-Path -LiteralPath $SigningPublicKeyFile -PathType Leaf)) -or $PublicKeyEnvironmentActive
    if ($privateAvailable -and $publicAvailable) {
        $signArgs = @('release-signature', '--manifest', $checksums, '--signature', $signature)
        if ($SigningPrivateKeyFile -and (Test-Path -LiteralPath $SigningPrivateKeyFile -PathType Leaf)) { $signArgs += @('--private-key-file', $SigningPrivateKeyFile) }
        if (-not $PublicKeyEnvironmentActive -and
            $SigningPublicKeyFile -and (Test-Path -LiteralPath $SigningPublicKeyFile -PathType Leaf)) {
            $signArgs += @('--public-key-file', $SigningPublicKeyFile)
        }
        Invoke-Step 'Firmando SHA256SUMS.txt con Ed25519' {
            $exitCode = Invoke-NativeCommand $Binary $signArgs
            if ($exitCode -ne 0) { throw "la firma Ed25519 terminó con código $exitCode" }
        }
        Invoke-Step 'Verificando firma Ed25519 de release' {
            $verifyArgs = $signArgs + @('--verify')
            $exitCode = Invoke-NativeCommand $Binary $verifyArgs
            if ($exitCode -ne 0) { throw "la verificación Ed25519 terminó con código $exitCode" }
        }
        Write-Log 'Firma Ed25519 generada y verificada correctamente.'
    } else {
        Remove-Item -LiteralPath $signature -Force -ErrorAction SilentlyContinue
        if ($SigningRequired) { throw "release estricta: faltan las claves Ed25519; se esperaban $SigningPrivateKeyFile y $SigningPublicKeyFile" }
        Write-Log 'AVISO: release local sin firma; se conserva SHA256SUMS.txt y se retira cualquier firma antigua.'
    }
    $sshConfiguration = Get-LToolsSshSigningConfiguration $Root
    if ($sshConfiguration.CanSign -and $sshConfiguration.CanVerify) {
        Invoke-Step 'Firmando SHA256SUMS.txt con la clave SSH de GitHub' {
            Invoke-LToolsSshManifestSigning $checksums $sshSignature $sshConfiguration
        }
        Invoke-Step 'Verificando firma SSH de release' {
            Test-LToolsSshManifestSignature $checksums $sshSignature $sshConfiguration
        }
        Write-Log 'Firma SSH de la release generada y verificada correctamente.'
    } else {
        Remove-Item -LiteralPath $sshSignature -Force -ErrorAction SilentlyContinue
        if ($SigningRequired) { throw 'release estricta: falta una clave SSH de firma, ssh-keygen o user.email; configure LTOOLS_SSH_SIGNING_KEY_FILE / LTOOLS_SSH_SIGNING_IDENTITY.' }
        Write-Log 'AVISO: release local sin firma SSH; no se generó SHA256SUMS.txt.sshsig.'
    }
    Write-Log "Checksums release: $checksums"
    if (Test-Path -LiteralPath $signature -PathType Leaf) { Write-Log "Firma release: $signature" }
    if (Test-Path -LiteralPath $sshSignature -PathType Leaf) { Write-Log "Firma SSH de release: $sshSignature" }
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
function Invoke-ExistingWindowsTests([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "No existe el ejecutable que se quiere probar: $Path"
    }
    $existingCli = Join-Path (Split-Path -Parent $Path) 'ltools-cli.exe'
    $smoke = Join-Path $WindowsDir 'tests\smoke.ps1'
    $e2e = Join-Path $WindowsDir 'tests\e2e.ps1'
    if ((Test-Path -LiteralPath $smoke -PathType Leaf) -and (Test-Path -LiteralPath $existingCli -PathType Leaf)) {
        Invoke-Step 'Smoke Windows sobre ejecutable existente' {
            $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $smoke, '-Binary', $Path, '-CliBinary', $existingCli, '-Version', $Version)
            if ($exitCode -ne 0) { throw "smoke Windows terminó con código $exitCode" }
        }
    } else { Write-Log 'AVISO: no se encontró ltools-cli.exe junto al ejecutable; se omite el smoke de perfiles.' }
    if (Test-Path -LiteralPath $e2e -PathType Leaf) {
        Invoke-Step 'E2E Windows sobre ejecutable existente' {
            $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $e2e, '-Binary', $Path)
            if ($exitCode -ne 0) { throw "E2E Windows terminó con código $exitCode" }
        }
    }
}

switch ($Component) {
    'backend' { $NoPackage = $true; $NoTests = $true; $NoSmoke = $true; $NoE2E = $true; $NoRun = $true }
    'frontend' { $NoPackage = $true; $NoTests = $true; $NoSmoke = $true; $NoE2E = $true; $NoRun = $true }
    'package' { $NoSmoke = $true; $NoE2E = $true; $NoRun = $true }
    'exe' { $NoPackage = $true; $NoTests = $true; $NoSmoke = $true; $NoE2E = $true; $NoRun = $true }
    'all' { }
    'cli' { }
}

Write-Log "WinSlim-Tools Windows build $Version"
Write-Log "Componente: $Component"
Write-Log "Target: $Target"
Write-Log "Perfil: $BuildProfile"
Write-Log "Salida: $OutputDir"
Write-Log "Publicación: $PublishDir"
Initialize-Signing
Write-Log "Firma requerida: $SigningRequired"
Write-Log "Clave privada: $SigningPrivateKeyFile"
Write-Log "Clave pública: $SigningPublicKeyFile"
if ($SshSigningConfiguration.PublicKey) {
    Write-Log "Clave pública SSH: $($SshSigningConfiguration.PublicKey)"
    Write-Log "Identidad de firma SSH: $($SshSigningConfiguration.Identity)"
} else { Write-Log 'Clave pública SSH: no configurada' }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw "No se encontró cargo. Instala Rust mediante rustup." }
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) { throw "No se encontró rustc." }
Invoke-Step 'Validando sintaxis de scripts PowerShell' { Test-PowerShellSyntax }
if ($StrictSecurity) {
    $missingStrictSecurityTools = @()
    foreach ($tool in @('cargo-audit', 'cargo-deny', 'shellcheck', 'actionlint')) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { $missingStrictSecurityTools += $tool }
    }
    if ($missingStrictSecurityTools.Count -gt 0) {
        throw "La revisión estricta no puede empezar; faltan: $($missingStrictSecurityTools -join ', ')."
    }
}
Invoke-StaticSecurityReview
Ensure-Target

if ($Preview) {
    & $LivePreviewScript -Target $Target
    exit $LASTEXITCODE
}
if ($TestExisting) {
    Invoke-ExistingWindowsTests $TestExisting
    exit 0
}
if ($Component -eq 'cli') {
    $cliTargetDir = Join-Path $Root 'rust\target\windows-cli'
    $previousTargetDir = $env:CARGO_TARGET_DIR
    try {
        $env:CARGO_TARGET_DIR = $cliTargetDir
        Invoke-Step 'Compilando perfil CLI Windows aislado' {
            Invoke-Cargo @('build', '--locked', '--manifest-path', $CargoManifest, '--release', '--target', $Target, '--features', 'cli')
        }
    } finally {
        $env:CARGO_TARGET_DIR = $previousTargetDir
    }
    $cliArtifact = Join-Path $cliTargetDir "$Target\release\ltools.exe"
    if (-not (Test-Path -LiteralPath $cliArtifact -PathType Leaf)) { throw "Cargo no generó el perfil CLI: $cliArtifact" }
    $publishedCli = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch-cli.exe"
    Copy-Item -LiteralPath $cliArtifact -Destination $publishedCli -Force
    Write-Log "Perfil CLI generado: $publishedCli"
    exit 0
}

$oldState = $null
if (Test-Path $StatePath) {
    try { $oldState = Get-Content $StatePath -Raw | ConvertFrom-Json } catch { Write-Log "AVISO: estado incremental inválido; se reconstruirá." }
}
$newSignatures = Get-Signatures
$oldSignatures = if ($oldState) { $oldState.files } else { $null }
$impact = Get-LToolsBuildImpact $oldSignatures $newSignatures
$profileChanged = Test-BuildProfileChanged $oldState $BuildProfile
$publicKeyEnvironment = if ($PublicKeyEnvironmentActive) { if (-not [string]::IsNullOrWhiteSpace($env:LTOOLS_UPDATE_PUBLIC_KEY)) { $env:LTOOLS_UPDATE_PUBLIC_KEY } else { $env:LTERMINAL_UPDATE_PUBLIC_KEY } } else { $null }
$publicKeyFingerprint = Get-LToolsPublicKeyFingerprint $SigningPublicKeyFile $publicKeyEnvironment
$sshPublicKeyFingerprint = if ($SshSigningConfiguration.PublicKey -and
    (Test-Path -LiteralPath $SshSigningConfiguration.PublicKey -PathType Leaf)) {
    Get-LToolsPublicKeyFingerprint $SshSigningConfiguration.PublicKey $null
} else { $null }
$previousSigning = Get-MapValue $oldState 'signing'
$signingKeyChanged = Test-LToolsSigningKeyChanged $previousSigning $publicKeyFingerprint
$buildBinaryHashes = Get-LToolsBuildBinaryHashes $Binary $GuiBinary $CliBinary
$previousBuildBinaryHashes = Get-MapValue $oldState 'buildBinaries'
$buildBinariesMatch = Test-LToolsBuildBinariesMatch $previousBuildBinaryHashes $buildBinaryHashes
$buildBinariesTampered = Test-LToolsBuildBinariesTampered $previousBuildBinaryHashes $buildBinaryHashes
$buildBinariesUntrusted = Test-LToolsBuildBinariesUntrusted $previousBuildBinaryHashes $buildBinaryHashes
$forceCleanTarget = $buildBinariesTampered -or $buildBinariesUntrusted
$needCompile = $Force -or $AutoFix -or $Clean -or $profileChanged -or $signingKeyChanged -or
    -not (Test-Path $Binary) -or -not (Test-Path $GuiBinary) -or -not (Test-Path $CliBinary) -or
    $impact.RustCompile.Count -gt 0 -or -not $buildBinariesMatch
if (-not $buildBinariesMatch) { Write-Log 'Hash de un binario compilado ausente o alterado; se recompilarán los perfiles antes de empaquetar.' }
if ($buildBinariesTampered) { Write-Log 'La caché compilada no coincide con su hash anterior; se limpiará el target para que Cargo no reutilice el binario alterado.' }
if ($buildBinariesUntrusted) { Write-Log 'El estado incremental aún no contiene hashes de compilación confiables; se limpiará el target para establecer una base verificada.' }
$existingZip = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch.zip"
$existingCli = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch-cli.exe"
$publishedExe = Join-Path $PublishDir "ltools-$Version-windows-$PackageArch.exe"
$publishedCli = Join-Path $PublishDir "ltools-$Version-windows-$PackageArch-cli.exe"
$publishedZip = Join-Path $PublishDir "ltools-$Version-windows-$PackageArch.zip"
$licenseZipName = 'THIRD-PARTY-LICENSES-windows.zip'
$existingLicenseZip = Join-Path $OutputDir $licenseZipName
$publishedLicenseZip = Join-Path $PublishDir $licenseZipName
$needPackage = $Force -or $needCompile -or $impact.Package.Count -gt 0 -or
    (Get-MapValue $oldState 'packagePending') -eq $true -or
    -not (Test-Path $existingZip) -or -not (Test-Path $existingCli)
$needPackage = $needPackage -or -not (Test-Path $publishedExe) -or -not (Test-Path $publishedCli) -or -not (Test-Path $publishedZip)
$needPackage = $needPackage -or -not (Test-Path $existingLicenseZip) -or -not (Test-Path $publishedLicenseZip)
$distDirectory = Join-Path $Root 'dist'
$artifactHashes = Get-LToolsBuildArtifactHashes $OutputDir $PublishDir $distDirectory $Version $PackageArch
$previousArtifactHashes = Get-MapValue $oldState 'artifacts'
$artifactsMatch = Test-LToolsBuildArtifactsMatch $previousArtifactHashes $artifactHashes $previousSigning `
    $SigningRequired $publicKeyFingerprint $sshPublicKeyFingerprint $SshSigningConfiguration.Identity
if (-not $artifactsMatch) { $needPackage = $true }
if ($SigningRequired -and -not $publicKeyFingerprint) {
    Write-Log 'La clave pública actual no está disponible para validar identidad; se regenerará/verificará la release.'
    $needPackage = $true
} elseif (-not $needPackage -and $SigningRequired) {
    $publishedChecksums = Join-Path $PublishDir 'SHA256SUMS.txt'
    $publishedSignature = Join-Path $PublishDir 'SHA256SUMS.txt.sig'
    $verifyArguments = @('release-signature', '--manifest', $publishedChecksums, '--signature', $publishedSignature, '--verify')
    if (-not $PublicKeyEnvironmentActive -and
        $SigningPublicKeyFile -and (Test-Path -LiteralPath $SigningPublicKeyFile -PathType Leaf)) {
        $verifyArguments += @('--public-key-file', $SigningPublicKeyFile)
    }
    $verifyExitCode = Invoke-NativeCommand $Binary $verifyArguments
    if ($verifyExitCode -ne 0) {
        Write-Log 'La firma de los artefactos actuales no coincide con la clave pública configurada; se volverá a empaquetar.'
        $needPackage = $true
    }
    $publishedSshSignature = Join-Path $PublishDir 'SHA256SUMS.txt.sshsig'
    if (-not $needPackage) {
        if (-not $SshSigningConfiguration.CanVerify -or -not (Test-Path -LiteralPath $publishedSshSignature -PathType Leaf)) {
            Write-Log 'La firma SSH de la release actual falta o no se puede verificar; se volverá a empaquetar.'
            $needPackage = $true
        } else {
            try { Test-LToolsSshManifestSignature $publishedChecksums $publishedSshSignature $SshSigningConfiguration }
            catch { Write-Log "La firma SSH de la release actual no es válida: $($_.Exception.Message)"; $needPackage = $true }
        }
    }
}
$needCargoTests = -not $NoTests -and ($Force -or $AutoFix -or $impact.CargoTests.Count -gt 0)
$needRustReview = $Force -or $AutoFix -or $impact.RustCompile.Count -gt 0
$needSmoke = -not ($NoTests -or $NoRun -or $NoSmoke) -and ($Force -or $AutoFix -or $impact.WindowsSmoke.Count -gt 0)
$needE2E = -not ($NoTests -or $NoRun -or $NoE2E) -and ($Force -or $AutoFix -or $impact.WindowsE2E.Count -gt 0)
$needBuildStateTests = -not $NoTests -and ($Force -or $AutoFix -or $impact.BuildStateTests.Count -gt 0)
$needPublishTests = -not $NoTests -and ($Force -or $AutoFix -or $impact.PublishTests.Count -gt 0)
Write-Log ("Impacto: compilar={0}, revisar-Rust={1}, empaquetar={2}, cargo-test={3}, smoke={4}, E2E={5}, estado-test={6}, publish-test={7}" -f
    $impact.RustCompile.Count, $needRustReview, $impact.Package.Count, $impact.CargoTests.Count,
    $impact.WindowsSmoke.Count, $impact.WindowsE2E.Count, $impact.BuildStateTests.Count, $impact.PublishTests.Count)
Write-Log ("Plan incremental: compilar={0}, revisar-Rust={1}, empaquetar={2}, cargo-test={3}, smoke={4}, E2E={5}, estado-test={6}, publish-test={7}" -f
    $needCompile, $needRustReview, $needPackage, $needCargoTests, $needSmoke, $needE2E, $needBuildStateTests, $needPublishTests)

Invoke-RustSecurityAudit

if ($needRustReview) {
    Invoke-Step 'Validando formato y Clippy estricto de todo el código Rust' { Invoke-RustQualityChecks }
} else { Write-Log '    SKIP: la firma incremental confirma que no cambió código ni configuración Rust.' }

if ($Clean -or $forceCleanTarget) {
    Invoke-Step "Limpiando target Windows" { Remove-Item -LiteralPath $TargetDir -Recurse -Force -ErrorAction SilentlyContinue }
    $needCompile = $true
}
if ($needCompile) {
    $cargoArgs = @('build', '--locked', '--manifest-path', $CargoManifest, '--release', '--target', $Target)
    $cargoProfileVariables = @(
        'CARGO_PROFILE_RELEASE_LTO',
        'CARGO_PROFILE_RELEASE_CODEGEN_UNITS',
        'CARGO_PROFILE_RELEASE_INCREMENTAL'
    )
    $previousCargoProfile = @{}
    foreach ($name in $cargoProfileVariables) {
        $previousCargoProfile[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    try {
        foreach ($name in $cargoProfileVariables) {
            [Environment]::SetEnvironmentVariable($name, $null, 'Process')
        }
        if ($Fast) {
            $env:CARGO_PROFILE_RELEASE_LTO = 'false'
            $env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '256'
            $env:CARGO_PROFILE_RELEASE_INCREMENTAL = 'true'
        }
        Invoke-Step "Compilando backend Rust Windows" { Invoke-Cargo $cargoArgs }
        Copy-Item -LiteralPath $Binary -Destination $GuiBinary -Force
        Invoke-Step "Compilando perfil CLI Rust Windows" { Invoke-Cargo ($cargoArgs + @('--features', 'cli')) }
        Copy-Item -LiteralPath $Binary -Destination $CliBinary -Force
        Copy-Item -LiteralPath $GuiBinary -Destination $Binary -Force
    } finally {
        foreach ($name in $cargoProfileVariables) {
            [Environment]::SetEnvironmentVariable($name, $previousCargoProfile[$name], 'Process')
        }
    }
} else { Write-Log "    SKIP: backend Rust sin cambios relevantes." }

if ($needBuildStateTests) {
    $stateTest = Join-Path $Root 'tests\build-state.ps1'
    if (Test-Path -LiteralPath $stateTest -PathType Leaf) {
        Invoke-Step 'Probando matriz de impacto e integridad incremental' {
            $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $stateTest)
            if ($exitCode -ne 0) { throw "prueba de estado incremental terminó con código $exitCode" }
        }
    }
}
if ($needPublishTests) {
    $publishTest = Join-Path $Root 'tests\build-publish.ps1'
    if (Test-Path -LiteralPath $publishTest -PathType Leaf) {
        Invoke-Step 'Probando la promoción segura de artefactos' {
            $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $publishTest)
            if ($exitCode -ne 0) { throw "prueba de promoción segura terminó con código $exitCode" }
        }
    }
}
if ($needCargoTests) {
    Invoke-Step "Ejecutando tests Rust (todas las features y targets)" { Invoke-Cargo @('test', '--all-features', '--all-targets', '--locked', '--manifest-path', $CargoManifest, '--workspace', '--target', $Target) }
}
if ($Target -match 'windows') {
    $smoke = Join-Path $Root 'windows\tests\smoke.ps1'
    if ($needSmoke -and (Test-Path -LiteralPath $smoke -PathType Leaf)) {
        Invoke-Step "Ejecutando smoke Windows" {
            $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $smoke, '-Binary', $Binary, '-CliBinary', $CliBinary, '-Version', $Version)
            if ($exitCode -ne 0) { throw "smoke Windows terminó con código $exitCode" }
        }
    }
    $e2e = Join-Path $Root 'windows\tests\e2e.ps1'
    if ($needE2E -and (Test-Path -LiteralPath $e2e -PathType Leaf)) {
        Invoke-Step "Ejecutando E2E Windows" {
            $exitCode = Invoke-NativeCommand 'powershell.exe' @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $e2e, '-Binary', $Binary)
            if ($exitCode -ne 0) { throw "E2E Windows terminó con código $exitCode" }
        }
    }
}
if (-not ($needCargoTests -or $needSmoke -or $needE2E -or $needBuildStateTests -or $needPublishTests)) {
    Write-Log '    SKIP: la matriz de impacto no asigna suites Windows a los cambios detectados.'
}

$packageCompleted = $false
if ($needPackage -and -not $NoPackage) {
    if (-not (Test-Path $Binary)) { throw "No existe el ejecutable Windows: $Binary" }
    $localArtifactPrefix = "ltools-$Version-windows-$PackageArch"
    $ExecutableArtifact = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch.exe"
    $CliExecutableArtifact = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch-cli.exe"
    $portable = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch"
    $zip = Join-Path $OutputDir "ltools-$Version-windows-$PackageArch.zip"
    $packageStageDir = Join-Path $OutputDir ".ltools-package-$Stamp-$PID"
    if (Test-Path -LiteralPath $packageStageDir) { throw "Ya existe el staging de este build: $packageStageDir" }
    New-Item -ItemType Directory -Path $packageStageDir | Out-Null
    $portableStage = Join-Path $packageStageDir $localArtifactPrefix
    $stagedExecutable = Join-Path $packageStageDir (Split-Path -Leaf $ExecutableArtifact)
    $stagedCliExecutable = Join-Path $packageStageDir (Split-Path -Leaf $CliExecutableArtifact)
    $stagedZip = Join-Path $packageStageDir (Split-Path -Leaf $zip)
    try {
        New-Item -ItemType Directory -Path $portableStage | Out-Null
        Copy-Item -LiteralPath $Binary -Destination $stagedExecutable
        Copy-Item -LiteralPath $CliBinary -Destination $stagedCliExecutable
        Copy-Item -LiteralPath $Binary -Destination (Join-Path $portableStage 'ltools.exe')
        Copy-Item -LiteralPath $CliBinary -Destination (Join-Path $portableStage 'ltools-cli.exe')
        Copy-Item -LiteralPath (Join-Path $WindowsDir 'ltools.ps1') -Destination $portableStage
        Copy-Item -LiteralPath (Join-Path $WindowsDir 'ltools.cmd') -Destination $portableStage
        Copy-Item -LiteralPath (Join-Path $WindowsDir 'ltools-cli.ps1') -Destination $portableStage
        Copy-Item -LiteralPath (Join-Path $WindowsDir 'ltools-cli.cmd') -Destination $portableStage
        Copy-Item -LiteralPath (Join-Path $Root 'README.md') -Destination $portableStage
        Copy-Item -LiteralPath (Join-Path $Root 'LICENSE') -Destination $portableStage
        New-LToolsThirdPartyLicenseBundle -Destination (Join-Path $portableStage 'THIRD-PARTY-LICENSES') `
            -ManifestPath $CargoManifest -Platform $Target | Out-Null
        $licenseIndex = Join-Path $portableStage 'THIRD-PARTY-LICENSES\INDEX.txt'
        if (-not (Test-Path -LiteralPath $licenseIndex -PathType Leaf) -or
            -not (Select-String -LiteralPath $licenseIndex -SimpleMatch 'ISC' -Quiet) -or
            -not (Select-String -LiteralPath $licenseIndex -SimpleMatch 'CDLA-Permissive-2.0' -Quiet)) {
            throw 'El bundle Windows no incluye el índice y las licencias obligatorias ISC/CDLA.'
        }

        $capabilities = & $Binary capabilities --format json 2>&1
        if ($LASTEXITCODE -ne 0) { throw "No se pudo generar ltools-capabilities.json: $capabilities" }
        $capabilitiesText = @($capabilities) -join [Environment]::NewLine
        Write-Utf8NoBom (Join-Path $portableStage 'ltools-capabilities.json') $capabilitiesText
        Copy-Item -LiteralPath (Join-Path $portableStage 'ltools-capabilities.json') -Destination (Join-Path $portableStage 'ltools-capabilities-windows.json')
        Copy-Item -LiteralPath (Join-Path $portableStage 'ltools-capabilities.json') -Destination (Join-Path $packageStageDir 'ltools-capabilities.json')
        Copy-Item -LiteralPath (Join-Path $portableStage 'ltools-capabilities-windows.json') -Destination (Join-Path $packageStageDir 'ltools-capabilities-windows.json')
        $terminalDescriptor = & $Binary capabilities --format terminal-json 2>&1
        if ($LASTEXITCODE -ne 0) { throw "No se pudo generar ltools-terminal.json: $terminalDescriptor" }
        $terminalDescriptorText = @($terminalDescriptor) -join [Environment]::NewLine
        Write-Utf8NoBom (Join-Path $portableStage 'ltools-terminal.json') $terminalDescriptorText
        Copy-Item -LiteralPath (Join-Path $portableStage 'ltools-terminal.json') -Destination (Join-Path $portableStage 'ltools-terminal-windows.json')
        Copy-Item -LiteralPath (Join-Path $portableStage 'ltools-terminal.json') -Destination (Join-Path $packageStageDir 'ltools-terminal.json')
        Copy-Item -LiteralPath (Join-Path $portableStage 'ltools-terminal-windows.json') -Destination (Join-Path $packageStageDir 'ltools-terminal-windows.json')
        Copy-Item -LiteralPath (Join-Path $Root 'appimage\ltools-capabilities.schema.json') -Destination $portableStage
        Copy-Item -LiteralPath (Join-Path $Root 'appimage\ltools-terminal.schema.json') -Destination $portableStage
        Copy-Item -LiteralPath (Join-Path $Root 'appimage\ltools-capabilities.schema.json') -Destination $packageStageDir
        Copy-Item -LiteralPath (Join-Path $Root 'appimage\ltools-terminal.schema.json') -Destination $packageStageDir
        @("WinSlim-Tools $Version", "Platform: Windows", "Target: $Target", "Backend: ltools.exe", "CLI backend: ltools-cli.exe (no arguments prints help)", "Linux-only Bash modules and AppImage assets are not included.") |
            Set-Content -Encoding UTF8 (Join-Path $portableStage 'BUILD-INFO.txt')

        $capabilitiesJson = Get-Content -Raw -LiteralPath (Join-Path $portableStage 'ltools-capabilities.json') | ConvertFrom-Json
        if ($capabilitiesJson.schema -ne 'ltools-capabilities-v1' -or $capabilitiesJson.platform -ne 'windows') {
            throw 'El descriptor JSON de capacidades Windows no es válido.'
        }
        $terminalJson = Get-Content -Raw -LiteralPath (Join-Path $portableStage 'ltools-terminal.json') | ConvertFrom-Json
        if ($terminalJson.schema -ne 'ltools-terminal-integration-v1' -or
            $terminalJson.platform -ne 'windows' -or
            $terminalJson.entrypoint.command -ne 'ltools.exe' -or
            $terminalJson.integration.optional -ne $true -or
            $terminalJson.integration.standalone_releases_require_it -ne $false -or
            $terminalJson.integration.exclusive_host_family -ne 'lterminal' -or
            $terminalJson.host.product -ne 'WinSlim Terminal') {
            throw 'El descriptor JSON de integración Windows no declara WinSlim Terminal correctamente.'
        }

        Invoke-Step 'Empaquetando ZIP portable Windows en staging' {
            Compress-Archive -Path (Join-Path $portableStage '*') -DestinationPath $stagedZip -CompressionLevel Optimal
            Compress-Archive -Path @((Join-Path $portableStage 'LICENSE'), (Join-Path $portableStage 'THIRD-PARTY-LICENSES')) `
                -DestinationPath (Join-Path $packageStageDir $licenseZipName) -CompressionLevel Optimal
        }
        if (-not (Test-Path -LiteralPath $stagedZip -PathType Leaf) -or (Get-Item -LiteralPath $stagedZip).Length -eq 0) {
            throw 'El ZIP portable Windows no existe o está vacío.'
        }
        $archiveCheckDir = Join-Path $packageStageDir 'zip-extracted'
        Expand-Archive -LiteralPath $stagedZip -DestinationPath $archiveCheckDir
        foreach ($requiredFile in @(
            'ltools.exe', 'ltools-cli.exe', 'ltools.ps1', 'ltools.cmd', 'ltools-cli.ps1', 'ltools-cli.cmd',
            'ltools-capabilities.json', 'ltools-capabilities-windows.json',
            'ltools-terminal.json', 'ltools-terminal-windows.json',
            'ltools-capabilities.schema.json', 'ltools-terminal.schema.json', 'README.md', 'LICENSE', 'BUILD-INFO.txt',
            'THIRD-PARTY-LICENSES\INDEX.txt'
        )) {
            if (-not (Test-Path -LiteralPath (Join-Path $archiveCheckDir $requiredFile) -PathType Leaf)) {
                throw "El ZIP portable omite $requiredFile"
            }
        }
        $licenseArchiveCheckDir = Join-Path $packageStageDir 'license-zip-extracted'
        Expand-Archive -LiteralPath (Join-Path $packageStageDir $licenseZipName) -DestinationPath $licenseArchiveCheckDir
        $licenseArchiveIndex = Join-Path $licenseArchiveCheckDir 'THIRD-PARTY-LICENSES\INDEX.txt'
        $licenseArchiveProject = Join-Path $licenseArchiveCheckDir 'LICENSE'
        if (-not (Test-Path -LiteralPath $licenseArchiveIndex -PathType Leaf) -or
            -not (Test-Path -LiteralPath $licenseArchiveProject -PathType Leaf) -or
            -not (Select-String -LiteralPath $licenseArchiveProject -SimpleMatch 'MIT License' -Quiet) -or
            -not (Select-String -LiteralPath $licenseArchiveIndex -SimpleMatch 'ISC' -Quiet) -or
            -not (Select-String -LiteralPath $licenseArchiveIndex -SimpleMatch 'CDLA-Permissive-2.0' -Quiet)) {
            throw 'El ZIP de licencias independiente está incompleto o dañado.'
        }
        foreach ($pair in @(
            @((Join-Path $archiveCheckDir 'ltools.exe'), $Binary),
            @((Join-Path $archiveCheckDir 'ltools-cli.exe'), $CliBinary)
        )) {
            if ((Get-FileHash -LiteralPath $pair[0] -Algorithm SHA256).Hash -ne
                (Get-FileHash -LiteralPath $pair[1] -Algorithm SHA256).Hash) {
                throw "El ZIP contiene un perfil distinto del binario probado: $($pair[0])"
            }
        }
        $nativeProcessHelper = Join-Path $WindowsDir 'tests\native-process.ps1'
        . $nativeProcessHelper
        $packageCli = Join-Path $archiveCheckDir 'ltools-cli.exe'
        $packageGui = Join-Path $archiveCheckDir 'ltools.exe'
        $packageVersion = Invoke-NativeProcess -FileName $packageCli -Arguments @('--version') -TimeoutSeconds 30
        if ($packageVersion.ExitCode -ne 0 -or $packageVersion.Stdout -notmatch [regex]::Escape($Version)) {
            throw (Format-NativeProcessFailure $packageVersion 'Perfil CLI extraído del ZIP')
        }
        $packageHelp = Invoke-NativeProcess -FileName $packageCli -EnvironmentOverrides @{ LTOOLS_LANG = 'es' } -TimeoutSeconds 30
        if ($packageHelp.ExitCode -ne 0 -or $packageHelp.Stdout -notmatch '(?im)^\s*(Uso|Usage):\s*ltools\b') {
            throw (Format-NativeProcessFailure $packageHelp 'Ayuda del perfil CLI extraído del ZIP')
        }
        $packageGuiVersion = Invoke-NativeProcess -FileName $packageGui -Arguments @('--version') -TimeoutSeconds 30
        if ($packageGuiVersion.ExitCode -ne 0 -or $packageGuiVersion.Stdout -notmatch [regex]::Escape($Version)) {
            throw (Format-NativeProcessFailure $packageGuiVersion 'Perfil GUI extraído del ZIP')
        }
        Write-Log 'ZIP portable validado: inventario, hashes y perfiles extraídos ejecutables.'

        Publish-StagedDirectory $portableStage $portable
        foreach ($stagedFile in Get-ChildItem -LiteralPath $packageStageDir -File | Sort-Object Name) {
            Publish-StagedFile $stagedFile.FullName (Join-Path $OutputDir $stagedFile.Name)
        }
        Write-Log "Ejecutable Windows: $ExecutableArtifact"
    } finally {
        Remove-Item -LiteralPath $packageStageDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    # La release completa se construye en un hermano temporal. Los artefactos
    # existentes (incluidos los Linux y los ficheros ajenos) siguen intactos
    # hasta que manifiesto, checksums, firma y E2E hayan pasado.
    $releaseStageDir = New-StagedDirectory $PublishDir
    try {
        Get-ChildItem -LiteralPath $releaseStageDir -File -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -like "ltools-$Version-windows-$PackageArch*" } |
            Remove-Item -Force
        foreach ($file in @(
            $ExecutableArtifact,
            $CliExecutableArtifact,
            $zip,
            (Join-Path $packageStageDir $licenseZipName),
            (Join-Path $OutputDir 'ltools-capabilities.json'),
            (Join-Path $OutputDir 'ltools-capabilities-windows.json'),
            (Join-Path $OutputDir 'ltools-terminal.json'),
            (Join-Path $OutputDir 'ltools-terminal-windows.json'),
            (Join-Path $OutputDir 'ltools-capabilities.schema.json'),
            (Join-Path $OutputDir 'ltools-terminal.schema.json')
        )) {
            if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Falta un fichero requerido para la release: $file" }
            Copy-Item -LiteralPath $file -Destination $releaseStageDir -Force
        }
        foreach ($file in @(
            (Join-Path $Root 'LICENSE'),
            (Join-Path $Root 'distribution\ltools-project.json'),
            (Join-Path $Root 'distribution\ltools-project.schema.json'),
            (Join-Path $Root 'distribution\ltools-release.schema.json')
        )) {
            Copy-Item -LiteralPath $file -Destination $releaseStageDir -Force
        }

        $releaseManifestOutput = Join-Path $releaseStageDir 'ltools-release.json'
        $releaseRepository = if ($env:LTOOLS_GITHUB_REPOSITORY) { $env:LTOOLS_GITHUB_REPOSITORY } else { 'Darkeiser003/Tools' }
        $releaseTag = if ($env:LTOOLS_GITHUB_TAG) { $env:LTOOLS_GITHUB_TAG } else { "v$Version" }
        Invoke-Step 'Generando manifiesto verificable de release en staging' {
            & $Binary release-manifest --output $releaseManifestOutput --repository $releaseRepository --tag $releaseTag --artifacts-dir $releaseStageDir 2>&1 |
                ForEach-Object { Write-Log ([string]$_) }
            if ($LASTEXITCODE -ne 0) { throw 'no se pudo generar ltools-release.json' }
        }
        $manifest = Get-Content -Raw -LiteralPath $releaseManifestOutput | ConvertFrom-Json
        if ($manifest.schema -ne 'ltools-release-v1' -or $manifest.application -notin @('LTools', 'WinSlim-Tools') -or
            $manifest.version -ne $Version -or $manifest.hash_algorithm -ne 'sha256' -or @($manifest.artifacts).Count -lt 1) {
            throw 'El manifiesto de release Windows no supera la validación estructural.'
        }
        Write-Log "Manifiesto de release validado: $releaseManifestOutput"
        Invoke-ReleaseSigning $releaseStageDir

        $releaseE2E = Join-Path $WindowsDir 'tests\release-e2e.ps1'
        Invoke-Step 'Validando manifiesto, checksums y firma de release' {
            $arguments = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $releaseE2E,
                '-ReleaseDirectory', $releaseStageDir, '-Version', $Version, '-Architecture', $PackageArch, '-Binary', $Binary)
            if ($SigningRequired) { $arguments += @('-RequireSignature', '-RequireSshSignature') }
            if ($SshSigningConfiguration.PublicKey -and (Test-Path -LiteralPath $SshSigningConfiguration.PublicKey -PathType Leaf)) {
                $arguments += @('-SshPublicKeyFile', $SshSigningConfiguration.PublicKey)
            }
            if ($SshSigningConfiguration.Identity) { $arguments += @('-SshIdentity', $SshSigningConfiguration.Identity) }
            if (-not $PublicKeyEnvironmentActive -and
                $SigningPublicKeyFile -and (Test-Path -LiteralPath $SigningPublicKeyFile -PathType Leaf)) {
                $arguments += @('-PublicKeyFile', $SigningPublicKeyFile)
            }
            $exitCode = Invoke-NativeCommand 'powershell.exe' $arguments
            if ($exitCode -ne 0) { throw "E2E release Windows terminó con código $exitCode" }
        }

        Publish-StagedDirectory $releaseStageDir $PublishDir
        $distribution = Join-Path $Root 'distribution'
        New-Item -ItemType Directory -Force -Path (Join-Path $Root 'dist') | Out-Null
        foreach ($file in @(
            'ltools-release.json', 'ltools-project.json', 'ltools-project.schema.json',
            'ltools-release.schema.json', 'SHA256SUMS.txt', 'SHA256SUMS.txt.sig', 'SHA256SUMS.txt.sshsig'
        )) {
            $publishedFile = Join-Path $PublishDir $file
            $distFile = Join-Path $Root "dist\$file"
            if (Test-Path -LiteralPath $publishedFile -PathType Leaf) { Copy-Item -LiteralPath $publishedFile -Destination $distFile -Force }
            elseif ($file -in @('SHA256SUMS.txt.sig', 'SHA256SUMS.txt.sshsig')) { Remove-Item -LiteralPath $distFile -Force -ErrorAction SilentlyContinue }
        }
        Write-Log "Release verificada y publicada: $PublishDir"
        $packageCompleted = $true
    } finally {
        if (Test-Path -LiteralPath $releaseStageDir -PathType Container) {
            Remove-Item -LiteralPath $releaseStageDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
} elseif (-not $NoPackage) { Write-Log "    SKIP: paquete Windows ya actualizado." }

if ($packageCompleted) {
    $artifactHashes = Get-LToolsBuildArtifactHashes $OutputDir $PublishDir $distDirectory $Version $PackageArch
    $signingState = [ordered]@{
        required = [bool]$SigningRequired
        publicKeyFingerprint = $publicKeyFingerprint
        signed = Test-Path -LiteralPath (Join-Path $PublishDir 'SHA256SUMS.txt.sig') -PathType Leaf
        sshPublicKeyFingerprint = $sshPublicKeyFingerprint
        sshIdentity = $SshSigningConfiguration.Identity
        sshSigned = Test-Path -LiteralPath (Join-Path $PublishDir 'SHA256SUMS.txt.sshsig') -PathType Leaf
    }
    $packagePending = $false
} elseif ($needPackage) {
    # -NoPackage solo aplaza el empaquetado; no registra los artefactos viejos
    # como si correspondieran a las fuentes nuevas.
    $artifactHashes = $previousArtifactHashes
    $signingState = $previousSigning
    $packagePending = Get-LToolsPackagePending $needPackage $packageCompleted
    Write-Log 'Paquete pendiente: la siguiente ejecución sin -NoPackage volverá a empaquetar.'
} else {
    $signingState = [ordered]@{
        required = [bool]$SigningRequired
        publicKeyFingerprint = $publicKeyFingerprint
        signed = Test-Path -LiteralPath (Join-Path $PublishDir 'SHA256SUMS.txt.sig') -PathType Leaf
        sshPublicKeyFingerprint = $sshPublicKeyFingerprint
        sshIdentity = $SshSigningConfiguration.Identity
        sshSigned = Test-Path -LiteralPath (Join-Path $PublishDir 'SHA256SUMS.txt.sshsig') -PathType Leaf
    }
    $packagePending = $false
}
$state = [ordered]@{
    version = $Version
    target = $Target
    profile = $BuildProfile
    builtAt = (Get-Date).ToUniversalTime().ToString('o')
    files = $newSignatures
    buildBinaries = Get-LToolsBuildBinaryHashes $Binary $GuiBinary $CliBinary
    artifacts = $artifactHashes
    signing = $signingState
    packagePending = [bool]$packagePending
}
$stateTemp = "$StatePath.$PID.tmp"
$state | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $stateTemp
Publish-StagedFile $stateTemp $StatePath
if ($TimingPath) {
    $total = [math]::Round($script:BuildStart.Elapsed.TotalSeconds, 3)
    ( "total" + [char]9 + $total + [char]9 + "ok" ) | Add-Content -Encoding UTF8 $TimingPath
    Write-Log "Tiempo total: $total s"
}
Write-Log "Build Windows terminada correctamente."
