#!/usr/bin/env bash
# E2E de la carpeta publicable release/.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
RELEASE_DIR="$ROOT_DIR/release"
VERSION=""
REQUIRE_WINDOWS=0
REQUIRE_WINDOWS_EXECUTABLES=0
REQUIRE_APPIMAGE=1
REQUIRE_PACKAGE=1
SIGNATURE_PUBLIC_KEY_FILE=""
SIGNATURE_VERIFIER=""

die() { printf 'RELEASE E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }

usage() {
    cat <<EOF
Uso: $0 [opciones]

  --release-dir DIR   Carpeta publicable; por defecto: ./release.
  --version VERSION   Versión esperada; por defecto se lee de Cargo.toml.
  --require-windows   Exige EXE, EXE-CLI y ZIP Windows además de Linux.
  --require-windows-executables
                      Exige los dos EXE Windows; útil para validación GNU/Wine.
  --no-appimage       No exige los dos perfiles AppImage Linux.
  --no-package        No exige el tarball runtime Linux.
  --signature-public-key-file FICHERO
                      Clave pública para verificar SHA256SUMS.txt.sig.
  --signature-verifier FICHERO
                      Backend LTools/WinSlim-Tools que verifica la firma.
  -h, --help          Muestra esta ayuda.
EOF
}

while (($#)); do
    case "$1" in
        --release-dir) (($# >= 2)) || die '--release-dir necesita una ruta'; RELEASE_DIR="$2"; shift ;;
        --version) (($# >= 2)) || die '--version necesita un valor'; VERSION="$2"; shift ;;
        --require-windows) REQUIRE_WINDOWS=1 ;;
        --require-windows-executables) REQUIRE_WINDOWS_EXECUTABLES=1 ;;
        --no-appimage) REQUIRE_APPIMAGE=0 ;;
        --no-package) REQUIRE_PACKAGE=0 ;;
        --signature-public-key-file) (($# >= 2)) || die '--signature-public-key-file necesita una ruta'; SIGNATURE_PUBLIC_KEY_FILE="$2"; shift ;;
        --signature-verifier) (($# >= 2)) || die '--signature-verifier necesita una ruta'; SIGNATURE_VERIFIER="$2"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) die "argumento desconocido: $1" ;;
    esac
    shift
done

if [[ -z "$VERSION" ]]; then
    VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/rust/Cargo.toml" | head -n1)"
fi
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] || die "versión inválida: $VERSION"
[[ -d "$RELEASE_DIR" ]] || die "no existe la carpeta release: $RELEASE_DIR"
command -v jq >/dev/null 2>&1 || die 'jq es necesario para validar la release'
command -v sha256sum >/dev/null 2>&1 || die 'sha256sum es necesario para validar la release'
command -v stat >/dev/null 2>&1 || die 'stat es necesario para validar la release'

manifest="$RELEASE_DIR/ltools-release.json"
[[ -s "$manifest" ]] || die 'falta ltools-release.json'
jq -e --arg version "$VERSION" \
    '.schema == "ltools-release-v1" and (.application == "LTools" or .application == "WinSlim-Tools") and
     .version == $version and .hash_algorithm == "sha256" and
     (.artifacts | type == "array" and length > 0)' "$manifest" >/dev/null \
    || die 'el manifiesto no supera el contrato de release'

for json in \
    ltools-capabilities.json \
    ltools-terminal.json \
    ltools-project.json; do
    [[ -s "$RELEASE_DIR/$json" ]] || die "falta $json"
    jq empty "$RELEASE_DIR/$json" >/dev/null || die "$json no es JSON válido"
done
for schema in \
    ltools-terminal.schema.json \
    ltools-project.schema.json \
    ltools-release.schema.json; do
    [[ -s "$RELEASE_DIR/$schema" ]] || die "falta $schema"
    jq empty "$RELEASE_DIR/$schema" >/dev/null || die "$schema no es JSON válido"
done
ok 'JSON y esquemas publicables válidos'

required_linux=()
if (( REQUIRE_APPIMAGE )); then
    required_linux+=(
        "ltools-$VERSION-linux-x86_64.AppImage"
        "ltools-$VERSION-linux-x86_64-cli.AppImage"
    )
fi
if (( REQUIRE_PACKAGE )); then
    required_linux+=("ltools-$VERSION-linux-x86_64.tar.gz")
fi
for artifact in "${required_linux[@]}"; do
    [[ -s "$RELEASE_DIR/$artifact" ]] || die "falta el artefacto Linux $artifact"
done
if (( ${#required_linux[@]} )); then
    ok 'artefactos Linux solicitados presentes'
else
    ok 'no se han solicitado artefactos binarios Linux adicionales'
fi

if (( REQUIRE_WINDOWS )); then
    required_windows=(
        "ltools-$VERSION-windows-x86_64.exe"
        "ltools-$VERSION-windows-x86_64-cli.exe"
        "ltools-$VERSION-windows-x86_64.zip"
    )
    for artifact in "${required_windows[@]}"; do
        [[ -s "$RELEASE_DIR/$artifact" ]] || die "falta el artefacto Windows $artifact"
    done
    ok 'artefactos Windows principal, CLI y ZIP presentes'
fi

if (( REQUIRE_WINDOWS_EXECUTABLES )); then
    for artifact in \
        "ltools-$VERSION-windows-x86_64.exe" \
        "ltools-$VERSION-windows-x86_64-cli.exe"; do
        [[ -s "$RELEASE_DIR/$artifact" ]] || die "falta el ejecutable Windows $artifact"
    done
    ok 'ejecutables Windows principal y CLI presentes'
fi

while IFS= read -r -d '' file; do
    name="$(basename -- "$file")"
    case "$name" in
        ltools-"$VERSION"-linux-*|ltools-"$VERSION"-windows-*) ;;
        *) continue ;;
    esac
    size="$(stat -c '%s' -- "$file")"
    hash="$(sha256sum -- "$file" | awk '{print $1}')"
    jq -e --arg name "$name" --arg hash "$hash" --argjson size "$size" \
        'any(.artifacts[]; .filename == $name and .size_bytes == $size and .sha256 == $hash)' \
        "$manifest" >/dev/null \
        || die "el manifiesto no coincide con $name"
done < <(find "$RELEASE_DIR" -maxdepth 1 -type f -print0 | sort -z)
ok 'cada artefacto reconocido coincide con tamaño y SHA-256 del manifiesto'

checksums="$RELEASE_DIR/SHA256SUMS.txt"
signature="$RELEASE_DIR/SHA256SUMS.txt.sig"
if [[ -e "$checksums" ]]; then
    [[ -s "$checksums" ]] || die 'SHA256SUMS.txt está vacío'
    declare -A listed=()
    checksum_count=0
    while IFS= read -r line || [[ -n "$line" ]]; do
        [[ "${#line}" -gt 66 && "${line:0:64}" =~ ^[[:xdigit:]]{64}$ && "${line:64:2}" == '  ' ]] ||
            die "línea inválida en SHA256SUMS.txt"
        expected="${line:0:64}"
        name="${line:66}"
        [[ -n "$name" && "$name" != */* && "$name" != .* && "$name" != *'..'* ]] || die "nombre inseguro en SHA256SUMS.txt: $name"
        [[ -f "$RELEASE_DIR/$name" ]] || die "SHA256SUMS.txt referencia un fichero inexistente: $name"
        [[ -z "${listed[$name]+yes}" ]] || die "SHA256SUMS.txt contiene el fichero duplicado: $name"
        listed[$name]=1
        actual="$(sha256sum -- "$RELEASE_DIR/$name" | awk '{print $1}')"
        [[ "${actual,,}" == "${expected,,}" ]] || die "SHA256SUMS.txt no coincide para $name"
        checksum_count=$((checksum_count + 1))
    done < "$checksums"
    expected_count="$(find "$RELEASE_DIR" -maxdepth 1 -type f ! -name 'SHA256SUMS.txt' ! -name 'SHA256SUMS.txt.sig' -printf '%f\n' | wc -l)"
    [[ "$checksum_count" == "$expected_count" ]] || die "SHA256SUMS.txt no cubre todos los ficheros publicables"
    ok 'SHA256SUMS.txt cubre todos los artefactos y sus hashes coinciden'
    if [[ -e "$signature" ]]; then
        [[ -s "$signature" ]] || die 'SHA256SUMS.txt.sig está vacío'
        [[ -n "$SIGNATURE_PUBLIC_KEY_FILE" && -s "$SIGNATURE_PUBLIC_KEY_FILE" ]] || die 'hay firma, pero falta --signature-public-key-file para verificarla'
        [[ -x "$SIGNATURE_VERIFIER" ]] || die 'hay firma, pero falta --signature-verifier ejecutable'
        "$SIGNATURE_VERIFIER" release-signature \
            --manifest "$checksums" --signature "$signature" \
            --public-key-file "$SIGNATURE_PUBLIC_KEY_FILE" --verify >/dev/null \
            || die 'la firma Ed25519 de SHA256SUMS.txt no es válida'
        ok 'firma Ed25519 de SHA256SUMS.txt verificada'
    fi
fi

if (( REQUIRE_WINDOWS || REQUIRE_WINDOWS_EXECUTABLES )); then
    jq -e '([.artifacts[].platform] | index("linux")) != null and
           ([.artifacts[].platform] | index("windows")) != null' "$manifest" >/dev/null \
        || die 'el manifiesto final no contiene ambas plataformas'
    ok 'manifiesto unificado Linux/Windows'
fi

printf 'E2E de release completado correctamente: %s\n' "$RELEASE_DIR"
