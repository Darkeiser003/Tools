//! Argument helpers shared by CLI dispatch and policy checks.
//!
//! A switch's value is data, never a subcommand. Keeping this in one place
//! prevents paths, service names, labels, and package names from changing
//! dispatch or privilege decisions just because they match a command word.

const OPTIONS_WITH_VALUES: &[&str] = &[
    "--apk",
    "--app",
    "--artifacts-dir",
    "--arch",
    "--alignment",
    "--arg",
    "--args",
    "--branch",
    "--candidate",
    "--category",
    "--channel",
    "--client",
    "--columns",
    "--command",
    "--comment",
    "--connection",
    "--context",
    "--count",
    "--cwd",
    "--date",
    "--deployment",
    "--dest",
    "--destination",
    "--description",
    "--detail",
    "--device",
    "--depth",
    "--directory",
    "--engine",
    "--entry",
    "--exclude",
    "--expiredate",
    "--file",
    "--filesystem",
    "--filter",
    "--flag",
    "--format",
    "--formula",
    "--full-name",
    "--from",
    "--fs",
    "--get",
    "--gid",
    "--group",
    "--groups",
    "--header-backup-file",
    "--home",
    "--hours",
    "--id",
    "--identity",
    "--image",
    "--include",
    "--interface",
    "--key",
    "--kind",
    "--label",
    "--lang",
    "--language",
    "--level",
    "--limit",
    "--local-port",
    "--login",
    "--manager",
    "--manifest",
    "--max-children",
    "--maxdays",
    "--member",
    "--members",
    "--message",
    "--min-size-mb",
    "--mindays",
    "--mode",
    "--mount",
    "--mountpoint",
    "--name",
    "--namespace",
    "--network",
    "--new-name",
    "--number",
    "--operation",
    "--options",
    "--out",
    "--output",
    "--package",
    "--password-file",
    "--password-never-expires",
    "--path",
    "--plan",
    "--port",
    "--ports",
    "--primary-group",
    "--profile",
    "--program",
    "--property",
    "--provider",
    "--qf",
    "--raid-devices",
    "--remote",
    "--replicas",
    "--repo",
    "--request-timeout",
    "--repository",
    "--resource",
    "--root",
    "--scope",
    "--script",
    "--search",
    "--serial",
    "--service",
    "--shell",
    "--signature",
    "--since",
    "--size",
    "--sort",
    "--source",
    "--start",
    "--state",
    "--tag",
    "--target",
    "--timeout",
    "--title",
    "--tool",
    "--type",
    "--uid",
    "--unit",
    "--url",
    "--value",
    "--vg",
    "--volume",
    "--working-directory",
];

pub(crate) fn option_takes_value(value: &str) -> bool {
    !value.contains('=') && OPTIONS_WITH_VALUES.contains(&value)
}

pub(crate) fn positionals(args: &[String]) -> Vec<&str> {
    let mut result = Vec::new();
    let mut skip_value = false;
    let mut options_ended = false;
    for value in args {
        if skip_value {
            skip_value = false;
            continue;
        }
        if options_ended {
            result.push(value.as_str());
            continue;
        }
        if value == "--" {
            options_ended = true;
            continue;
        }
        if value.starts_with('-') {
            if option_takes_value(value) {
                skip_value = true;
            }
            continue;
        }
        result.push(value.as_str());
    }
    result
}

pub(crate) fn option_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let equals_prefix = format!("{name}=");
    for (index, value) in args.iter().enumerate() {
        if let Some(value) = value.strip_prefix(&equals_prefix) {
            return Some(value);
        }
        if value == name {
            return args.get(index + 1).map(String::as_str);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{option_value, positionals};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn option_values_are_never_subcommands() {
        assert_eq!(positionals(&args(&["--path", "manage", "map"])), ["map"]);
        assert_eq!(
            positionals(&args(&["manage", "--path", "menu", "delete"])),
            ["manage", "delete"]
        );
        assert_eq!(
            positionals(&args(&["mkfs", "--label", "map", "--device", "/dev/sdb1"])),
            ["mkfs"]
        );
        assert_eq!(
            positionals(&args(&["--", "delete", "target"])),
            ["delete", "target"]
        );
    }

    #[test]
    fn option_value_accepts_separate_and_equals_forms() {
        assert_eq!(
            option_value(&args(&["--path", "/tmp/a"]), "--path"),
            Some("/tmp/a")
        );
        assert_eq!(
            option_value(&args(&["--path=/tmp/b"]), "--path"),
            Some("/tmp/b")
        );
    }

    #[test]
    fn command_specific_values_are_not_mistaken_for_actions_or_global_flags() {
        assert_eq!(
            positionals(&args(&[
                "--alignment",
                "minimal",
                "--flag",
                "menu",
                "storage",
                "mkfs",
            ])),
            ["storage", "mkfs"]
        );
        assert_eq!(
            positionals(&args(&[
                "--full-name",
                "--elevate",
                "--primary-group",
                "wheel",
                "accounts",
                "modify",
            ])),
            ["accounts", "modify"]
        );
        assert_eq!(
            positionals(&args(&[
                "--manifest",
                "--no-elevate",
                "--repository",
                "owner/repo",
                "release-manifest",
            ])),
            ["release-manifest"]
        );
        // --user is a value-taking account option in one command family but
        // a boolean systemd-scope switch in another. A generic positional
        // parser must not consume the following system action as its value.
        assert_eq!(
            positionals(&args(&["--user", "service", "restart", "demo.service"])),
            ["service", "restart", "demo.service"]
        );
    }
}
