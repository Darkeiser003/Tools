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
function Test-NativeHelp([string]$ToolName, [string[]]$Arguments) {
    $command = Get-Command $ToolName -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $command) { return }
    $result = Invoke-NativeProcess -FileName $command.Source -Arguments $Arguments -TimeoutSeconds 12
    $output = [string]$result.Stdout + [string]$result.Stderr
    if ([string]::IsNullOrWhiteSpace($output)) {
        throw "La herramienta Windows $ToolName está instalada pero no devuelve ayuda."
    }
    if ($output -notmatch '(?i)usage|options|commands|help|opciones|comandos|syntax|parameter') {
        throw "La consulta de Windows $ToolName no parece una página de ayuda válida (código $($result.ExitCode))."
    }
    if ($output -match '(?i)unknown command|unrecognized command|invalid choice|not a valid command|unknown option|unrecognized option|invalid option|illegal option|bad option|invalid parameter|incorrect parameter|incorrect syntax') {
        throw "La ayuda Windows de $ToolName rechazó la consulta solicitada (código $($result.ExitCode))."
    }
    Write-Host ("  [OK] ayuda nativa Windows: {0} ({1})" -f $ToolName, ($Arguments -join ' '))
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
    if ($capabilityJson.application -ne 'WinSlim-Tools' -or $capabilityJson.platform -ne 'windows') {
        throw 'La identidad Windows del contrato no es WinSlim-Tools.'
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
    $storageStatus = Run @('storage', 'status')
    if ($storageStatus -notmatch 'Almacenamiento Windows') { throw 'El estado de almacenamiento Windows falló.' }
    $storagePartitions = Run @('storage', 'partitions')
    if ($storagePartitions -notmatch 'Discos y particiones Windows') { throw 'El inventario de particiones Windows falló.' }
    $nativeTools = Run @('native', 'tools', 'status')
    foreach ($toolName in @('ssh', 'scp', 'sftp', 'adb', 'docker', 'kubectl')) {
        if ($nativeTools -notmatch [regex]::Escape($toolName)) {
            throw "El inventario Windows de herramientas nativas no mostró $toolName."
        }
    }
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
    $nativeHelpSpecs = @(
        [pscustomobject]@{ Tool = 'adb'; Args = @('help') }
        [pscustomobject]@{ Tool = 'git'; Args = @('help', '-a') }
        [pscustomobject]@{ Tool = 'gh'; Args = @('--help') }
        # OpenSSH clients print usage without a destination and return a
        # nonzero status; -h is not a portable help flag for these tools.
        [pscustomobject]@{ Tool = 'ssh'; Args = @() }
        [pscustomobject]@{ Tool = 'scp'; Args = @() }
        [pscustomobject]@{ Tool = 'sftp'; Args = @() }
        [pscustomobject]@{ Tool = 'docker'; Args = @('--help') }
        [pscustomobject]@{ Tool = 'podman'; Args = @('--help') }
        [pscustomobject]@{ Tool = 'kubectl'; Args = @('help') }
        [pscustomobject]@{ Tool = 'curl'; Args = @('--help') }
        [pscustomobject]@{ Tool = 'winget'; Args = @('--help') }
        [pscustomobject]@{ Tool = 'choco'; Args = @('--help') }
        [pscustomobject]@{ Tool = 'sc.exe'; Args = @('/?') }
        [pscustomobject]@{ Tool = 'reg.exe'; Args = @('/?') }
    )
    foreach ($spec in $nativeHelpSpecs) {
        Test-NativeHelp $spec.Tool $spec.Args
    }
    $diskGuide = Run @('storage', 'guide')
    foreach ($guideMarker in @('list disk', 'select disk', 'detail disk', 'clean all', 'C:')) {
        if ($diskGuide -notmatch [regex]::Escape($guideMarker)) {
            throw "La guía DiskPart Windows no contiene el paso protegido esperado: $guideMarker"
        }
    }
    $windowsGuideTopics = @('audit','packages','software','git','aliases','automation','automation-register','clean','storage','storage-partitions','storage-filesystems','storage-volumes','system','services','accounts','native','network','boot','registry','diagnostics','defaults','installable','settings','updates','containers','containers-lifecycle','containers-images','containers-volumes','containers-compose','kubernetes','ssh','connectivity','adb','utilities','privileges','wine','prefix','winslim')
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
    $winslimGuide = Run @('guide', 'gui', 'winslim')
    $winslimIndexVisible = $allGuides -match '(?m)^WinSlim:\s*$'
    if ($winslimGuide -match 'no tiene disponible la pantalla WinSlim/NSudo') {
        if ($allGuides -match '(?m)^WinSlim:\s*$') {
            throw 'El índice GUI anuncia WinSlim/NSudo sin WSCore ni un lanzador detectado.'
        }
    } else {
        if ($winslimGuide -notmatch 'Estado de WSCore y NSudo' -or -not $winslimIndexVisible) {
            throw 'La guía o el índice no reflejan el menú WinSlim condicional disponible.'
        }
    }
    $winslimStatus = Run @('winslim', 'status')
    if ($winslimStatus -notmatch 'WSCore|WinSlim' -or $winslimStatus -notmatch 'NSudo') {
        throw 'El estado WinSlim no informa de forma diferenciada WSCore y NSudo.'
    }
    if ($winslimStatus -notmatch [regex]::Escape($nsudoFixture)) {
        throw 'LTOOLS_NSUDO_PATH no detectó la ruta explícita del lanzador de prueba.'
    }
    $winslimGuideText = Run @('winslim', 'guide')
    foreach ($nsudoMarker in @('TrustedInstaller', 'drop-rights', '--integrity', '--all-privileges', '--yes')) {
        if ($winslimGuideText -notmatch [regex]::Escape($nsudoMarker)) {
            throw "La guía NSudo omite una identidad u opción compatible: $nsudoMarker"
        }
    }
    $nsudoMenuResult = Invoke-NativeProcess -FileName $Binary -Arguments @('winslim', 'menu') -InputText ("q" + [Environment]::NewLine) -TimeoutSeconds 15
    $nsudoMenu = [string]$nsudoMenuResult.Stdout + [string]$nsudoMenuResult.Stderr
    if ($nsudoMenuResult.ExitCode -ne 0 -or $nsudoMenu -notmatch 'Lanzar proceso con contexto elegido') {
        throw 'El menú WinSlim no muestra el asistente cuando se detecta un lanzador NSudo.'
    }
    $nsudoPlan = Run @('--dry-run', 'winslim', 'launch', '--identity', 'system', '--program', 'cmd.exe', '--arg', '/c', '--arg', 'ver', '--all-privileges', '--integrity', 'high', '--window', 'maximize', '--wait', '--console')
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
        'Arranque, EFI y cargador del sistema', 'Inspeccionar el Registro de Windows', 'Volver'
    )) {
        if ($nativeGuiGuide -notmatch [regex]::Escape($nativeOption)) {
            throw "La guía de herramientas nativas Windows no refleja su botón: $nativeOption"
        }
    }
    $windowsStorageGuiGuide = Run @('guide', 'gui', 'storage')
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
