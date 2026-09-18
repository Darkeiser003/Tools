# E2E Windows aislada: inventario nativo y operaciones del sistema.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Binary)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) { throw "No existe el ejecutable: $Binary" }
$temp = Join-Path ([IO.Path]::GetTempPath()) ("ltools-windows-e2e-" + [guid]::NewGuid().ToString('N'))
$fixtureRoot = Join-Path $temp 'fixture con espacios y ñ'
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Epic Games\ExampleGame') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Rockstar Games\ExampleGame') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Steam\steamapps') | Out-Null
Set-Content -Encoding UTF8 (Join-Path $fixtureRoot 'Steam\steamapps\appmanifest_123.acf') '"name" "Native Example"'
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot '.wine\drive_c') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'Lutris\games') | Out-Null
$reparseTarget = Join-Path $fixtureRoot 'junction-target'
$junctionPath = Join-Path $fixtureRoot 'junction-to-target'
New-Item -ItemType Directory -Force -Path $reparseTarget | Out-Null
Set-Content -Encoding UTF8 (Join-Path $reparseTarget 'keep.txt') 'junction target data'
$junctionCreated = $false
try {
    New-Item -ItemType Junction -Path $junctionPath -Target $reparseTarget -ErrorAction Stop | Out-Null
    $junctionCreated = $true
} catch {
    Write-Host '  [SKIP] no se pudo crear una junction para la regresión de reanálisis.'
}
$oldUserProfile = $env:USERPROFILE
$oldHome = $env:HOME
$oldAppData = $env:APPDATA
$oldLocalAppData = $env:LOCALAPPDATA
$oldTemp = $env:TEMP
$oldSystemRoot = $env:SystemRoot
$oldSystemDrive = $env:SystemDrive
$oldLanguage = $env:LTOOLS_LANG
$oldCliMode = $env:LTOOLS_CLI
$oldNoClear = $env:LTOOLS_NO_CLEAR
$oldNsudoPath = $env:LTOOLS_NSUDO_PATH
$nsudoFixture = Join-Path $temp 'NSudo fixture\NSudoLC.exe'
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $nsudoFixture) | Out-Null
New-Item -ItemType File -Force -Path $nsudoFixture | Out-Null
$env:LTOOLS_NSUDO_PATH = $nsudoFixture
$env:USERPROFILE = $temp
$env:HOME = $temp
$env:APPDATA = Join-Path $temp 'AppData\Roaming'
$env:LOCALAPPDATA = Join-Path $temp 'AppData\Local'
$env:LTOOLS_LANG = 'es'
Remove-Item Env:LTOOLS_CLI -ErrorAction SilentlyContinue
$env:LTOOLS_NO_CLEAR = '1'
$processHelper = Join-Path $PSScriptRoot 'native-process.ps1'
if (-not (Test-Path -LiteralPath $processHelper)) { throw "Falta el helper de procesos: $processHelper" }
. $processHelper

function Run([string[]]$Arguments) {
    Write-Host ("  [RUN] ltools.exe {0}" -f ($Arguments -join ' '))
    $result = Invoke-NativeProcess -FileName $Binary -Arguments $Arguments
    if ($result.ExitCode -ne 0) {
        throw (Format-NativeProcessFailure $result "ltools.exe $($Arguments -join ' ')")
    }
    $output = [string]$result.Stdout
    if ([string]::IsNullOrWhiteSpace($output) -and
        -not [string]::IsNullOrWhiteSpace($result.Stderr)) { $output = [string]$result.Stderr }
    return [string](([string]$output).Trim())
}
function Run-WithInput([string[]]$Arguments, [string]$InputText) {
    Write-Host ("  [RUN-INPUT] ltools.exe {0}" -f ($Arguments -join ' '))
    $result = Invoke-NativeProcess -FileName $Binary -Arguments $Arguments -InputText $InputText
    if ($result.ExitCode -ne 0) {
        throw (Format-NativeProcessFailure $result 'ltools.exe E2E')
    }
    Write-Host ([string](([string]$result.Stdout).Trim()))
}
function Get-NativeHelpArguments([string]$ToolName) {
    switch -CaseSensitive ($ToolName) {
        'adb' { return @('help') }
        'git' { return @('help', '-a') }
        'git-lfs' { return @('--help') }
        'gh' { return @('--help') }
        'ssh' { return @() }
        'scp' { return @() }
        'sftp' { return @() }
        'ssh-keygen' { return @('-?') }
        'ssh-keyscan' { return @('-?') }
        'kubectl' { return @('help') }
        'helm' { return @('help') }
        '7z.exe' { return @('-h') }
        'powershell' { return @('-?') }
        'dotnet.exe' { return @('--help') }
        'curl.exe' { return @('--help') }
        'tar.exe' { return @('--help') }
        'wsl.exe' { return @('--help') }
        'python.exe' { return @('--help') }
        'node.exe' { return @('--help') }
        'npm.cmd' { return @('--help') }
        'java.exe' { return @('-help') }
        'winget' { return @('--help') }
        'choco' { return @('--help') }
        'scoop' { return @('help') }
        'docker' { return @('--help') }
        'docker-compose' { return @('--help') }
        'podman' { return @('--help') }
        'podman-compose' { return @('--help') }
        'nerdctl' { return @('--help') }
        'containerd' { return @('--help') }
        'kubeadm' { return @('help') }
        'kubelet' { return @('--help') }
        'kind' { return @('--help') }
        'minikube' { return @('--help') }
        'k3d' { return @('--help') }
        'k9s' { return @('--help') }
        'nsudo' { return @('-?') }
        'nslookup.exe' { return @('/?') }
        'certutil.exe' { return @('-?') }
        'hostname.exe' { return @() }
        default { return @('/?') }
    }
}

function Test-NativeHelp([string]$ToolName, [string[]]$Arguments) {
    if ($ToolName -match '\.msc$' -or $ToolName -in @(
        'msiexec.exe', 'trash', 'nsudo', 'msinfo32.exe', 'perfmon.exe', 'taskmgr.exe'
    )) {
        Write-Host ("  [SKIP] {0}: interfaz gráfica, integración o lanzador de elevación sin consulta segura" -f $ToolName)
        return $false
    }
    if ($ToolName -eq 'hostname.exe') {
        $command = Get-Command $ToolName -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -eq $command -or [string]::IsNullOrWhiteSpace([string]$command.Source)) {
            throw 'El catálogo marca hostname.exe disponible, pero no se resuelve a un ejecutable.'
        }
        $result = Invoke-NativeProcess -FileName $command.Source -TimeoutSeconds 12
        if ($result.ExitCode -ne 0 -or [string]::IsNullOrWhiteSpace([string]$result.Stdout)) {
            throw 'hostname.exe no completó su consulta inocua sin argumentos.'
        }
        Write-Host '  [OK] consulta nativa inocua: hostname.exe (sin página de ayuda)'
        return $true
    }
    if ($ToolName -match '^Get-[A-Za-z0-9]+$') {
        $shell = Get-Command powershell.exe, pwsh.exe -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($null -eq $shell) { throw "No hay PowerShell para consultar la sintaxis de $ToolName." }
        $syntax = Invoke-NativeProcess -FileName $shell.Source -Arguments @(
            '-NoProfile', '-NonInteractive', '-Command', "Get-Command -Name '$ToolName' -Syntax"
        ) -TimeoutSeconds 12
        $syntaxOutput = [string]$syntax.Stdout + [string]$syntax.Stderr
        if ($syntax.ExitCode -ne 0 -or $syntaxOutput -notmatch [regex]::Escape($ToolName)) {
            throw "PowerShell no publicó la sintaxis nativa de $ToolName."
        }
        Write-Host ("  [OK] sintaxis nativa PowerShell: {0}" -f $ToolName)
        return $true
    }

    $arguments = @($Arguments)
    $command = Get-Command $ToolName -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($ToolName -eq 'docker-compose' -and $null -eq $command) {
        $command = Get-Command docker -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -ne $command) { $arguments = @('compose') + $arguments }
    }
    if ($null -eq $command -or [string]::IsNullOrWhiteSpace([string]$command.Source)) {
        throw "El catálogo marca $ToolName disponible, pero no se resuelve a un ejecutable en la E2E."
    }
    $extension = [IO.Path]::GetExtension([string]$command.Source)
    if ($extension -in @('.cmd', '.bat', '.ps1')) {
        $shell = Get-Command powershell.exe, pwsh.exe -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($null -eq $shell) { throw "No hay PowerShell para consultar el wrapper $ToolName." }
        $literalPath = ([string]$command.Source).Replace("'", "''")
        $argumentText = ($arguments | ForEach-Object { " '$_'" }) -join ''
        $result = Invoke-NativeProcess -FileName $shell.Source -Arguments @(
            '-NoProfile', '-NonInteractive', '-Command', "& '$literalPath'$argumentText"
        ) -TimeoutSeconds 12
    } else {
        $result = Invoke-NativeProcess -FileName $command.Source -Arguments $arguments -TimeoutSeconds 12
    }
    $output = [string]$result.Stdout + [string]$result.Stderr
    $helpLabels = @(
        'usage', 'syntax', 'options', 'commands', 'available commands', 'parameters',
        'uso', 'sintaxis', 'opciones', 'comandos', 'parámetros',
        'utilisation', 'syntaxe', 'commandes', 'paramètres',
        'verwendung', 'optionen', 'befehle', 'utilizzo', 'sintassi', 'opzioni', 'comandi', 'parametri',
        'utilização', 'opções', 'opcions', 'ordres', 'paràmetres',
        'gebruik', 'syntaxis', 'opties', 'opdrachten', 'użycie', 'składnia', 'opcje', 'polecenia',
        'parametry', 'الاستخدام', 'استخدام', 'بناء الجملة', 'خيارات', 'أوامر', 'معلمات',
        'उपयोग', 'वाक्य रचना', 'विकल्प', 'कमांड', 'पैरामीटर',
        '使用法', '使用方法', '構文', 'オプション', 'コマンド', 'パラメーター', 'パラメータ',
        '사용법', '구문', '옵션', '명령', '매개 변수',
        'utilizare', 'sintaxă', 'opțiuni', 'comenzi', 'parametri',
        'использование', 'синтаксис', 'параметры', 'опции', 'команды',
        'використання', 'параметри', 'опції', 'команди',
        '用法', '语法', '选项', '命令', '参数'
    )
    $helpPattern = '(?im)^\s*(' + (($helpLabels | ForEach-Object { [regex]::Escape($_) }) -join '|') + ')\b'
    if ([string]::IsNullOrWhiteSpace($output)) {
        throw "La herramienta Windows $ToolName está instalada pero no devuelve ayuda."
    }
    # Algunos clientes OpenSSH imprimen su uso en stderr y devuelven 1 o 255
    # cuando se invocan sin destino (la forma segura que usamos aquí). Esos
    # códigos son una excepción documentada; una herramienta distinta debe
    # terminar correctamente para que una salida de error no se contabilice
    # como ayuda válida.
    $nonZeroHelpTools = @('ssh', 'scp', 'sftp', 'ssh-keygen', 'ssh-keyscan')
    $helpExitAccepted = $result.ExitCode -eq 0 -or
        ($ToolName -in $nonZeroHelpTools -and $result.ExitCode -in @(1, 255))
    if (-not $helpExitAccepted) {
        throw "La consulta de Windows $ToolName terminó con código $($result.ExitCode); no se acepta salida parcial como ayuda."
    }
    if ($output -notmatch $helpPattern) {
        throw "La consulta de Windows $ToolName no muestra uso/opciones nativos reconocibles (código $($result.ExitCode))."
    }
    if ($output -match '(?i)unknown command|unrecognized command|invalid choice|not a valid command|unknown option|unrecognized option|invalid option|illegal option|bad option|invalid parameter|incorrect parameter|incorrect syntax') {
        throw "La ayuda Windows de $ToolName rechazó la consulta solicitada (código $($result.ExitCode))."
    }
    Write-Host ("  [OK] ayuda nativa Windows: {0} ({1})" -f $ToolName, ($arguments -join ' '))
    return $true
}

try {
    Run-WithInput @() ("q" + [Environment]::NewLine)
    $cleanerTemp = Join-Path $env:LOCALAPPDATA 'Temp'
    $unsafeAppData = Join-Path $env:LOCALAPPDATA 'LTOOLS_UNSAFE_APP_DATA'
    New-Item -ItemType Directory -Force -Path $cleanerTemp, $unsafeAppData | Out-Null
    Set-Content -Encoding UTF8 (Join-Path $cleanerTemp 'temporary-fixture.tmp') 'temporary data'
    Set-Content -Encoding UTF8 (Join-Path $unsafeAppData 'must-not-be-cleaned.txt') 'application data'
    $env:TEMP = $cleanerTemp
    $cleanPreview = Run @('clean', '--automatic', '--preview')
    if ($cleanPreview -match 'LTOOLS_UNSAFE_APP_DATA') {
        throw 'El limpiador clasificó datos arbitrarios de LOCALAPPDATA como caché segura.'
    }
    $tempPathOccurrences = [regex]::Matches(
        $cleanPreview,
        [regex]::Escape($cleanerTemp),
        [Text.RegularExpressions.RegexOptions]::IgnoreCase
    ).Count
    if ($tempPathOccurrences -ne 1) {
        throw "El limpiador duplicó TEMP/LOCALAPPDATA\Temp ($tempPathOccurrences apariciones)."
    }
    if (-not (Test-Path -LiteralPath (Join-Path $unsafeAppData 'must-not-be-cleaned.txt'))) {
        throw 'La vista previa del limpiador modificó datos de aplicación.'
    }
    Write-Host '  [OK] limpieza Windows limitada a rutas seguras, sin duplicar TEMP ni tocar datos de aplicación'
    $output = Run @('prefix', 'list', '--root', $fixtureRoot)
    if ($output -notmatch 'no aplican') { throw 'Windows no bloqueó la lógica de prefijos Wine.' }
    $gamesReport = Join-Path $temp 'games report con espacios y ñ'
    $games = Run @('games', '--root', $fixtureRoot, '--out', $gamesReport)
    $launcherRows = Get-Content -LiteralPath (Join-Path $gamesReport 'windows-game-launchers.tsv') | Select-Object -Skip 1
    if ($launcherRows -match 'Wine|Lutris|Heroic|UMU' -or
        $games -notmatch 'Auditoría nativa de juegos Windows') {
        throw 'El inventario de juegos Windows mezcló detectores Linux/Wine.'
    }
    if ($launcherRows -notmatch 'Steam\tmanifest') {
        throw 'El inventario Windows no recogió el manifiesto nativo de Steam.'
    }
    $capabilities = Run @('capabilities', '--format', 'json')
    if ($capabilities -notmatch 'ltools-capabilities-v1') {
        throw 'El contrato JSON Windows anuncia o mezcla funciones Linux/Wine.'
    }
    $capabilityJson = $capabilities | ConvertFrom-Json
    if ($capabilityJson.application -ne 'WTools' -or $capabilityJson.platform -ne 'windows') {
        throw 'La identidad Windows del contrato no es WTools.'
    }
    if ($capabilityJson.features -notcontains 'verified-updates' -or
        $capabilityJson.features -notcontains 'verified-update-download' -or
        $capabilityJson.distribution.windows.updates.automatic_install -ne $false) {
        throw 'El contrato Windows omite la descarga verificada o promete instalación automática.'
    }
    if ($capabilityJson.features -contains 'wine-prefixes' -or
        $capabilities -match 'Heroic|Lutris|UMU') {
        throw 'El contrato JSON Windows anuncia o mezcla funciones Linux/Wine.'
    }
    $actionCatalogText = Run @('actions', 'list', '--format', 'json')
    $actionCatalog = $actionCatalogText | ConvertFrom-Json
    if ($actionCatalog.schema -ne 'ltools-actions-v1' -or
        $actionCatalog.platform -ne 'windows' -or
        $null -eq $actionCatalog.actions) {
        throw 'El catálogo JSON Windows de acciones no devuelve esquema, plataforma o acciones válidos.'
    }
    $actionEntries = @($actionCatalog.actions)
    if ($actionEntries.Count -eq 0) {
        throw 'El catálogo JSON Windows de acciones está vacío.'
    }
    $actionIds = @($actionEntries | ForEach-Object { [string]$_.id })
    if (($actionIds | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($actionIds | Sort-Object -Unique).Count -ne $actionIds.Count)) {
        throw 'El catálogo JSON Windows contiene IDs de acciones vacíos o duplicados.'
    }
    $actionKeys = @($actionEntries | ForEach-Object { [string]$_.actionKey })
    $qualifiedActionKeys = @($actionEntries | ForEach-Object { [string]$_.qualifiedActionKey })
    $canonicalActionKeys = @($actionEntries | ForEach-Object { [string]$_.canonicalKey })
    $actionIdsCanonical = @($actionEntries | ForEach-Object { [string]$_.actionId })
    $actionOperations = @($actionEntries | ForEach-Object { [string]$_.operation })
    $actionLabels = @($actionEntries | ForEach-Object { [string]$_.label })
    $actionShortLabels = @($actionEntries | ForEach-Object { [string]$_.shortLabel })
    $actionDisplayNames = @($actionEntries | ForEach-Object { [string]$_.displayName })
    if (($actionKeys | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($actionKeys | Sort-Object -Unique).Count -ne $actionKeys.Count)) {
        throw 'El catálogo JSON Windows contiene actionKey vacíos o duplicados.'
    }
    if (($qualifiedActionKeys | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($qualifiedActionKeys | Sort-Object -Unique).Count -ne $qualifiedActionKeys.Count)) {
        throw 'El catálogo JSON Windows contiene qualifiedActionKey vacíos o duplicados.'
    }
    if (($canonicalActionKeys | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($canonicalActionKeys | Sort-Object -Unique).Count -ne $canonicalActionKeys.Count)) {
        throw 'El catálogo JSON Windows contiene canonicalKey vacíos o duplicados.'
    }
    if (($actionIdsCanonical | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($actionIdsCanonical | Sort-Object -Unique).Count -ne $actionIdsCanonical.Count)) {
        throw 'El catálogo JSON Windows contiene actionId vacíos o duplicados.'
    }
    if (($actionOperations | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($actionOperations | Sort-Object -Unique).Count -ne $actionOperations.Count)) {
        throw 'El catálogo JSON Windows contiene operation vacíos o duplicados.'
    }
    if (($actionLabels | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($actionLabels | Sort-Object -Unique).Count -ne $actionLabels.Count) -or
        ($actionShortLabels | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($actionShortLabels | Sort-Object -Unique).Count -ne $actionShortLabels.Count)) {
        throw 'El catálogo JSON Windows contiene nombres descriptivos vacíos o duplicados.'
    }
    if (($actionDisplayNames | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
        (@($actionDisplayNames | Sort-Object -Unique).Count -ne $actionDisplayNames.Count)) {
        throw 'El catálogo JSON Windows contiene displayName vacíos o duplicados.'
    }
    $knownActionBackends = @(
        'audit', 'packages', 'games', 'storage', 'native', 'system', 'accounts',
        'defaults', 'clean', 'diagnostics', 'automation', 'boot', 'wine'
    )
    foreach ($action in $actionEntries) {
        $actionKey = [string]$action.actionKey
        if ([string]$action.id -cne [string]$action.actionId -or
            [string]::IsNullOrWhiteSpace([string]$action.legacyId) -or
            [string]$action.actionId -cne [string]$action.canonicalKey -or
            [string]$action.qualifiedActionKey -cne ('windows.' + $actionKey) -or
            [string]$action.canonicalKey -cne [string]$action.qualifiedActionKey -or
            @($action.menuPath).Count -ne 2 -or
            [string]::IsNullOrWhiteSpace([string]$action.displayName) -or
            [string]$action.invocation.executable -notin @('ltools.exe', 'ltools') -or
            (@($action.invocation.args) -join '|') -cne ('actions|run|' + [string]$action.actionId) -or
            [string]$action.invocation.target -cne [string]$action.target) {
            throw "La acción Windows $($action.id) no publica identidad cualificada o invocación declarativa consistente."
        }
        $scope = [string]$action.scope
        $operation = [string]$action.operation
        if ($actionKey -notmatch '^[a-z0-9]+(?:[.-][a-z0-9]+)+$' -or
            [string]::IsNullOrWhiteSpace($scope) -or
            [string]::IsNullOrWhiteSpace($operation) -or
            $actionKey.Split('.')[0] -ne $scope -or
            $operation -cne ($actionKey -replace '\.', '-')) {
            throw "La acción Windows $($action.id) carece de actionKey/scope/operation consistentes."
        }
        if ([string]::IsNullOrWhiteSpace([string]$action.category) -or
            [string]::IsNullOrWhiteSpace([string]$action.command) -or
            [string]::IsNullOrWhiteSpace([string]$action.label) -or
            [string]::IsNullOrWhiteSpace([string]$action.shortLabel) -or
            [string]::IsNullOrWhiteSpace([string]$action.description) -or
            $knownActionBackends -notcontains [string]$action.command -or
            [string]::IsNullOrWhiteSpace([string]$action.target) -or
            [string]::IsNullOrWhiteSpace([string]$action.profile) -or
            [string]::IsNullOrWhiteSpace([string]$action.confirmation) -or
            $action.mutating -isnot [bool]) {
            throw "La acción Windows $($action.id) carece de backend, política o metadatos válidos."
        }
        $invalidActionArgs = @($action.args | Where-Object { $_ -isnot [string] })
        if ($invalidActionArgs.Count -ne 0) {
            throw "La acción Windows $($action.id) contiene argumentos que no son cadenas."
        }
        if ([string]$action.target -ne 'none' -and
            [string]$action.targetPolicy -ne 'explicit-only') {
            throw "La acción Windows $($action.id) no exige objetivo explícito."
        }
        if ([bool]$action.mutating -and [string]$action.confirmation -eq 'none') {
            throw "La acción Windows mutadora $($action.id) no tiene confirmación."
        }
    }
    Write-Host ("  [OK] catálogo Windows de acciones: {0} IDs, backends, argumentos y políticas verificados" -f $actionEntries.Count)
    foreach ($nativeActionId in @('native-network', 'native-hardware', 'native-security')) {
        $nativeActions = @($capabilityJson.actions | Where-Object { $_.legacyId -eq $nativeActionId })
        if ($nativeActions.Count -ne 1 -or @($nativeActions[0].requiresCommands).Count -ne 0) {
            throw "El contrato Windows marca $nativeActionId como dependiente de una herramienta opcional pese a disponer de fallbacks nativos."
        }
    }
    $powerAction = @($capabilityJson.actions | Where-Object { $_.legacyId -eq 'native-power' })
    if ($powerAction.Count -ne 1 -or
        @($powerAction[0].requiresCommands) -notcontains 'powercfg') {
        throw 'El contrato Windows no conserva powercfg como dependencia nativa de energía.'
    }
    Write-Host '  [OK] contrato Windows distingue fallbacks nativos de dependencias opcionales'
    $updateHelp = Run @('update', '--help')
    foreach ($marker in @('check', 'download', '--repository OWNER/REPO')) {
        if ($updateHelp -notmatch [regex]::Escape($marker)) {
            throw "La ayuda Windows del actualizador no documenta: $marker"
        }
    }
    $pausedUpdateFailure = Invoke-NativeProcess -FileName $Binary -Arguments @('update', 'invalid', '--pause') -InputText ([Environment]::NewLine)
    $pausedUpdateOutput = [string]$pausedUpdateFailure.Stdout + [string]$pausedUpdateFailure.Stderr
    if ($pausedUpdateFailure.ExitCode -eq 0 -or
        $pausedUpdateOutput -notmatch 'Error: acción update desconocida' -or
        $pausedUpdateOutput -notmatch 'Pulsa Enter para cerrar esta consulta') {
        throw 'La consola pausada de actualización no conserva el error visible y el código de fallo.'
    }
    $updateGuide = Run @('guide', 'updates')
    foreach ($marker in @('firma Ed25519', 'SHA256SUMS.txt', 'No ejecuta ni reemplaza')) {
        if ($updateGuide -notmatch [regex]::Escape($marker)) {
            throw "La guía CLI Windows del actualizador no documenta: $marker"
        }
    }
    $updateGuiGuide = Run @('guide', 'gui', 'updates')
    foreach ($marker in @('Comprobar actualizaciones', 'Descargar actualización verificada', 'Windows', 'No se eleva')) {
        if ($updateGuiGuide -notmatch [regex]::Escape($marker)) {
            throw "La guía GUI Windows del actualizador no documenta: $marker"
        }
    }
    Write-Host '  [OK] actualizador Windows: CLI, GUI, plataforma e integridad documentadas sin acceso a red'
    Run-WithInput @('system', '--dry-run', 'service', 'restart', 'EventLog') ("y" + [Environment]::NewLine)
    $storageOutput = Run @('storage', 'tools')
    if ($storageOutput -notmatch 'diskpart') { throw 'El módulo Windows de almacenamiento falló.' }
    $snapshotStatus = Run @('snapshots', 'status')
    if ($snapshotStatus -notmatch 'Backends de instantáneas') { throw 'El E2E Windows no pudo consultar snapshots.' }
    $snapshotPlan = Run @('--dry-run', 'snapshots', 'create', '--backend', 'vss', '--volume', 'C:')
    if ($snapshotPlan -notmatch 'vssadmin(?:\.exe)? create shadow /for=C:') {
        throw 'El E2E Windows no generó el plan VSS esperado.'
    }
    $storageStatus = Run @('storage', 'status')
    if ($storageStatus -notmatch 'Almacenamiento Windows') { throw 'El estado de almacenamiento Windows falló.' }
    $storagePartitions = Run @('storage', 'partitions')
    if ($storagePartitions -notmatch 'Discos y particiones Windows') { throw 'El inventario de particiones Windows falló.' }
    foreach ($storageQuery in @('usage', 'pools', 'bitlocker')) {
        $storageQueryOutput = Run @('storage', $storageQuery)
        if ([string]::IsNullOrWhiteSpace($storageQueryOutput)) {
            throw "La consulta Windows de almacenamiento $storageQuery no devolvió salida."
        }
    }
    $storageMapJson = Run @('storage', 'map', '--path', $fixtureRoot, '--depth', '1', '--format', 'json')
    $storageMap = $storageMapJson | ConvertFrom-Json
    if ($storageMap.schema -ne 'ltools-storage-map-v1' -or
        $storageMap.platform -ne 'windows' -or $storageMap.roots.Count -ne 1) {
        throw 'El mapa Windows no devuelve su esquema, plataforma y raíz esperados.'
    }
    $mappedVolume = $storageMap.roots[0]
    if ($null -eq $mappedVolume.filesystem_total -or
        $null -eq $mappedVolume.filesystem_used -or
        $null -eq $mappedVolume.filesystem_free -or
        $null -eq $mappedVolume.filesystem_available -or
        [uint64]$mappedVolume.filesystem_total -ne
        ([uint64]$mappedVolume.filesystem_used + [uint64]$mappedVolume.filesystem_free) -or
        [uint64]$mappedVolume.filesystem_available -gt [uint64]$mappedVolume.filesystem_free) {
        throw 'El mapa Windows no informa espacio total, ocupado, libre y disponible de forma coherente.'
    }
    $tarCommand = Get-Command tar.exe -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -ne $tarCommand) {
        $archiveSource = Join-Path $temp 'safe archive source'
        New-Item -ItemType Directory -Force -Path $archiveSource | Out-Null
        Set-Content -Encoding UTF8 (Join-Path $archiveSource 'inside.txt') 'archive fixture'
        $safeArchive = Join-Path $temp 'safe-archive.tar'
        $archiveResult = Invoke-NativeProcess -FileName $Binary -Arguments @(
            'storage', 'manage', 'tar', '--source', $archiveSource,
            '--destination', $safeArchive, '--yes'
        )
        if ($archiveResult.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $safeArchive -PathType Leaf)) {
            throw 'El archivador Windows no pudo publicar un TAR completo.'
        }
        $archiveListing = Invoke-NativeProcess -FileName $tarCommand.Source -Arguments @('-tf', $safeArchive)
        if ($archiveListing.ExitCode -ne 0 -or [string]$archiveListing.Stdout -notmatch 'inside\.txt') {
            throw 'El TAR Windows publicado no contiene el archivo esperado.'
        }
        $existingArchive = Join-Path $temp 'existing-archive.tar'
        Set-Content -Encoding ASCII $existingArchive 'preserve existing archive'
        $existingResult = Invoke-NativeProcess -FileName $Binary -Arguments @(
            'storage', 'manage', 'tar', '--source', $archiveSource,
            '--destination', $existingArchive, '--yes'
        )
        if ($existingResult.ExitCode -eq 0 -or
            (Get-Content -Raw -LiteralPath $existingArchive).Trim() -ne 'preserve existing archive') {
            throw 'El archivador Windows sobrescribió un destino preexistente.'
        }
        if (@(Get-ChildItem -LiteralPath $temp -Directory -Force |
            Where-Object { $_.Name -like '.ltools-stage-*' }).Count -gt 0) {
            throw 'El gestor de archivos Windows dejó un directorio temporal de staging.'
        }
        Write-Host '  [OK] TAR Windows: staging publicado completo, destino existente preservado y staging retirado'
    } else {
        Write-Host '  [SKIP] TAR Windows no está instalado; no se valida publicación de archivos.'
    }
    if ($junctionCreated) {
        $junctionNode = @($mappedVolume.children | Where-Object { $_.path -ieq $junctionPath }) |
            Select-Object -First 1
        if ($null -eq $junctionNode -or
            $junctionNode.kind -notin @('symlink', 'reparse-point') -or
            $junctionNode.size -ne 0 -or @($junctionNode.children).Count -ne 0) {
            throw 'El mapa Windows atravesó una junction o no la identificó como punto de reanálisis.'
        }
        $junctionDelete = Invoke-NativeProcess -FileName $Binary -Arguments @(
            'storage', 'manage', 'delete', '--path', $junctionPath, '--yes'
        )
        if ($junctionDelete.ExitCode -eq 0 -or
            -not (Test-Path -LiteralPath $junctionPath) -or
            -not (Test-Path -LiteralPath (Join-Path $reparseTarget 'keep.txt'))) {
            throw 'La papelera Windows no rechazó la junction o alteró su destino.'
        }
        $junctionArchivePath = Join-Path $temp 'junction-tree.zip'
        $junctionArchive = Invoke-NativeProcess -FileName $Binary -Arguments @(
            'storage', 'manage', 'zip', '--source', $fixtureRoot,
            '--destination', $junctionArchivePath, '--yes'
        )
        if ($junctionArchive.ExitCode -eq 0 -or (Test-Path -LiteralPath $junctionArchivePath)) {
            throw 'El archivador Windows atravesó una junction dentro del árbol.'
        }
        $cleanerLocalAppData = Join-Path $fixtureRoot 'cleaner-local-appdata'
        $cleanerTempRoot = Join-Path $cleanerLocalAppData 'Temp'
        $cleanerSystemRoot = Join-Path $fixtureRoot 'cleaner-windows-root'
        $cleanerJunction = Join-Path $cleanerTempRoot 'external-junction'
        New-Item -ItemType Directory -Force -Path $cleanerTempRoot, (Join-Path $cleanerSystemRoot 'Temp') | Out-Null
        Set-Content -Encoding UTF8 (Join-Path $cleanerTempRoot 'ordinary.tmp') 'preserve incomplete cleanup'
        New-Item -ItemType Junction -Path $cleanerJunction -Target $reparseTarget -ErrorAction Stop | Out-Null
        $savedCleanerLocalAppData = $env:LOCALAPPDATA
        $savedCleanerTemp = $env:TEMP
        $savedCleanerSystemRoot = $env:SystemRoot
        try {
            $env:LOCALAPPDATA = $cleanerLocalAppData
            $env:TEMP = $cleanerTempRoot
            $env:SystemRoot = $cleanerSystemRoot
            $cleanerResult = Invoke-NativeProcess -FileName $Binary -Arguments @('clean', '--automatic') `
                -InputText ("y{0}y{0}" -f [Environment]::NewLine) -TimeoutSeconds 30
            $cleanerOutput = [string]$cleanerResult.Stdout + [string]$cleanerResult.Stderr
        } finally {
            $env:LOCALAPPDATA = $savedCleanerLocalAppData
            $env:TEMP = $savedCleanerTemp
            $env:SystemRoot = $savedCleanerSystemRoot
        }
        if ($cleanerResult.ExitCode -eq 0 -or
            $cleanerOutput -notmatch '(?i)punto de reanálisis' -or
            -not (Test-Path -LiteralPath $cleanerJunction) -or
            -not (Test-Path -LiteralPath (Join-Path $reparseTarget 'keep.txt')) -or
            -not (Test-Path -LiteralPath (Join-Path $cleanerTempRoot 'ordinary.tmp'))) {
            throw 'El limpiador Windows no informó el fallo o borró contenido antes de detectar la junction.'
        }
        Write-Host '  [OK] limpiador Windows rechaza junctions antes de borrar contenido y nunca toca su destino'
        Write-Host '  [OK] mapa/papelera/archivo Windows no atraviesan ni borran el destino de una junction'
    }
    Write-Host '  [OK] mapa Windows: total, ocupado, libre total y disponible para la cuenta coherentes'
    $nativeTools = Run @('native', 'tools', 'status')
    foreach ($toolName in @('ssh', 'scp', 'sftp', 'adb', 'docker', 'kubectl', 'shellcheck.exe', 'actionlint.exe', 'zizmor.exe', 'gitleaks.exe', 'osv-scanner.exe', 'codeql.exe', 'scorecard.exe', 'cargo-audit.exe', 'cargo-deny.exe')) {
        if ($nativeTools -notmatch "(?im)^$([regex]::Escape($toolName))[\t ]{2,}") {
            throw "El inventario Windows de herramientas nativas no mostró la fila de $toolName.`nSalida recibida:`n$nativeTools"
        }
    }
    foreach ($networkAction in @('status', 'interfaces', 'routes', 'dns', 'listening', 'connections')) {
        $networkOutput = Run @('native', 'network', $networkAction)
        if ([string]::IsNullOrWhiteSpace($networkOutput)) {
            throw "La consulta nativa Windows de red $networkAction no devolvió salida."
        }
    }
    Write-Host '  [OK] red Windows: interfaces, rutas, DNS, escucha y conexiones por separado'
    foreach ($nativeQuery in @('hardware', 'power', 'security')) {
        $nativeQueryOutput = Run @('native', $nativeQuery, 'status')
        if ([string]::IsNullOrWhiteSpace($nativeQueryOutput)) {
            throw "La consulta nativa Windows de $nativeQuery no devolvió salida."
        }
    }
    Write-Host '  [OK] hardware, energía y seguridad Windows: herramientas nativas o fallbacks integrados'
    $accountsGuide = Run @('guide', 'gui', 'accounts')
    foreach ($adminMarker in @('Conceder permisos de administrador', 'Ver grupo y miembros administradores', 'S-1-5-32-544', 'TrustedInstaller es una identidad de servicio')) {
        if ($accountsGuide -notmatch [regex]::Escape($adminMarker)) {
            throw "La guía Windows de cuentas no documenta correctamente: $adminMarker"
        }
    }
    $adminDryRun = Run @('--dry-run', 'accounts', 'admin-add')
    if ($adminDryRun -notmatch 'S-1-5-32-544' -or $adminDryRun -notmatch 'Add-LocalGroupMember') {
        throw 'La concesión de administrador Windows no prepara el grupo integrado correcto en modo simulación.'
    }
    $nativeHelpTested = 0
    $nativeHelpSkipped = 0
    foreach ($tool in $capabilityJson.host_tools) {
        if (-not $tool.available) { continue }
        $arguments = @(Get-NativeHelpArguments ([string]$tool.command))
        if (Test-NativeHelp ([string]$tool.command) $arguments) {
            $nativeHelpTested++
        } else {
            $nativeHelpSkipped++
        }
    }
    if ($nativeHelpTested -eq 0) { throw 'La E2E no ejecutó ayudas ni sintaxis de herramientas Windows disponibles.' }
    Write-Host ("  [OK] catálogo nativo Windows: {0} ayudas, sintaxis o consultas inocuas probadas; {1} GUI/integraciones/lanzadores privilegiados omitidos" -f $nativeHelpTested, $nativeHelpSkipped)
    $diskGuide = Run @('storage', 'guide')
    foreach ($guideMarker in @('list disk', 'select disk', 'detail disk', 'clean all', 'C:')) {
        if ($diskGuide -notmatch [regex]::Escape($guideMarker)) {
            throw "La guía DiskPart Windows no contiene el paso protegido esperado: $guideMarker"
        }
    }
    if (-not (($capabilityJson.host_tools | ForEach-Object { [string]$_.id }) -contains 'git-lfs')) {
        throw 'El catálogo Windows omite Git LFS como dependencia instalable de Git for Windows.'
    }
    $gitLfsTool = @($capabilityJson.host_tools | Where-Object { $_.id -eq 'git-lfs' }) | Select-Object -First 1
    if ([string]$gitLfsTool.install_package -ne 'Git.Git') {
        throw 'Git LFS Windows no está vinculado al paquete Git.Git.'
    }
    if ($gitLfsTool.available) {
        $gitLfsVersion = Run @('git', 'lfs', 'version')
        if ($gitLfsVersion -notmatch 'git-lfs') {
            throw 'Git LFS está marcado como disponible, pero `git lfs version` no respondió.'
        }
        Write-Host '  [OK] Git LFS Windows disponible y ejecutable mediante Git for Windows'
    } else {
        Write-Host '  [SKIP] Git LFS no está instalado en este anfitrión Windows; catálogo e instalación sí están verificados.'
    }
    $windowsGitGuide = Run @('guide', 'cli', 'git')
    foreach ($gitLfsMarker in @('git-lfs.exe', 'ltools git lfs status', 'git lfs native')) {
        if ($windowsGitGuide -notmatch [regex]::Escape($gitLfsMarker)) {
            throw "La guía Git Windows omite la integración de Git LFS: $gitLfsMarker"
        }
    }
    $windowsGuideTopics = @('audit','packages','software','git','aliases','automation','automation-register','clean','storage','storage-partitions','storage-filesystems','storage-volumes','system','services','accounts','native','network','boot','registry','diagnostics','defaults','installable','settings','updates','containers','containers-lifecycle','containers-images','containers-volumes','containers-compose','kubernetes','ssh','connectivity','adb','utilities','privileges','wine','prefix','wtools')
    foreach ($guideTopic in $windowsGuideTopics) {
        $guide = Run @('guide', 'gui', $guideTopic)
        if ([string]::IsNullOrWhiteSpace($guide) -or $guide -notmatch 'GUÍA (CLI|GRÁFICA)') {
            throw "La guía Windows GUI/$guideTopic no devuelve su cabecera ni contenido."
        }
        if ($guide -match '«\s*»') {
            throw "La guía Windows GUI/$guideTopic incluye una opción de menú sin etiqueta."
        }
    }
    foreach ($topic in @('clean','services','ssh','connectivity','utilities','wine','prefix','aliases','containers-lifecycle','containers-images','containers-volumes','containers-compose')) {
        $guide = Run @('guide', 'gui', $topic)
        if ($guide -notmatch 'no tiene|no tienen|no aplican') {
            throw "La guía Windows GUI/$topic no declara que la pantalla no existe o no aplica."
        }
    }
    foreach ($topic in @('storage-partitions','storage-filesystems','storage-volumes')) {
        $guide = Run @('guide', 'gui', $topic)
        if ($guide -notmatch 'no se implementan en la GUI Windows') {
            throw "La guía Windows GUI/$topic no rechaza claramente las operaciones exclusivas de Linux."
        }
    }
    $automationGuide = Run @('guide', 'gui', 'automation')
    foreach ($marker in @('Listar scripts registrados', 'Registrar un script', 'Ejecutar un script registrado', 'Editar un script registrado', 'Eliminar un registro')) {
        if ($automationGuide -notmatch [regex]::Escape($marker)) {
            throw "La guía de automatización Windows no refleja la opción real: $marker"
        }
    }
    $installableGuide = Run @('guide', 'gui', 'installable')
    if ($installableGuide -notmatch 'ADB: dispositivos conectados') {
        throw 'La guía instalable Windows no etiqueta la acción de ADB.'
    }
    $settingsGuide = Run @('guide', 'gui', 'settings')
    foreach ($marker in @('Campo «Tema»', 'Campo «Idioma»', 'Aplicar ajustes', 'auto')) {
        if ($settingsGuide -notmatch [regex]::Escape($marker)) {
            throw "La guía de Ajustes Windows no documenta el control nativo: $marker"
        }
    }
    foreach ($themeId in @('silver','winslim','ocean','forest','amber','violet','nordic','crimson','matrix','contrast','slate','plum','teal')) {
        if ($settingsGuide -notmatch [regex]::Escape($themeId)) {
            throw "La guía de Ajustes Windows no documenta el tema admitido: $themeId"
        }
    }
    foreach ($languageId in @('ar','de','en','es','fr','hi','it','ja','ko','pl','pt','ro','ru','uk','zh')) {
        if ($settingsGuide -notmatch [regex]::Escape($languageId)) {
            throw "La guía de Ajustes Windows no documenta el idioma admitido: $languageId"
        }
    }
    if ($settingsGuide -match 'Guía de ajustes y visibilidad»') {
        throw 'La guía de Ajustes Windows anuncia un botón de guía que no existe en ese formulario.'
    }
    $allGuides = Run @('guide', 'gui', 'all')
    foreach ($marker in @('Panel principal:', 'Cada categoría abre un menú propio', 'Las guías contextuales')) {
        if ($allGuides -notmatch [regex]::Escape($marker)) { throw "El índice de guías Windows no contiene: $marker" }
    }
    foreach ($menuOption in @(
        'Auditar discos y aplicaciones', 'Resumen de espacio y montajes',
        'Discos y particiones', 'Dependencias',
        'Rutas predeterminadas', 'Herramientas instalables', 'Automatización',
        'Usuarios, grupos y sesiones', 'Aplicar ajustes'
    )) {
        if ($allGuides -notmatch [regex]::Escape($menuOption)) {
            throw "El índice Windows no refleja una opción real de sus menús: $menuOption"
        }
    }
    if ($allGuides -match 'Crear prefijo|Migrar prefijo|Mapa desplegable') {
        throw 'El índice Windows anuncia una opción Linux o una función GUI Windows no implementada.'
    }
    $winslimGuide = Run @('guide', 'gui', 'wtools')
    $winslimIndexVisible = $allGuides -match '(?m)^WTools:\s*$'
    if ($winslimGuide -match 'no tiene disponible la pantalla WTools/NSudo') {
        if ($allGuides -match '(?m)^WTools:\s*$') {
            throw 'El índice GUI anuncia WTools/NSudo sin WSCore ni un lanzador detectado.'
        }
    } else {
        if ($winslimGuide -notmatch 'Estado de WSCore y NSudo' -or -not $winslimIndexVisible) {
            throw 'La guía o el índice no reflejan el menú WTools condicional disponible.'
        }
    }
    $winslimStatus = Run @('wtools', 'status')
    if ($winslimStatus -notmatch 'WSCore|WTools' -or $winslimStatus -notmatch 'NSudo') {
        throw 'El estado WTools no informa de forma diferenciada WSCore y NSudo.'
    }
    if ($winslimStatus -notmatch [regex]::Escape($nsudoFixture)) {
        throw 'LTOOLS_NSUDO_PATH no detectó la ruta explícita del lanzador de prueba.'
    }
    $winslimGuideText = Run @('wtools', 'guide')
    foreach ($nsudoMarker in @('TrustedInstaller', 'drop-rights', '--integrity', '--all-privileges', '--yes')) {
        if ($winslimGuideText -notmatch [regex]::Escape($nsudoMarker)) {
            throw "La guía NSudo omite una identidad u opción compatible: $nsudoMarker"
        }
    }
    $nsudoMenuResult = Invoke-NativeProcess -FileName $Binary -Arguments @('wtools', 'menu') -InputText ("q" + [Environment]::NewLine) -TimeoutSeconds 15
    $nsudoMenu = [string]$nsudoMenuResult.Stdout + [string]$nsudoMenuResult.Stderr
    if ($nsudoMenuResult.ExitCode -ne 0 -or $nsudoMenu -notmatch 'Lanzar proceso con contexto elegido') {
        throw 'El menú WTools no muestra el asistente cuando se detecta un lanzador NSudo.'
    }
    $nsudoPlan = Run @('--dry-run', 'wtools', 'launch', '--identity', 'system', '--program', 'cmd.exe', '--arg', '/c', '--arg', 'ver', '--all-privileges', '--integrity', 'high', '--window', 'maximize', '--wait', '--console')
    foreach ($nsudoMarker in @('-U:S -P:E -M:H -Wait -UseCurrentConsole', 'Modo de ventana: maximize', 'Programa: cmd.exe (2 argumento(s)', 'Simulación: no se inició')) {
        if ($nsudoPlan -notmatch [regex]::Escape($nsudoMarker)) {
            throw "El plan NSudo no conserva el contexto/parámetros esperados: $nsudoMarker"
        }
    }
    if ($nsudoPlan -match 'Se inició NSudo|Ejecutado correctamente') {
        throw 'La simulación NSudo parece haber lanzado un proceso real.'
    }
    $nativeGuiGuide = Run @('guide', 'gui', 'native')
    foreach ($nativeOption in @(
        'Resumen de espacio y montajes', 'Discos y particiones',
        'Abrir gestor nativo de particiones', 'Estado del sistema',
        'Arranque, EFI y cargador del sistema', 'Inspeccionar el Registro de Windows', 'Volver',
        'Estado del hardware', 'Estado y planes de energía',
        'Estado del firewall y seguridad', 'Analizadores de código y CI'
    )) {
        if ($nativeGuiGuide -notmatch [regex]::Escape($nativeOption)) {
            throw "La guía de herramientas nativas Windows no refleja su botón: $nativeOption"
        }
    }
    $windowsStorageGuiGuide = Run @('guide', 'gui', 'storage')
    foreach ($storageGuiOption in @('Uso de espacio por volumen', 'Espacios de almacenamiento y discos virtuales', 'Estado de BitLocker')) {
        if ($windowsStorageGuiGuide -notmatch [regex]::Escape($storageGuiOption)) {
            throw "La guía gráfica Windows no refleja la consulta disponible: $storageGuiOption"
        }
    }
    if ($windowsStorageGuiGuide -notmatch 'no está integrado en la\s+GUI Windows' -or
        $windowsStorageGuiGuide -match 'Mapa desplegable de discos y rutas') {
        throw 'La guía gráfica de almacenamiento Windows afirma que existe un mapa interactivo que no está en esta GUI.'
    }
    $registryOutput = Run @('registry', 'status')
    if ($registryOutput -notmatch 'Registro Windows') { throw 'El módulo Windows de registro falló.' }
    $storesOutput = Run @('software', 'stores')
    if ($storesOutput -match 'pacman|apt|flatpak|dnf|zypper|brew') {
        throw 'El inventario Windows de stores mezcló gestores Linux.'
    }
    if ($storesOutput -notmatch 'winget|choco|scoop') {
        throw 'El inventario Windows no mostró su catálogo nativo de stores.'
    }
    $cliResult = Invoke-NativeProcess -FileName $Binary -EnvironmentOverrides @{ LTOOLS_CLI = '1' } -TimeoutSeconds 15
    $cliOutput = [string]$cliResult.Stdout + [string]$cliResult.Stderr
    if ($cliResult.ExitCode -ne 0 -or $cliOutput -notmatch 'Uso: ltools|Usage: ltools') {
        throw (Format-NativeProcessFailure $cliResult 'El perfil CLI Windows falló')
    }
    $processOutput = Run @('system', 'processes', '--sort', 'memory', '--limit', '3')
    if ($processOutput -notmatch 'Procesos Windows') { throw 'El listado de procesos Windows falló.' }
    Write-Host 'Windows E2E completado correctamente.'
} finally {
    $env:USERPROFILE = $oldUserProfile
    $env:HOME = $oldHome
    $env:APPDATA = $oldAppData
    $env:LOCALAPPDATA = $oldLocalAppData
    if ($null -eq $oldTemp) { Remove-Item Env:TEMP -ErrorAction SilentlyContinue }
    else { $env:TEMP = $oldTemp }
    if ($null -eq $oldSystemRoot) { Remove-Item Env:SystemRoot -ErrorAction SilentlyContinue }
    else { $env:SystemRoot = $oldSystemRoot }
    if ($null -eq $oldSystemDrive) { Remove-Item Env:SystemDrive -ErrorAction SilentlyContinue }
    else { $env:SystemDrive = $oldSystemDrive }
    if ($null -eq $oldLanguage) { Remove-Item Env:LTOOLS_LANG -ErrorAction SilentlyContinue }
    else { $env:LTOOLS_LANG = $oldLanguage }
    if ($null -eq $oldCliMode) { Remove-Item Env:LTOOLS_CLI -ErrorAction SilentlyContinue }
    else { $env:LTOOLS_CLI = $oldCliMode }
    if ($null -eq $oldNoClear) { Remove-Item Env:LTOOLS_NO_CLEAR -ErrorAction SilentlyContinue }
    else { $env:LTOOLS_NO_CLEAR = $oldNoClear }
    if ($null -eq $oldNsudoPath) { Remove-Item Env:LTOOLS_NSUDO_PATH -ErrorAction SilentlyContinue }
    else { $env:LTOOLS_NSUDO_PATH = $oldNsudoPath }
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
