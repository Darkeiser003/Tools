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
REQUIRE_SSH_SIGNATURE=0
LINUX_ARCH="$(uname -m)"
WINDOWS_ARCH=x86_64
SIGNATURE_PUBLIC_KEY_FILE=""
SIGNATURE_VERIFIER=""
SSH_SIGNATURE_PUBLIC_KEY_FILE=""
SSH_SIGNATURE_IDENTITY=""

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
  --linux-arch ARCH    Arquitectura Linux esperada (por defecto: uname -m).
  --windows-arch ARCH  Arquitectura Windows del paquete esperado (por defecto: x86_64).
  --no-appimage       No exige los dos perfiles AppImage Linux.
  --no-package        No exige el tarball runtime Linux.
  --require-ssh-signature
                      Exige y verifica la firma OpenSSH de los checksums.
  --signature-public-key-file FICHERO
                      Clave pública para verificar SHA256SUMS.txt.sig.
  --signature-verifier FICHERO
                      Backend LTools/WinSlim-Tools que verifica la firma.
  --ssh-public-key-file FICHERO
                      Clave OpenSSH pública que valida SHA256SUMS.txt.sshsig.
  --ssh-identity IDENTIDAD
                      Principal OpenSSH para validar la firma de release.
  -h, --help          Muestra esta ayuda.
EOF
}

while (($#)); do
    case "$1" in
        --release-dir) (($# >= 2)) || die '--release-dir necesita una ruta'; RELEASE_DIR="$2"; shift ;;
        --version) (($# >= 2)) || die '--version necesita un valor'; VERSION="$2"; shift ;;
        --require-windows) REQUIRE_WINDOWS=1 ;;
        --require-windows-executables) REQUIRE_WINDOWS_EXECUTABLES=1 ;;
        --linux-arch) (($# >= 2)) || die '--linux-arch necesita una arquitectura'; LINUX_ARCH="$2"; shift ;;
        --windows-arch) (($# >= 2)) || die '--windows-arch necesita una arquitectura'; WINDOWS_ARCH="$2"; shift ;;
        --no-appimage) REQUIRE_APPIMAGE=0 ;;
        --no-package) REQUIRE_PACKAGE=0 ;;
        --require-ssh-signature) REQUIRE_SSH_SIGNATURE=1 ;;
        --signature-public-key-file) (($# >= 2)) || die '--signature-public-key-file necesita una ruta'; SIGNATURE_PUBLIC_KEY_FILE="$2"; shift ;;
        --signature-verifier) (($# >= 2)) || die '--signature-verifier necesita una ruta'; SIGNATURE_VERIFIER="$2"; shift ;;
        --ssh-public-key-file) (($# >= 2)) || die '--ssh-public-key-file necesita una ruta'; SSH_SIGNATURE_PUBLIC_KEY_FILE="$2"; shift ;;
        --ssh-identity) (($# >= 2)) || die '--ssh-identity necesita un valor'; SSH_SIGNATURE_IDENTITY="$2"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) die "argumento desconocido: $1" ;;
    esac
    shift
done

if [[ -z "$VERSION" ]]; then
    VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/rust/Cargo.toml" | head -n1)"
fi
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] || die "versión inválida: $VERSION"
[[ "$LINUX_ARCH" =~ ^[A-Za-z0-9_]+$ ]] || die "arquitectura Linux no válida: $LINUX_ARCH"
[[ "$WINDOWS_ARCH" =~ ^[A-Za-z0-9_]+$ ]] || die "arquitectura Windows no válida: $WINDOWS_ARCH"
[[ -d "$RELEASE_DIR" && ! -L "$RELEASE_DIR" ]] || die "no existe o no es una carpeta release normal: $RELEASE_DIR"
[[ -f "$RELEASE_DIR/LICENSE" && ! -L "$RELEASE_DIR/LICENSE" ]] || die 'falta la licencia MIT del proyecto en la release'
grep -Fq 'MIT License' "$RELEASE_DIR/LICENSE" || die 'la licencia del proyecto en release no identifica MIT'
grep -Fq 'Darkeiser003' "$RELEASE_DIR/LICENSE" || die 'la licencia del proyecto no identifica al titular'
command -v jq >/dev/null 2>&1 || die 'jq es necesario para validar la release'
command -v sha256sum >/dev/null 2>&1 || die 'sha256sum es necesario para validar la release'
command -v stat >/dev/null 2>&1 || die 'stat es necesario para validar la release'

manifest="$RELEASE_DIR/ltools-release.json"
[[ -f "$manifest" && ! -L "$manifest" && -s "$manifest" ]] || die 'falta ltools-release.json regular'
jq -e --arg version "$VERSION" \
    '.schema == "ltools-release-v1" and (.application == "LTools" or .application == "WinSlim-Tools") and
     .version == $version and .hash_algorithm == "sha256" and
     (.artifacts | type == "array" and length > 0)' "$manifest" >/dev/null \
    || die 'el manifiesto no supera el contrato de release'

for json in \
    ltools-capabilities.json \
    ltools-terminal.json \
    ltools-project.json; do
    [[ -f "$RELEASE_DIR/$json" && ! -L "$RELEASE_DIR/$json" && -s "$RELEASE_DIR/$json" ]] || die "falta $json regular"
    jq empty "$RELEASE_DIR/$json" >/dev/null || die "$json no es JSON válido"
done
project_json="$RELEASE_DIR/ltools-project.json"
if (( REQUIRE_APPIMAGE || REQUIRE_PACKAGE )); then
    jq -e '.platforms.linux.runtime.appimage_requires_fuse == true and
            .platforms.linux.runtime.appimage_extract_override == "APPIMAGE_EXTRACT_AND_RUN=1" and
            (.platforms.windows.runtime // null) == null' "$project_json" >/dev/null \
        || die 'ltools-project.json no declara correctamente el requisito Linux de FUSE y su ausencia en Windows'
    ok 'metadatos de runtime AppImage/FUSE coherentes por plataforma'
fi
for json in ltools-capabilities-windows.json ltools-terminal-windows.json; do
    if [[ -e "$RELEASE_DIR/$json" || -L "$RELEASE_DIR/$json" ]]; then
        [[ -f "$RELEASE_DIR/$json" && ! -L "$RELEASE_DIR/$json" ]] || die "$json no es un fichero regular"
        jq empty "$RELEASE_DIR/$json" >/dev/null || die "$json no es JSON válido"
    fi
done
if (( REQUIRE_WINDOWS_EXECUTABLES )); then
    [[ -f "$RELEASE_DIR/ltools-capabilities-windows.json" && ! -L "$RELEASE_DIR/ltools-capabilities-windows.json" && -s "$RELEASE_DIR/ltools-capabilities-windows.json" ]] || die 'falta el descriptor regular de capacidades Windows'
    [[ -f "$RELEASE_DIR/ltools-terminal-windows.json" && ! -L "$RELEASE_DIR/ltools-terminal-windows.json" && -s "$RELEASE_DIR/ltools-terminal-windows.json" ]] || die 'falta el descriptor regular de terminal Windows'
    jq -e '.platform == "windows" and .application == "WinSlim-Tools"' \
        "$RELEASE_DIR/ltools-capabilities-windows.json" >/dev/null \
        || die 'el descriptor de capacidades Windows no declara la plataforma correcta'
    jq -e '.platform == "windows" and .host.product == "WinSlim Terminal"' \
        "$RELEASE_DIR/ltools-terminal-windows.json" >/dev/null \
        || die 'el descriptor de terminal Windows no declara WinSlim Terminal'
    ok 'descriptores Windows separados presentes y coherentes'
fi
for schema in \
    ltools-capabilities.schema.json \
    ltools-terminal.schema.json \
    ltools-project.schema.json \
    ltools-release.schema.json; do
    [[ -f "$RELEASE_DIR/$schema" && ! -L "$RELEASE_DIR/$schema" && -s "$RELEASE_DIR/$schema" ]] || die "falta $schema regular"
    jq empty "$RELEASE_DIR/$schema" >/dev/null || die "$schema no es JSON válido"
done
ok 'JSON y esquemas publicables válidos'
ok 'licencia MIT del proyecto y titular presentes'

required_linux=()
if (( REQUIRE_APPIMAGE )); then
    required_linux+=(
        "ltools-$VERSION-linux-$LINUX_ARCH.AppImage"
        "ltools-$VERSION-linux-$LINUX_ARCH-cli.AppImage"
    )
fi
if (( REQUIRE_PACKAGE )); then
    required_linux+=("ltools-$VERSION-linux-$LINUX_ARCH.tar.gz")
fi
for artifact in "${required_linux[@]}"; do
    [[ -f "$RELEASE_DIR/$artifact" && ! -L "$RELEASE_DIR/$artifact" && -s "$RELEASE_DIR/$artifact" ]] || die "falta el artefacto Linux regular $artifact"
done
if (( ${#required_linux[@]} )); then
    ok 'artefactos Linux solicitados presentes'
else
    ok 'no se han solicitado artefactos binarios Linux adicionales'
fi

if (( REQUIRE_WINDOWS )); then
    required_windows=(
        "ltools-$VERSION-windows-$WINDOWS_ARCH.exe"
        "ltools-$VERSION-windows-$WINDOWS_ARCH-cli.exe"
        "ltools-$VERSION-windows-$WINDOWS_ARCH.zip"
    )
    for artifact in "${required_windows[@]}"; do
        [[ -f "$RELEASE_DIR/$artifact" && ! -L "$RELEASE_DIR/$artifact" && -s "$RELEASE_DIR/$artifact" ]] || die "falta el artefacto Windows regular $artifact"
    done
    [[ -f "$RELEASE_DIR/THIRD-PARTY-LICENSES-windows.zip" && ! -L "$RELEASE_DIR/THIRD-PARTY-LICENSES-windows.zip" && -s "$RELEASE_DIR/THIRD-PARTY-LICENSES-windows.zip" ]] || die 'falta el ZIP independiente de licencias Windows'
    ok 'artefactos Windows principal, CLI, ZIP portable y avisos de licencias presentes'
fi

if (( REQUIRE_WINDOWS_EXECUTABLES )); then
    for artifact in \
        "ltools-$VERSION-windows-$WINDOWS_ARCH.exe" \
        "ltools-$VERSION-windows-$WINDOWS_ARCH-cli.exe"; do
        [[ -f "$RELEASE_DIR/$artifact" && ! -L "$RELEASE_DIR/$artifact" && -s "$RELEASE_DIR/$artifact" ]] || die "falta el ejecutable Windows regular $artifact"
    done
    ok 'ejecutables Windows principal y CLI presentes'
fi

required_manifest_artifacts=("${required_linux[@]}")
if (( REQUIRE_WINDOWS )); then
    required_manifest_artifacts+=("${required_windows[@]}")
elif (( REQUIRE_WINDOWS_EXECUTABLES )); then
    required_manifest_artifacts+=(
        "ltools-$VERSION-windows-$WINDOWS_ARCH.exe"
        "ltools-$VERSION-windows-$WINDOWS_ARCH-cli.exe"
    )
fi
for artifact in "${required_manifest_artifacts[@]}"; do
    jq -e --arg filename "$artifact" 'any(.artifacts[]; .filename == $filename)' "$manifest" >/dev/null \
        || die "el manifiesto omite el artefacto requerido $artifact"
done

while IFS= read -r -d '' link; do
    name="$(basename -- "$link")"
    case "$name" in
        "ltools-$VERSION-linux-"*|"ltools-$VERSION-windows-"*) die "un artefacto de release no puede ser enlace simbólico: $name" ;;
    esac
done < <(find "$RELEASE_DIR" -maxdepth 1 -type l -print0)

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
if [[ -e "$checksums" || -L "$checksums" ]]; then
    [[ -f "$checksums" && ! -L "$checksums" && -s "$checksums" ]] || die 'SHA256SUMS.txt falta, está vacío o no es un fichero normal'
    declare -A listed=()
    checksum_count=0
    while IFS= read -r line || [[ -n "$line" ]]; do
        [[ "${#line}" -gt 66 && "${line:0:64}" =~ ^[[:xdigit:]]{64}$ && "${line:64:2}" == '  ' ]] ||
            die "línea inválida en SHA256SUMS.txt"
        expected="${line:0:64}"
        name="${line:66}"
        [[ -n "$name" && "$name" != . && "$name" != .. && "$name" != */* && "$name" != *'\'* ]] || die "nombre inseguro en SHA256SUMS.txt: $name"
        [[ -f "$RELEASE_DIR/$name" && ! -L "$RELEASE_DIR/$name" ]] || die "SHA256SUMS.txt referencia un fichero ausente o no regular: $name"
        [[ -z "${listed[$name]+yes}" ]] || die "SHA256SUMS.txt contiene el fichero duplicado: $name"
        listed[$name]=1
        actual="$(sha256sum -- "$RELEASE_DIR/$name" | awk '{print $1}')"
        [[ "${actual,,}" == "${expected,,}" ]] || die "SHA256SUMS.txt no coincide para $name"
        checksum_count=$((checksum_count + 1))
    done < "$checksums"
    expected_count="$(find "$RELEASE_DIR" -maxdepth 1 -type f ! -name 'SHA256SUMS.txt' ! -name 'SHA256SUMS.txt.sig' ! -name 'SHA256SUMS.txt.sshsig' ! -name '*.tmp' ! -name '*.bak' -printf '%f\n' | wc -l)"
    [[ "$checksum_count" == "$expected_count" ]] || die "SHA256SUMS.txt no cubre todos los ficheros publicables"
    ok 'SHA256SUMS.txt cubre todos los artefactos y sus hashes coinciden'
    if [[ -e "$signature" || -L "$signature" ]]; then
        [[ -f "$signature" && ! -L "$signature" && -s "$signature" ]] || die 'SHA256SUMS.txt.sig está vacío o no es un fichero normal'
        [[ ( -n "$SIGNATURE_PUBLIC_KEY_FILE" && -s "$SIGNATURE_PUBLIC_KEY_FILE" ) || -n "${LTOOLS_UPDATE_PUBLIC_KEY:-${LTERMINAL_UPDATE_PUBLIC_KEY:-}}" ]] || die 'hay firma, pero falta --signature-public-key-file o una clave pública Ed25519 de entorno para verificarla'
        [[ -x "$SIGNATURE_VERIFIER" ]] || die 'hay firma, pero falta --signature-verifier ejecutable'
        signature_args=(release-signature --manifest "$checksums" --signature "$signature")
        if [[ -n "$SIGNATURE_PUBLIC_KEY_FILE" ]]; then
            signature_args+=(--public-key-file "$SIGNATURE_PUBLIC_KEY_FILE")
        fi
        "$SIGNATURE_VERIFIER" "${signature_args[@]}" --verify >/dev/null \
            || die 'la firma Ed25519 de SHA256SUMS.txt no es válida'
        ok 'firma Ed25519 de SHA256SUMS.txt verificada'
    fi
    ssh_signature="$RELEASE_DIR/SHA256SUMS.txt.sshsig"
    if (( REQUIRE_SSH_SIGNATURE )); then
        [[ -f "$ssh_signature" && ! -L "$ssh_signature" && -s "$ssh_signature" ]] || die 'se requiere SHA256SUMS.txt.sshsig y está ausente'
    fi
    if [[ -e "$ssh_signature" || -L "$ssh_signature" ]]; then
        [[ -f "$ssh_signature" && ! -L "$ssh_signature" && -s "$ssh_signature" ]] || die 'SHA256SUMS.txt.sshsig está vacío o no es un fichero normal'
        source "$ROOT_DIR/scripts/lib/ssh-signing.sh"
        if [[ -n "$SSH_SIGNATURE_PUBLIC_KEY_FILE" ]]; then
            export LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="$SSH_SIGNATURE_PUBLIC_KEY_FILE"
        fi
        if [[ -n "$SSH_SIGNATURE_IDENTITY" ]]; then
            export LTOOLS_SSH_SIGNING_IDENTITY="$SSH_SIGNATURE_IDENTITY"
        fi
        ltools_ssh_verify_manifest "$checksums" "$ssh_signature" >/dev/null \
            || die 'la firma OpenSSH de SHA256SUMS.txt.sshsig no es válida'
        ok 'firma OpenSSH de SHA256SUMS.txt verificada'
    fi
fi

if (( REQUIRE_WINDOWS || REQUIRE_WINDOWS_EXECUTABLES )); then
    jq -e '([.artifacts[].platform] | index("linux")) != null and
           ([.artifacts[].platform] | index("windows")) != null' "$manifest" >/dev/null \
        || die 'el manifiesto final no contiene ambas plataformas'
    ok 'manifiesto unificado Linux/Windows'
fi

printf 'E2E de release completado correctamente: %s\n' "$RELEASE_DIR"
