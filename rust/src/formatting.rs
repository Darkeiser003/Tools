//! Formateadores de salida humana compartidos por CLI y GUI.
//!
//! Las herramientas del sistema suelen devolver texto pensado para una
//! terminal concreta. Este módulo conserva todos los valores, pero convierte
//! las filas en tablas estables para que una salida se pueda leer igual en la
//! CLI, en un modal y en una ventana estrecha con scroll horizontal.

/// Renderiza una tabla sin envolver las celdas. Las líneas largas se dejan
/// completas para que la interfaz pueda desplazarlas horizontalmente.
pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let columns = headers.len();
    if columns == 0 {
        return String::new();
    }

    let mut widths = headers
        .iter()
        .map(|header| display_width(header))
        .collect::<Vec<_>>();
    for row in rows {
        for (index, value) in row.iter().take(columns).enumerate() {
            widths[index] = widths[index].max(display_width(value));
        }
    }

    let mut output = String::new();
    output.push_str(&render_row(
        &headers
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>(),
        &widths,
    ));
    output.push('\n');
    output.push_str(
        &widths
            .iter()
            .map(|width| "─".repeat(*width))
            .collect::<Vec<_>>()
            .join("  "),
    );
    for row in rows {
        output.push('\n');
        output.push_str(&render_row(row, &widths));
    }
    output
}

/// Convierte una salida con columnas separadas por espacios en una tabla.
/// Las primeras columnas son valores simples y la última puede contener
/// espacios escapados o texto compuesto.
pub fn whitespace_table(output: &str, headers: &[&str], skip_header: bool) -> String {
    let mut rows = Vec::new();
    for line in output.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if skip_header
            && line
                .split_whitespace()
                .next()
                .is_some_and(|value| value.eq_ignore_ascii_case(headers[0]))
        {
            continue;
        }
        rows.push(split_row(line, headers.len()));
    }
    table(headers, &rows)
}

/// Formatea la salida de findmnt -o SOURCE,TARGET,FSTYPE,OPTIONS.
pub fn findmnt(output: &str) -> String {
    whitespace_table(output, &["SOURCE", "TARGET", "FSTYPE", "OPTIONS"], true)
}

/// Formatea df en sus dos variantes habituales: con o sin columna TYPE.
pub fn df(output: &str) -> String {
    let header = output
        .lines()
        .find(|line| line.split_whitespace().next() == Some("Filesystem"));
    let with_type = header.is_some_and(|line| line.split_whitespace().any(|value| value == "Type"));
    let headers = if with_type {
        &[
            "FILESYSTEM",
            "TYPE",
            "SIZE",
            "USED",
            "AVAIL",
            "USE%",
            "MOUNTED ON",
        ][..]
    } else {
        &["FILESYSTEM", "SIZE", "USED", "AVAIL", "USE%", "MOUNTED ON"][..]
    };
    whitespace_table(output, headers, true)
}

/// Formatea la salida de blkid como una tabla de propiedades por dispositivo.
pub fn blkid(output: &str) -> String {
    let mut rows = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let Some((device, properties)) = line.split_once(':') else {
            rows.push(vec![
                line.trim().to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ]);
            continue;
        };
        let mut row = vec![
            device.trim().to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        for (key, value) in parse_properties(properties) {
            match key {
                "UUID" => row[1] = value,
                "LABEL" => row[2] = value,
                "TYPE" => row[3] = value,
                "PARTUUID" => row[4] = value,
                _ => {}
            }
        }
        rows.push(row);
    }
    table(&["DEVICE", "UUID", "LABEL", "TYPE", "PARTUUID"], &rows)
}

fn split_row(line: &str, columns: usize) -> Vec<String> {
    let mut fields = line.split_whitespace();
    let mut row = Vec::with_capacity(columns);
    for _ in 0..columns.saturating_sub(1) {
        row.push(fields.next().unwrap_or_default().to_string());
    }
    row.push(fields.collect::<Vec<_>>().join(" "));
    row
}

fn parse_properties(input: &str) -> Vec<(&str, String)> {
    let mut index = 0;
    let mut properties = Vec::new();
    while index < input.len() {
        while let Some((character, size)) = input[index..].chars().next().map(|c| (c, c.len_utf8()))
        {
            if !character.is_whitespace() {
                break;
            }
            index += size;
        }
        let key_start = index;
        while let Some((character, size)) = input[index..].chars().next().map(|c| (c, c.len_utf8()))
        {
            if character == '=' {
                break;
            }
            if character.is_whitespace() {
                break;
            }
            index += size;
        }
        if index >= input.len() || !input[index..].starts_with('=') {
            break;
        }
        let key_end = index;
        index += 1;
        let value = if input[index..].starts_with('"') {
            index += 1;
            let mut value = String::new();
            while let Some((character, size)) =
                input[index..].chars().next().map(|c| (c, c.len_utf8()))
            {
                if character == '\\' {
                    index += size;
                    if let Some((escaped, escaped_size)) =
                        input[index..].chars().next().map(|c| (c, c.len_utf8()))
                    {
                        value.push(escaped);
                        index += escaped_size;
                    }
                } else if character == '"' {
                    index += size;
                    break;
                } else {
                    value.push(character);
                    index += size;
                }
            }
            value
        } else {
            let value_start = index;
            while let Some((character, size)) =
                input[index..].chars().next().map(|c| (c, c.len_utf8()))
            {
                if character.is_whitespace() {
                    break;
                }
                index += size;
            }
            input[value_start..index].to_string()
        };
        let key = &input[key_start..key_end];
        properties.push((key, value));
    }
    properties
}

fn render_row(row: &[String], widths: &[usize]) -> String {
    widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let value = row.get(index).map(String::as_str).unwrap_or_default();
            if index + 1 == widths.len() {
                value.to_string()
            } else {
                format!("{value:<width$}")
            }
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn display_width(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monta_tabla_estable_y_alineada() {
        let result = table(
            &["ORIGEN", "DESTINO"],
            &[
                vec!["/dev/sda1".into(), "/home".into()],
                vec!["tmpfs".into(), "/run/user/1000".into()],
            ],
        );
        assert!(result.contains("ORIGEN"));
        assert!(result.contains("/dev/sda1  /home"));
        assert!(result.contains("──────"));
    }

    #[test]
    fn formatea_findmnt_sin_perder_opciones() {
        let result = findmnt(
            "SOURCE TARGET FSTYPE OPTIONS\n/dev/sda1 / ext4 rw,noatime\ntmpfs /run tmpfs rw,nosuid",
        );
        assert!(result.contains("SOURCE"));
        assert!(result.contains("/dev/sda1"));
        assert!(result.contains("rw,noatime"));
    }

    #[test]
    fn formatea_df_con_y_sin_tipo() {
        let plain = df("Filesystem Size Used Avail Use% Mounted on\n/dev/sda1 100G 40G 60G 40% /");
        let typed = df(
            "Filesystem Type Size Used Avail Use% Mounted on\n/dev/sda1 ext4 100G 40G 60G 40% /",
        );
        assert!(plain.contains("MOUNTED ON"));
        assert!(typed.contains("TYPE"));
        assert!(typed.contains("ext4"));
    }

    #[test]
    fn formatea_blkid_en_propiedades() {
        let result = blkid("/dev/sda1: UUID=\"abc\" TYPE=\"ext4\" LABEL=\"Datos con espacios\"");
        assert!(result.contains("DEVICE"));
        assert!(result.contains("abc"));
        assert!(result.contains("Datos con espacios"));
    }
}
