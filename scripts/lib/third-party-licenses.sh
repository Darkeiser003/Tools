#!/usr/bin/env bash
# Copia los avisos/licencias de las dependencias Cargo al paquete distribuible.

ltools_create_third_party_license_bundle() {
    local destination="$1" target="${2:-}" root_manifest metadata
    local name version license manifest crate_root package_destination file relative target_file
    local copied=0

    [[ -n "$destination" ]] || { printf 'licencias: falta la carpeta destino.\n' >&2; return 2; }
    [[ ! -e "$destination" && ! -L "$destination" ]] || {
        printf 'licencias: el destino ya existe; no se mezclan paquetes ni restos: %s\n' "$destination" >&2
        return 1
    }
    command -v cargo >/dev/null 2>&1 || { printf 'licencias: cargo no está disponible.\n' >&2; return 1; }
    command -v jq >/dev/null 2>&1 || { printf 'licencias: jq es necesario para leer cargo metadata.\n' >&2; return 1; }
    command -v realpath >/dev/null 2>&1 || { printf 'licencias: realpath no está disponible.\n' >&2; return 1; }

    root_manifest="$(realpath -e -- "$ROOT_DIR/rust/Cargo.toml")" || return 1
    local -a metadata_args=(metadata --offline --format-version 1 --manifest-path "$root_manifest")
    [[ -z "$target" ]] || metadata_args+=(--filter-platform "$target")
    metadata="$(cargo "${metadata_args[@]}")" || {
        printf 'licencias: cargo metadata falló; no se publica un paquete sin avisos verificables.\n' >&2
        return 1
    }
    jq -e '.packages | type == "array" and length > 0' <<<"$metadata" >/dev/null || {
        printf 'licencias: cargo metadata no devolvió paquetes.\n' >&2
        return 1
    }

    mkdir -p -- "$destination" || return 1
    {
        printf 'LTools — avisos de dependencias de terceros\n'
        printf 'Plataforma Cargo: %s\n' "${target:-todas las plataformas del grafo bloqueado}"
        printf 'Se conservan los ficheros legales originales de cada paquete Cargo incluido.\n\n'
        printf 'Paquete\tVersión\tExpresión de licencia\n'
    } >"$destination/INDEX.txt" || return 1

    while IFS=$'\t' read -r name version license manifest; do
        [[ -n "$name" && -n "$version" && -n "$manifest" ]] || {
            printf 'licencias: entrada de paquete incompleta en cargo metadata.\n' >&2
            return 1
        }
        [[ -n "$license" && "$license" != null ]] || {
            printf 'licencias: el paquete %s %s no declara licencia.\n' "$name" "$version" >&2
            return 1
        }
        [[ "$name" =~ ^[A-Za-z0-9_.+-]+$ && "$version" =~ ^[A-Za-z0-9_.+-]+$ ]] || {
            printf 'licencias: nombre/version no seguros en cargo metadata: %s %s\n' "$name" "$version" >&2
            return 1
        }
        manifest="$(realpath -e -- "$manifest")" || return 1
        [[ "$manifest" != "$root_manifest" ]] || continue
        crate_root="${manifest%/*}"
        package_destination="$destination/$name-$version"
        local -a legal_files=()
        mapfile -d '' legal_files < <(find "$crate_root" -type f \
            \( -iname 'license*' -o -iname 'copying*' -o -iname 'notice*' -o -iname 'copyright*' -o -iname 'patents*' \) -print0)
        ((${#legal_files[@]} > 0)) || {
            printf 'licencias: %s %s no contiene ficheros LICENSE/COPYING/NOTICE verificables.\n' "$name" "$version" >&2
            return 1
        }
        printf '%s\t%s\t%s\n' "$name" "$version" "$license" >>"$destination/INDEX.txt" || return 1
        for file in "${legal_files[@]}"; do
            relative="${file#"$crate_root"/}"
            [[ "$relative" != "$file" && "$relative" != ../* && "$relative" != */../* ]] || {
                printf 'licencias: ruta de aviso fuera del paquete: %s\n' "$file" >&2
                return 1
            }
            target_file="$package_destination/$relative"
            mkdir -p -- "${target_file%/*}" || return 1
            cp -- "$file" "$target_file" || return 1
            copied=$((copied + 1))
        done
    done < <(jq -r --arg root "$root_manifest" '
        .packages[]
        | select(.manifest_path != $root)
        | [.name, .version, (.license // ""), .manifest_path]
        | @tsv' <<<"$metadata" | sort -t $'\t' -k1,1 -k2,2)

    ((copied > 0)) || {
        printf 'licencias: no se copiaron avisos de dependencias.\n' >&2
        return 1
    }
    printf 'Ficheros legales copiados: %s\n' "$copied" >>"$destination/INDEX.txt" || return 1
    printf 'Avisos de terceros preparados: %s paquetes, %s ficheros legales.\n' \
        "$(jq --arg root "$root_manifest" '[.packages[] | select(.manifest_path != $root)] | length' <<<"$metadata")" "$copied"
}
