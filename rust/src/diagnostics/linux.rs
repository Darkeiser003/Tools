use super::Probe;
use crate::common::{command_exists, command_output_detailed};
use std::fs;

pub(super) fn collect(action: &str) -> Vec<Probe> {
    match action {
        "network" => vec![
            probe("addresses", "ip", &["-brief", "address"]),
            probe("routes", "ip", &["route"]),
            probe("dns", "resolvectl", &["status"]),
            probe("listening", "ss", &["-tuln"]),
        ],
        "hardware" => vec![
            probe("kernel", "uname", &["-srmo"]),
            probe("cpu", "lscpu", &[]),
            probe("memory", "free", &["-h"]),
            probe(
                "block-devices",
                "lsblk",
                &[
                    "-e7",
                    "-o",
                    "NAME,SIZE,TYPE,FSTYPE,MOUNTPOINTS,FSAVAIL,FSUSE%",
                ],
            ),
            probe("pci", "lspci", &[]),
            probe("usb", "lsusb", &[]),
        ],
        "users" => vec![
            probe("logged-in", "who", &[]),
            probe("user-sessions", "loginctl", &["list-users", "--no-legend"]),
            probe("identity", "id", &["-un"]),
        ],
        _ => vec![
            probe_file("operating-system", "/etc/os-release"),
            probe("kernel", "uname", &["-srmo"]),
            probe("uptime", "uptime", &["-p"]),
            probe("memory", "free", &["-h"]),
            probe("root-filesystem", "df", &["-h", "/"]),
            probe("network", "ip", &["-brief", "address"]),
        ],
    }
}

fn probe(key: &'static str, command: &'static str, args: &[&str]) -> Probe {
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

fn probe_file(key: &'static str, path: &'static str) -> Probe {
    let installed = fs::metadata(path).is_ok();
    match fs::read_to_string(path) {
        Ok(output) => Probe {
            key,
            command: path,
            available: true,
            installed,
            output,
            error: String::new(),
            status_code: None,
            timed_out: false,
        },
        Err(error) => Probe {
            key,
            command: path,
            available: false,
            installed,
            output: String::new(),
            error: error.to_string(),
            status_code: None,
            timed_out: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::probe_file;

    #[test]
    fn unreadable_probe_files_keep_the_read_error_visible() {
        let probe = probe_file("directory-fixture", "/etc");
        assert!(probe.installed, "the fixture directory should exist");
        assert!(
            !probe.available,
            "a directory is not readable as a text file"
        );
        assert!(probe.output.is_empty());
        assert!(
            !probe.error.is_empty(),
            "the probe must explain why the file could not be read"
        );
    }
}
