#!/usr/bin/env bash
# Preparación y promoción recuperable de la carpeta plana de release.

LTOOLS_RELEASE_DESTINATION=''
LTOOLS_RELEASE_STAGING=''
LTOOLS_RELEASE_ORIGINAL_ID=''
LTOOLS_RELEASE_ORIGINAL_EXISTS=0

ltools_create_release_staging() {
    local destination="$1" parent name
    [[ -n "$destination" ]] || { printf 'Ruta de release vacía.\n' >&2; return 1; }
    [[ -z "$LTOOLS_RELEASE_STAGING" ]] || { printf 'Ya existe un staging de release activo.\n' >&2; return 1; }
    parent="$(dirname -- "$destination")"
    name="$(basename -- "$destination")"
    [[ -n "$name" && "$name" != . && "$name" != .. ]] || {
        printf 'Nombre de release no válido: %s\n' "$destination" >&2
        return 1
    }
    mkdir -p -- "$parent" || { printf 'No se pudo crear el padre de release: %s\n' "$parent" >&2; return 1; }
    if [[ -L "$destination" || ( -e "$destination" && ! -d "$destination" ) ]]; then
        printf 'El destino de release debe ser una carpeta normal, no un archivo ni un enlace: %s\n' "$destination" >&2
        return 1
    fi

    LTOOLS_RELEASE_DESTINATION="$destination"
    if [[ -d "$destination" ]]; then
        LTOOLS_RELEASE_ORIGINAL_EXISTS=1
        LTOOLS_RELEASE_ORIGINAL_ID="$(stat -c '%d:%i' -- "$destination")" || return 1
    else
        LTOOLS_RELEASE_ORIGINAL_EXISTS=0
        LTOOLS_RELEASE_ORIGINAL_ID=''
    fi
    LTOOLS_RELEASE_STAGING="$(mktemp -d -- "$parent/.$name.ltools-release.XXXXXX")" || {
        LTOOLS_RELEASE_DESTINATION=''
        return 1
    }
    if (( LTOOLS_RELEASE_ORIGINAL_EXISTS )); then
        if find "$destination" \( ! -type d -a ! -type f \) -print -quit | grep -q .; then
            printf 'La release existente contiene enlaces u objetos especiales; no se copia ni se modifica: %s\n' "$destination" >&2
            ltools_cleanup_release_staging
            return 1
        fi
        if ! cp -a -- "$destination/." "$LTOOLS_RELEASE_STAGING/"; then
            ltools_cleanup_release_staging
            return 1
        fi
    fi
}

ltools_promote_release_staging() {
    local destination="$LTOOLS_RELEASE_DESTINATION" staging="$LTOOLS_RELEASE_STAGING"
    local parent name current_id backup status
    [[ -n "$destination" && -n "$staging" && -d "$staging" && ! -L "$staging" ]] || {
        printf 'No existe un staging de release válido para promover.\n' >&2
        return 1
    }
    parent="$(dirname -- "$destination")"
    name="$(basename -- "$destination")"

    if (( LTOOLS_RELEASE_ORIGINAL_EXISTS )); then
        [[ -d "$destination" && ! -L "$destination" ]] || {
            printf 'La carpeta release cambió de tipo desde que se preparó el staging: %s\n' "$destination" >&2
            return 1
        }
        current_id="$(stat -c '%d:%i' -- "$destination")" || return 1
        [[ "$current_id" == "$LTOOLS_RELEASE_ORIGINAL_ID" ]] || {
            printf 'La carpeta release fue sustituida concurrentemente; se conserva el destino actual.\n' >&2
            return 1
        }
        if mv --help 2>&1 | grep -Fq -- '--exchange'; then
            if mv --exchange -T --no-copy -- "$staging" "$destination"; then
                # Tras el intercambio, staging contiene la release previa.
                rm -rf -- "$staging" || printf 'AVISO: release previa conservada en %s\n' "$staging" >&2
                LTOOLS_RELEASE_STAGING=''
                LTOOLS_RELEASE_DESTINATION=''
                LTOOLS_RELEASE_ORIGINAL_ID=''
                LTOOLS_RELEASE_ORIGINAL_EXISTS=0
                return 0
            fi
        fi

        backup="$(mktemp -d -- "$parent/.$name.ltools-previous.XXXXXX")" || return 1
        rmdir -- "$backup" || return 1
        if ! mv -T -- "$destination" "$backup"; then
            printf 'No se pudo guardar la release previa; sigue intacta: %s\n' "$destination" >&2
            return 1
        fi
        if mv -Tn -- "$staging" "$destination" && [[ ! -e "$staging" ]]; then
            rm -rf -- "$backup" || printf 'AVISO: release previa conservada en %s\n' "$backup" >&2
            LTOOLS_RELEASE_STAGING=''
            LTOOLS_RELEASE_DESTINATION=''
            LTOOLS_RELEASE_ORIGINAL_ID=''
            LTOOLS_RELEASE_ORIGINAL_EXISTS=0
            return 0
        fi
        status=1
        if mv -T -- "$backup" "$destination"; then
            printf 'No se pudo instalar el staging; se restauró la release previa.\n' >&2
        else
            printf 'ERROR: no se pudo restaurar automáticamente la release previa; permanece en %s\n' "$backup" >&2
        fi
        return "${status:-1}"
    fi

    if [[ -e "$destination" || -L "$destination" ]]; then
        printf 'Apareció un destino release durante la preparación; no se sobrescribe: %s\n' "$destination" >&2
        return 1
    fi
    if ! mv -Tn -- "$staging" "$destination" || [[ -e "$staging" ]]; then
        printf 'No se pudo promover la carpeta release; el staging se conserva en %s\n' "$staging" >&2
        return 1
    fi
    LTOOLS_RELEASE_STAGING=''
    LTOOLS_RELEASE_DESTINATION=''
    LTOOLS_RELEASE_ORIGINAL_ID=''
    LTOOLS_RELEASE_ORIGINAL_EXISTS=0
}

ltools_cleanup_release_staging() {
    if [[ -n "$LTOOLS_RELEASE_STAGING" && -d "$LTOOLS_RELEASE_STAGING" && ! -L "$LTOOLS_RELEASE_STAGING" ]]; then
        rm -rf -- "$LTOOLS_RELEASE_STAGING" || true
    fi
    LTOOLS_RELEASE_STAGING=''
    LTOOLS_RELEASE_DESTINATION=''
    LTOOLS_RELEASE_ORIGINAL_ID=''
    LTOOLS_RELEASE_ORIGINAL_EXISTS=0
}
