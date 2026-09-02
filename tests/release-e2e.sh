#!/usr/bin/env bash
# E2E de la carpeta publicable release/.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
RELEASE_DIR="$ROOT_DIR/release"
VERSION=""
REQUIRE_WINDOWS=0

die() { printf 'RELEASE E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }

usage() {
    cat <<EOF
Uso: $0 [opciones]

  --release-dir DIR   Carpeta publicable; por defecto: ./release.
  --version VERSION   Versión esperada; por defecto se lee de Cargo.toml.
  --require-windows   Exige EXE, EXE-CLI y ZIP Windows además de Linux.
  -h, --help          Muestra esta ayuda.
EOF
}

while (($#)); do
    case "$1" in
        --release-dir) (($# >= 2)) || die '--release-dir necesita una ruta'; RELEASE_DIR="$2"; shift ;;
        --version) (($# >= 2)) || die '--version necesita un valor'; VERSION="$2"; shift ;;
        --require-windows) REQUIRE_WINDOWS=1 ;;
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
    '.schema == "ltools-release-v1" and .application == "LTools" and
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

required_linux=(
    "ltools-$VERSION-linux-x86_64.AppImage"
    "ltools-$VERSION-linux-x86_64-cli.AppImage"
    "ltools-$VERSION-linux-x86_64.tar.gz"
)
for artifact in "${required_linux[@]}"; do
    [[ -s "$RELEASE_DIR/$artifact" ]] || die "falta el artefacto Linux $artifact"
done
ok 'artefactos Linux principal y CLI presentes'

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

if (( REQUIRE_WINDOWS )); then
    jq -e '([.artifacts[].platform] | index("linux")) != null and
           ([.artifacts[].platform] | index("windows")) != null' "$manifest" >/dev/null \
        || die 'el manifiesto final no contiene ambas plataformas'
    ok 'manifiesto unificado Linux/Windows'
fi

printf 'E2E de release completado correctamente: %s\n' "$RELEASE_DIR"
