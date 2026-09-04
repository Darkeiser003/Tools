use super::Probe;
use crate::common::command_exists;
use std::process::Command;

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
    let result = Command::new(command)
        .args(args)
        .output()
        .ok()
        .filter(|result| result.status.success())
        .map(|result| {
            String::from_utf8_lossy(&result.stdout)
                .trim_end()
                .to_string()
        });
    Probe {
        key,
        command,
        available: command_exists(command) && result.is_some(),
        output: result.unwrap_or_default(),
    }
}

fn powershell_probe(key: &'static str, script: &'static str) -> Probe {
    let shell = ["powershell", "pwsh"]
        .into_iter()
        .find(|name| command_exists(name));
    let result = shell
        .and_then(|shell| {
            Command::new(shell)
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .output()
                .ok()
        })
        .filter(|result| result.status.success())
        .map(|result| {
            String::from_utf8_lossy(&result.stdout)
                .trim_end()
                .to_string()
        });
    Probe {
        key,
        command: "PowerShell",
        available: shell.is_some() && result.is_some(),
        output: result.unwrap_or_default(),
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
        output: String::new(),
    }
}
