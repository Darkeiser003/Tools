#!/usr/bin/env bash
# Valida sintaxis de los scripts de build y pruebas sin compilar Rust.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
failures=0

while IFS= read -r -d '' file; do
    if ! bash -n "$file"; then
        printf 'SCRIPT SYNTAX ERROR: Bash: %s\n' "${file#$ROOT_DIR/}" >&2
        failures=$((failures + 1))
    fi
done < <(rg --files --hidden -0 -g '*.sh' -g '!.git/**' -g '!dist/**' -g '!rust/target/**' "$ROOT_DIR")

bash "$ROOT_DIR/tests/publish-release.sh"
bash "$ROOT_DIR/tests/update-signing.sh"
bash "$ROOT_DIR/tests/temp-cleaner.sh"
if command -v pwsh >/dev/null 2>&1; then
    pwsh -NoLogo -NoProfile -NonInteractive -File "$ROOT_DIR/tests/build-state.ps1"
fi

if command -v pwsh >/dev/null 2>&1; then
    while IFS= read -r -d '' file; do
        if ! LTOOLS_PARSE_FILE="$file" pwsh -NoLogo -NoProfile -NonInteractive -Command '
            $tokens = $null
            $parseErrors = $null
            [void][System.Management.Automation.Language.Parser]::ParseFile(
                $env:LTOOLS_PARSE_FILE, [ref]$tokens, [ref]$parseErrors)
            if ($parseErrors.Count -gt 0) {
                $parseErrors | ForEach-Object { [Console]::Error.WriteLine($_.ToString()) }
                exit 1
            }
        ' >/dev/null; then
            printf 'SCRIPT SYNTAX ERROR: PowerShell: %s\n' "${file#$ROOT_DIR/}" >&2
            failures=$((failures + 1))
        fi
    done < <(rg --files --hidden -0 -g '*.ps1' -g '!.git/**' -g '!dist/**' -g '!rust/target/**' "$ROOT_DIR")
    pwsh -NoLogo -NoProfile -NonInteractive -File "$ROOT_DIR/tests/build-publish.ps1"
else
    printf 'SKIP: pwsh no está instalado; sintaxis PowerShell se valida en el builder Windows.\n'
fi

if ((failures)); then
    printf '%s error(es) de sintaxis en scripts.\n' "$failures" >&2
    exit 1
fi
printf 'Sintaxis correcta en scripts Bash y PowerShell disponibles.\n'
