[CmdletBinding()]
param(
    [string]$Target = 'x86_64-pc-windows-msvc',
    [switch]$Help
)

$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$manifest = Join-Path $root 'rust\Cargo.toml'
$previewTarget = Join-Path $root 'rust\target\live-preview-windows'
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if ($Help) {
    Write-Output 'Uso: powershell -File scripts\live-preview.ps1 [-Target x86_64-pc-windows-msvc]'
    Write-Output 'Compila en rust\target\live-preview-windows, reabre la GUI al cambiar Rust/Cargo y no empaqueta releases. Ctrl+C detiene el vigilante.'
    exit 0
}
if (-not $cargo) { throw 'No se encontró cargo.' }
if ($Target -notmatch '^(x86_64|aarch64|i686)-pc-windows-(msvc|gnu)$') { throw "Target Windows no compatible: $Target" }

function Get-PreviewFingerprint {
    $roots = @(
        (Join-Path $root 'rust\src'),
        (Join-Path $root 'rust\crates'),
        (Join-Path $root 'rust\.cargo')
    ) | Where-Object { Test-Path -LiteralPath $_ -PathType Container }
    $files = @($roots | ForEach-Object { Get-ChildItem -LiteralPath $_ -Recurse -File -Force })
    foreach ($name in @('Cargo.toml', 'Cargo.lock', 'build.rs', 'rust-toolchain', 'rust-toolchain.toml')) {
        $path = Join-Path (Join-Path $root 'rust') $name
        if (Test-Path -LiteralPath $path -PathType Leaf) { $files += Get-Item -LiteralPath $path -Force }
    }
    $lines = @($files | Sort-Object FullName -Unique | ForEach-Object {
        "$($_.FullName)=$((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash)"
    })
    $joined = [Text.Encoding]::UTF8.GetBytes(($lines -join "`n"))
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($joined))).Replace('-', '') }
    finally { $sha.Dispose() }
}

$binary = Join-Path (Join-Path (Join-Path $previewTarget $Target) 'debug') 'ltools.exe'
$previousTargetDir = [Environment]::GetEnvironmentVariable('CARGO_TARGET_DIR', 'Process')
$previewProcess = $null
try {
    [Environment]::SetEnvironmentVariable('CARGO_TARGET_DIR', $previewTarget, 'Process')
    Write-Host "Preview GUI vigilado (target aislado: $previewTarget). Ctrl+C para terminar."
    $lastFingerprint = ''
    while ($true) {
        $before = Get-PreviewFingerprint
        if ($before -ne $lastFingerprint -or -not (Test-Path -LiteralPath $binary -PathType Leaf)) {
            if ($previewProcess -and -not $previewProcess.HasExited) {
                Stop-Process -Id $previewProcess.Id -Force -ErrorAction SilentlyContinue
                $previewProcess.WaitForExit()
            }
            $previewProcess = $null
            Write-Host "`nCambios detectados; compilando perfil debug aislado..."
            & $cargo.Source build --manifest-path $manifest --target $Target
            $exitCode = $LASTEXITCODE
            if ($exitCode -ne 0) {
                Write-Warning 'Falló la compilación. Se volverá a intentar al cambiar un archivo vigilado.'
                $lastFingerprint = Get-PreviewFingerprint
                Start-Sleep -Seconds 1
                continue
            }
            $after = Get-PreviewFingerprint
            if ($before -ne $after) { $lastFingerprint = ''; continue }
            $lastFingerprint = $after
            $previewProcess = Start-Process -FilePath $binary -PassThru
            Write-Host "Preview abierto (PID $($previewProcess.Id))."
        }

        Start-Sleep -Seconds 1
        $current = Get-PreviewFingerprint
        if ($current -ne $lastFingerprint) {
            Write-Host "`nCambio detectado; cerrando solo el proceso preview."
            if ($previewProcess -and -not $previewProcess.HasExited) {
                Stop-Process -Id $previewProcess.Id -Force -ErrorAction SilentlyContinue
                $previewProcess.WaitForExit()
            }
            $previewProcess = $null
            $lastFingerprint = ''
        } elseif ($previewProcess -and $previewProcess.HasExited) {
            Write-Host 'La ventana se cerró; el vigilante continúa esperando cambios.'
            $previewProcess = $null
        }
    }
} finally {
    if ($previewProcess -and -not $previewProcess.HasExited) {
        Stop-Process -Id $previewProcess.Id -Force -ErrorAction SilentlyContinue
    }
    [Environment]::SetEnvironmentVariable('CARGO_TARGET_DIR', $previousTargetDir, 'Process')
}
