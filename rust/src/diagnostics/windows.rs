use super::Probe;
use crate::common::{command_exists, command_output_detailed};

pub(super) fn collect(action: &str) -> Vec<Probe> {
    match action {
        "network" => vec![
            powershell_or_command_probe(
                "addresses",
                "Get-NetIPConfiguration | Format-Table -AutoSize",
                "ipconfig",
                &["/all"],
            ),
            powershell_or_command_probe(
                "routes",
                "Get-NetRoute | Format-Table -AutoSize",
                "route",
                &["print"],
            ),
            powershell_or_command_probe(
                "dns",
                "Get-DnsClientServerAddress | Format-Table -AutoSize",
                "ipconfig",
                &["/all"],
            ),
            powershell_or_command_probe(
                "listening",
                "Get-NetTCPConnection -State Listen | Sort-Object LocalPort | Format-Table -AutoSize",
                "netstat",
                &["-ano"],
            ),
        ],
        "hardware" => vec![
            powershell_probe(
                "operating-system",
                "Get-CimInstance Win32_OperatingSystem | Format-List Caption,Version,BuildNumber,LastBootUpTime",
            ),
            powershell_probe(
                "computer",
                "Get-CimInstance Win32_ComputerSystem | Format-List Manufacturer,Model,TotalPhysicalMemory",
            ),
            powershell_probe(
                "cpu",
                "Get-CimInstance Win32_Processor | Format-Table Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed -AutoSize",
            ),
            powershell_probe(
                "memory",
                "Get-CimInstance Win32_PhysicalMemory | Format-Table Manufacturer,Capacity,Speed -AutoSize",
            ),
            powershell_probe(
                "disks",
                "Get-Disk | Format-Table Number,FriendlyName,Size,BusType,HealthStatus -AutoSize",
            ),
            powershell_probe(
                "volumes",
                "Get-Volume | Format-Table DriveLetter,FileSystemLabel,FileSystem,Size,SizeRemaining,HealthStatus -AutoSize",
            ),
            powershell_probe(
                "graphics",
                "Get-CimInstance Win32_VideoController | Format-Table Name,DriverVersion,AdapterRAM -AutoSize",
            ),
        ],
        "users" => vec![
            command_probe("logged-in", "query", &["user"]),
            powershell_probe("identity", "$env:USERNAME"),
            powershell_probe(
                "computer-user",
                "(Get-CimInstance Win32_ComputerSystem).UserName",
            ),
        ],
        _ => vec![
            powershell_probe(
                "operating-system",
                "Get-CimInstance Win32_OperatingSystem | Format-List Caption,Version,BuildNumber,LastBootUpTime",
            ),
            powershell_probe(
                "computer",
                "Get-CimInstance Win32_ComputerSystem | Format-List Manufacturer,Model,TotalPhysicalMemory",
            ),
            powershell_probe(
                "memory",
                "Get-CimInstance Win32_OperatingSystem | Format-List FreePhysicalMemory,TotalVisibleMemorySize",
            ),
            powershell_probe(
                "system-drive",
                "Get-Volume -DriveLetter C | Format-List DriveLetter,FileSystem,Size,SizeRemaining,HealthStatus",
            ),
            powershell_probe("network", "Get-NetIPConfiguration | Format-Table -AutoSize"),
        ],
    }
}

fn command_probe(key: &'static str, command: &'static str, args: &[&str]) -> Probe {
    let result = command_output_detailed(command, args);
    let installed = command_exists(command);
    let (available, output, error, status_code, timed_out) = match result {
        Ok(value) => (
            installed && value.success(),
            value.stdout.trim_end().to_owned(),
            value.stderr.trim_end().to_owned(),
            value.status_code,
            value.timed_out,
        ),
        Err(error) => (false, String::new(), error.to_string(), None, false),
    };
    Probe {
        key,
        command,
        installed,
        available,
        output,
        error,
        status_code,
        timed_out,
    }
}

fn powershell_probe(key: &'static str, script: &'static str) -> Probe {
    let shell = ["powershell", "pwsh"]
        .into_iter()
        .find(|name| command_exists(name));
    let (available, output, error, status_code, timed_out) = match shell {
        Some(shell) => match command_output_detailed(
            shell,
            &["-NoProfile", "-NonInteractive", "-Command", script],
        ) {
            Ok(result) => (
                result.success(),
                result.stdout.trim_end().to_owned(),
                result.stderr.trim_end().to_owned(),
                result.status_code,
                result.timed_out,
            ),
            Err(error) => (
                false,
                String::new(),
                format!("no se pudo ejecutar {shell}: {error}"),
                None,
                false,
            ),
        },
        None => (
            false,
            String::new(),
            "no se encontró powershell ni pwsh".to_owned(),
            None,
            false,
        ),
    };
    Probe {
        key,
        command: "PowerShell",
        installed: shell.is_some(),
        available,
        output,
        error,
        status_code,
        timed_out,
    }
}

fn powershell_or_command_probe(
    key: &'static str,
    script: &'static str,
    fallback_command: &'static str,
    fallback_args: &[&str],
) -> Probe {
    let primary = powershell_probe(key, script);
    if primary.available {
        return primary;
    }
    let fallback = command_probe(key, fallback_command, fallback_args);
    if fallback.available {
        return fallback;
    }
    Probe {
        key,
        command: "PowerShell/fallback",
        available: false,
        installed: primary.installed || fallback.installed,
        output: String::new(),
        error: if !primary.error.is_empty() {
            primary.error.clone()
        } else {
            fallback.error.clone()
        },
        status_code: primary.status_code.or(fallback.status_code),
        timed_out: primary.timed_out || fallback.timed_out,
    }
}
