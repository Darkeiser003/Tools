# Captura común de procesos nativos para las pruebas Windows.
# PowerShell 5.1 mezcla stderr con ErrorRecord y usa la página de códigos de
# la consola al invocar EXE con &, por lo que no sirve para validar JSON o
# traducciones UTF-8 de forma fiable. Este helper usa .NET directamente.

function Quote-NativeProcessArgument([string]$Value) {
    if ($null -eq $Value -or $Value.Length -eq 0) { return '""' }
    if ($Value -notmatch '[\s"]') { return $Value }
    # Regla de CommandLineToArgvW: las barras antes de comillas y al final
    # deben duplicarse para que el EXE reciba exactamente el argumento.
    $escaped = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $escaped = [regex]::Replace($escaped, '(\\+)$', '$1$1')
    return '"' + $escaped + '"'
}

function Invoke-NativeProcess {
    param(
        [Parameter(Mandatory = $true)][string]$FileName,
        [string[]]$Arguments = @(),
        [string]$InputText = $null,
        [hashtable]$EnvironmentOverrides = @{},
        [ValidateRange(1, 3600)][int]$TimeoutSeconds = 120
    )
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $FileName
    $info.Arguments = (($Arguments | ForEach-Object { Quote-NativeProcessArgument ([string]$_) }) -join ' ')
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    # Estas propiedades existen en .NET moderno y en Windows PowerShell 5.1
    # con .NET 4.5+. Si un host antiguo no las expone, el proceso sigue
    # funcionando y el diagnóstico indicará la página de códigos disponible.
    try {
        $info.StandardOutputEncoding = [Text.Encoding]::UTF8
        $info.StandardErrorEncoding = [Text.Encoding]::UTF8
    } catch { }
    foreach ($key in $EnvironmentOverrides.Keys) {
        $value = $EnvironmentOverrides[$key]
        if ($null -eq $value) {
            [void]$info.EnvironmentVariables.Remove($key)
        } else {
            $info.EnvironmentVariables[$key] = [string]$value
        }
    }

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $info
    $startedAt = [Diagnostics.Stopwatch]::StartNew()
    try {
        try {
            [void]$process.Start()
        } catch {
            throw "No se pudo iniciar el proceso: $FileName $($info.Arguments)`n$($_.Exception.Message)"
        }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if ($null -ne $InputText) {
            $process.StandardInput.Write($InputText)
        }
        $process.StandardInput.Close()
        if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
            try { $process.Kill() } catch { }
            try { [void]$process.WaitForExit(2000) } catch { }
            $partialStdout = if ($stdoutTask.IsCompleted) { [string]$stdoutTask.Result } else { '' }
            $partialStderr = if ($stderrTask.IsCompleted) { [string]$stderrTask.Result } else { '' }
            throw "Tiempo de espera agotado (${TimeoutSeconds}s): $FileName $($info.Arguments)`nSalida parcial:`n$partialStdout`n$partialStderr"
        }
        # WaitForExit() adicional garantiza que las tareas asíncronas hayan
        # vaciado sus buffers antes de leer Result y evita perder el final.
        # El límite adicional evita que un hijo defectuoso deje una pipe abierta
        # después de que el proceso principal ya haya terminado.
        $process.WaitForExit()
        if (-not $stdoutTask.Wait(5000) -or -not $stderrTask.Wait(5000)) {
            throw "La captura de salida no terminó después de que finalizara el proceso: $FileName $($info.Arguments)"
        }
        $stdout = [string]$stdoutTask.Result
        $stderr = [string]$stderrTask.Result
        [pscustomobject]@{
            ExitCode = [int]$process.ExitCode
            Stdout = $stdout
            Stderr = $stderr
            DurationMs = [int]$startedAt.ElapsedMilliseconds
            CommandLine = "$FileName $($info.Arguments)"
        }
    } finally {
        $startedAt.Stop()
        $process.Dispose()
    }
}

function Format-NativeProcessFailure($Result, [string]$Context) {
    $text = @(
        "$Context terminó con código $($Result.ExitCode)."
        "Comando: $($Result.CommandLine)"
        "Salida estándar:"
        $Result.Stdout
        "Salida de error:"
        $Result.Stderr
    ) -join [Environment]::NewLine
    return $text.Trim()
}
