# E2E Windows aislada: migración nativa, verificación de contenido y listado.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Binary)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Binary)) { throw "No existe el ejecutable: $Binary" }
$temp = Join-Path ([IO.Path]::GetTempPath()) ("ltools-windows-e2e-" + [guid]::NewGuid().ToString('N'))
$source = Join-Path $temp 'source-prefix'
$destination = Join-Path $temp 'destination-prefix'
$plan = Join-Path $temp 'migration-plan.tsv'
New-Item -ItemType Directory -Force -Path (Join-Path $source 'drive_c\users\test') | Out-Null
Set-Content -Encoding UTF8 (Join-Path $source 'system.reg') 'synthetic-system'
Set-Content -Encoding UTF8 (Join-Path $source 'user.reg') 'synthetic-user'
Set-Content -Encoding UTF8 (Join-Path $source 'drive_c\users\test\marker.txt') 'native-copy'
$oldUserProfile = $env:USERPROFILE
$oldHome = $env:HOME
$oldAppData = $env:APPDATA
$oldLocalAppData = $env:LOCALAPPDATA
$env:USERPROFILE = $temp
$env:HOME = $temp
$env:APPDATA = Join-Path $temp 'AppData\Roaming'
$env:LOCALAPPDATA = Join-Path $temp 'AppData\Local'

function Quote-Arg([string]$Value) {
    return '"' + $Value.Replace('"', '\"') + '"'
}
function Run-WithInput([string[]]$Arguments, [string]$InputText) {
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $Binary
    $info.Arguments = (($Arguments | ForEach-Object { Quote-Arg $_ }) -join ' ')
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $info
    [void]$process.Start()
    $process.StandardInput.Write($InputText)
    $process.StandardInput.Close()
    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw "Falló ltools.exe E2E: $stdout $stderr" }
    Write-Host $stdout.Trim()
}

try {
    Run-WithInput @() ("q" + [Environment]::NewLine)
    Run-WithInput @('prefix', 'migrate', '--source', $source, '--dest', $destination, '--plan', $plan) ("y" + [Environment]::NewLine)
    if (-not (Test-Path $plan)) { throw 'La migración no generó el plan reversible.' }
    foreach ($relative in @('system.reg', 'user.reg', 'drive_c\users\test\marker.txt')) {
        $left = Get-Content -Raw (Join-Path $source $relative)
        $right = Get-Content -Raw (Join-Path $destination $relative)
        if ($left -ne $right) { throw "Contenido distinto tras migrar $relative" }
    }
    Run-WithInput @('rollback', '--plan', $plan) ("y" + [Environment]::NewLine)
    if (-not (Test-Path (Join-Path $source 'system.reg'))) { throw 'El rollback Windows no conservó el origen.' }
    if (Test-Path $destination) { throw 'El rollback Windows no retiró el destino a la papelera.' }
    $output = & $Binary 'prefix' 'list' '--root' $temp 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $output -notmatch 'Prefijos detectados') { throw 'El listado de prefijos Windows falló.' }
    $capabilities = & $Binary 'capabilities' '--format' 'json' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $capabilities -notmatch 'ltools-capabilities-v1') { throw 'El contrato JSON Windows falló.' }
    Run-WithInput @('system', '--dry-run', 'service', 'restart', 'EventLog') ("y" + [Environment]::NewLine)
    $processOutput = & $Binary 'system' 'processes' '--sort' 'memory' '--limit' '3' 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0 -or $processOutput -notmatch 'Procesos Windows') { throw 'El listado de procesos Windows falló.' }
    Write-Host 'Windows E2E completado correctamente.'
} finally {
    $env:USERPROFILE = $oldUserProfile
    $env:HOME = $oldHome
    $env:APPDATA = $oldAppData
    $env:LOCALAPPDATA = $oldLocalAppData
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
