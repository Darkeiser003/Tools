//! Parser de campos de argumentos para comandos nativos, sin evaluación de shell.

/// Divide una línea en argumentos sin ejecutar expansiones de shell.
///
/// Admite espacios y comillas simples/dobles. No expande variables, comodines,
/// tuberías ni sustituciones. Limita el tamaño de entrada a 32 KiB, cada token
/// a 4 KiB y el resultado a 63 argumentos.
pub fn split_native_argument_line(input: &str) -> Result<Vec<String>, String> {
    if input.len() > 32_768 || input.chars().any(char::is_control) {
        return Err(
            "los argumentos de gh superan el límite permitido o contienen controles".into(),
        );
    }
    let mut result = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;
    for character in input.chars() {
        if escaped {
            token.push(character);
            if token.len() > 4_096 {
                return Err("un argumento de gh supera el máximo de 4096 bytes".into());
            }
            escaped = false;
            started = true;
            continue;
        }
        match quote {
            Some('\'') if character != '\'' => token.push(character),
            Some('"') if character == '\\' => escaped = true,
            Some(active) if character == active => quote = None,
            Some(_) => token.push(character),
            None if character == '\\' => {
                escaped = true;
                started = true;
            }
            None if character == '\'' || character == '"' => {
                quote = Some(character);
                started = true;
            }
            None if character.is_whitespace() => {
                if started {
                    result.push(std::mem::take(&mut token));
                    started = false;
                }
            }
            None => {
                token.push(character);
                started = true;
            }
        }
        if token.len() > 4_096 {
            return Err("un argumento de gh supera el máximo de 4096 bytes".into());
        }
    }
    if escaped {
        return Err("los argumentos de gh terminan con una barra invertida sin escapar".into());
    }
    if quote.is_some() {
        return Err("las comillas de los argumentos de gh no están cerradas".into());
    }
    if started {
        result.push(token);
    }
    if result.len() > 63 {
        return Err("la GUI admite como máximo 63 argumentos nativos de gh".into());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::split_native_argument_line;

    #[test]
    fn parses_quotes_and_never_expands_shell_syntax() {
        assert_eq!(
            split_native_argument_line(
                "issue --title 'Fix spaces' --body \"$HOME; echo no\" escaped\\ value"
            )
            .unwrap(),
            [
                "issue",
                "--title",
                "Fix spaces",
                "--body",
                "$HOME; echo no",
                "escaped value"
            ]
        );
    }

    #[test]
    fn preserves_empty_quoted_argument_and_unicode() {
        assert_eq!(
            split_native_argument_line("'' \"áéí\"").unwrap(),
            ["", "áéí"]
        );
    }

    #[test]
    fn rejects_unclosed_quotes_escapes_and_control_characters() {
        assert!(split_native_argument_line("issue 'unfinished").is_err());
        assert!(split_native_argument_line("issue trailing\\").is_err());
        assert!(split_native_argument_line("issue\nlist").is_err());
    }

    #[test]
    fn enforces_input_token_and_argument_limits() {
        assert!(split_native_argument_line(&"x".repeat(32_769)).is_err());
        assert!(split_native_argument_line(&format!("'{}'", "x".repeat(4_097))).is_err());
        assert!(split_native_argument_line(&vec!["x"; 64].join(" ")).is_err());
        assert_eq!(
            split_native_argument_line(&vec!["x"; 63].join(" "))
                .unwrap()
                .len(),
            63
        );
    }

    #[test]
    fn escaped_characters_cannot_bypass_the_token_size_limit() {
        let input = format!("{}\\y", "x".repeat(4_096));
        assert!(split_native_argument_line(&input).is_err());
    }
}
