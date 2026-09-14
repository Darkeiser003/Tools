#!/usr/bin/env bash
# Verifica que el bundle se crea desde el grafo Cargo bloqueado y no mezcla destinos.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "$0")/.." && pwd -P)"
source "$ROOT_DIR/scripts/lib/third-party-licenses.sh"
TEST_TMP="$(mktemp -d "/tmp/ltools-third-party-licenses-$$-XXXXXX")"
trap 'rm -rf -- "$TEST_TMP"' EXIT

host_target="$(rustc -vV | sed -n 's/^host: //p')"
[[ -n "$host_target" ]] || { printf 'LICENSE TEST ERROR: no se detectó el host Rust.\n' >&2; exit 1; }
ltools_create_third_party_license_bundle "$TEST_TMP/linux" "$host_target"
index="$TEST_TMP/linux/INDEX.txt"
[[ -s "$index" ]] || { printf 'LICENSE TEST ERROR: falta el índice generado.\n' >&2; exit 1; }
grep -Fq $'ring\t0.17.14\tApache-2.0 AND ISC' "$index" || {
    printf 'LICENSE TEST ERROR: el índice omite la licencia dual de ring.\n' >&2; exit 1;
}
grep -Fq $'webpki-roots\t1.0.9\tCDLA-Permissive-2.0' "$index" || {
    printf 'LICENSE TEST ERROR: el índice omite los datos de raíces TLS CDLA.\n' >&2; exit 1;
}
for legal_file in \
    ring-0.17.14/LICENSE \
    rustls-webpki-0.103.15/LICENSE \
    untrusted-0.9.0/LICENSE.txt \
    webpki-roots-1.0.9/LICENSE; do
    [[ -s "$TEST_TMP/linux/$legal_file" ]] || {
        printf 'LICENSE TEST ERROR: falta el texto legal original %s.\n' "$legal_file" >&2; exit 1;
    }
done

mkdir "$TEST_TMP/existing"
printf 'preserve me\n' >"$TEST_TMP/existing/sentinel"
if ltools_create_third_party_license_bundle "$TEST_TMP/existing" "$host_target" >/dev/null 2>&1; then
    printf 'LICENSE TEST ERROR: se permitió mezclar un destino preexistente.\n' >&2
    exit 1
fi
grep -Fxq 'preserve me' "$TEST_TMP/existing/sentinel" || {
    printf 'LICENSE TEST ERROR: se alteró el contenido previo del destino.\n' >&2; exit 1;
}

printf 'OK: bundle Linux genera %s avisos y protege destinos existentes.\n' \
    "$(find "$TEST_TMP/linux" -type f | wc -l | tr -d ' ')"
