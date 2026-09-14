#!/usr/bin/env bash

# Configuración y firma SSHSIG de SHA256SUMS para publicación verificable.
# La firma Ed25519 de LTools se conserva por compatibilidad con el actual
# verificador de actualizaciones; esta firma adicional usa OpenSSH/GitHub.

LTOOLS_SSH_SIGNING_KEY_FILE="${LTOOLS_SSH_SIGNING_KEY_FILE:-}"
LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="${LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE:-}"
LTOOLS_SSH_SIGNING_IDENTITY="${LTOOLS_SSH_SIGNING_IDENTITY:-}"
LTOOLS_SSH_KEYGEN=''

ltools_ssh_verification_config() {
    local configured_key home_dir
    if [[ -n "${LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE:-}" ]]; then
        :
    elif [[ -n "${LTERMINAL_SSH_SIGNING_PUBLIC_KEY_FILE:-}" ]]; then
        LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="$LTERMINAL_SSH_SIGNING_PUBLIC_KEY_FILE"
    else
        configured_key="${LTOOLS_SSH_SIGNING_KEY_FILE:-${LTERMINAL_SSH_SIGNING_KEY_FILE:-$(git config --get user.signingkey 2>/dev/null || true)}}"
        [[ "$configured_key" != key::* ]] || configured_key=''
        if [[ "$configured_key" == *.pub ]]; then
            LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="$configured_key"
        elif [[ -n "$configured_key" ]]; then
            LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="${configured_key}.pub"
        else
            home_dir="${HOME:-}"
            [[ -n "$home_dir" ]] || return 1
            LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="$home_dir/.ssh/id_ed25519.pub"
        fi
    fi
    [[ "$LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE" == '~/'* ]] &&
        LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="${HOME:?HOME no está definido}/${LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE#~/}"
    LTOOLS_SSH_SIGNING_IDENTITY="${LTOOLS_SSH_SIGNING_IDENTITY:-${LTERMINAL_SSH_SIGNING_IDENTITY:-$(git config --get user.email 2>/dev/null || true)}}"
    LTOOLS_SSH_KEYGEN="$(command -v ssh-keygen 2>/dev/null || true)"
    [[ -n "$LTOOLS_SSH_KEYGEN" && -s "$LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE" &&
        "$LTOOLS_SSH_SIGNING_IDENTITY" =~ ^[A-Za-z0-9._@+-]+$ ]]
}

ltools_ssh_signing_config() {
    local configured_key home_dir
    ltools_ssh_verification_config || return 1
    if [[ -n "${LTOOLS_SSH_SIGNING_KEY_FILE:-}" ]]; then
        configured_key="$LTOOLS_SSH_SIGNING_KEY_FILE"
    elif [[ -n "${LTERMINAL_SSH_SIGNING_KEY_FILE:-}" ]]; then
        configured_key="$LTERMINAL_SSH_SIGNING_KEY_FILE"
    else
        configured_key="$(git config --get user.signingkey 2>/dev/null || true)"
        [[ "$configured_key" != key::* ]] || configured_key=''
        if [[ -z "$configured_key" ]]; then
            home_dir="${HOME:-}"
            [[ -n "$home_dir" ]] || return 1
            configured_key="$home_dir/.ssh/id_ed25519"
        fi
    fi
    [[ "$configured_key" == '~/'* ]] && configured_key="${HOME:?HOME no está definido}/${configured_key#~/}"
    if [[ "$configured_key" == *.pub ]]; then
        configured_key="${configured_key%.pub}"
    fi
    LTOOLS_SSH_SIGNING_KEY_FILE="$configured_key"
    [[ -r "$LTOOLS_SSH_SIGNING_KEY_FILE" && -s "$LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE" ]]
}

ltools_ssh_allowed_signers_file() {
    local destination="$1" public_key_type public_key_data extra
    [[ -r "$LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE" ]] || return 1
    IFS=' ' read -r public_key_type public_key_data extra < "$LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE" || return 1
    [[ "$public_key_type" =~ ^(ssh-|sk-ssh-|ecdsa-) && -n "$public_key_data" ]] || return 1
    printf '%s %s %s\n' "$LTOOLS_SSH_SIGNING_IDENTITY" "$public_key_type" "$public_key_data" > "$destination"
}

ltools_ssh_verify_manifest() {
    local manifest="$1" signature="$2" temporary verification_key
    ltools_ssh_verification_config || {
        printf 'No se puede validar la firma OpenSSH: falta ssh-keygen, la clave pública o la identidad GitHub.\n' >&2
        return 1
    }
    [[ -s "$manifest" && -s "$signature" ]] || {
        printf 'Falta el manifiesto o la firma SSH para verificar.\n' >&2
        return 1
    }
    temporary="$(mktemp -d "$(realpath -m -- "${TMPDIR:-/tmp}")/ltools-ssh-verify-$$-XXXXXX")" || return 1
    verification_key="$temporary/allowed_signers"
    if ! ltools_ssh_allowed_signers_file "$verification_key"; then
        rm -rf -- "$temporary"
        printf 'La clave pública SSH no tiene un formato OpenSSH admitido.\n' >&2
        return 1
    fi
    if ! "$LTOOLS_SSH_KEYGEN" -Y verify -f "$verification_key" \
        -I "$LTOOLS_SSH_SIGNING_IDENTITY" -n ltools-release -s "$signature" \
        < "$manifest"; then
        rm -rf -- "$temporary"
        return 1
    fi
    rm -rf -- "$temporary"
}

ltools_ssh_sign_manifest() {
    local manifest="$1" signature="$2" temporary staged_manifest staged_signature verification_key
    ltools_ssh_signing_config || {
        printf 'Falta configurar una clave SSH de firma, su .pub, ssh-keygen o la identidad GitHub.\n' >&2
        return 1
    }
    [[ -s "$manifest" ]] || {
        printf 'No se puede firmar un SHA256SUMS.txt ausente o vacío.\n' >&2
        return 1
    }
    temporary="$(mktemp -d "$(realpath -m -- "${TMPDIR:-/tmp}")/ltools-ssh-sign-$$-XXXXXX")" || return 1
    staged_manifest="$temporary/SHA256SUMS.txt"
    staged_signature="$staged_manifest.sig"
    verification_key="$temporary/allowed_signers"
    if ! cp -- "$manifest" "$staged_manifest" || ! ltools_ssh_allowed_signers_file "$verification_key"; then
        rm -rf -- "$temporary"
        printf 'No se pudo preparar la firma OpenSSH de la release.\n' >&2
        return 1
    fi
    if ! "$LTOOLS_SSH_KEYGEN" -Y sign -f "$LTOOLS_SSH_SIGNING_KEY_FILE" \
        -n ltools-release "$staged_manifest"; then
        rm -rf -- "$temporary"
        printf 'ssh-keygen no pudo firmar la release.\n' >&2
        return 1
    fi
    if ! "$LTOOLS_SSH_KEYGEN" -Y verify -f "$verification_key" \
        -I "$LTOOLS_SSH_SIGNING_IDENTITY" -n ltools-release -s "$staged_signature" \
        < "$staged_manifest"; then
        rm -rf -- "$temporary"
        printf 'La firma OpenSSH recién generada no supera la verificación.\n' >&2
        return 1
    fi
    mkdir -p -- "$(dirname -- "$signature")"
    cp -- "$staged_signature" "${signature}.tmp-$$" && mv -f -- "${signature}.tmp-$$" "$signature"
    local result=$?
    rm -rf -- "$temporary"
    (( result == 0 )) || {
        printf 'No se pudo publicar la firma OpenSSH en %s.\n' "$signature" >&2
        return 1
    }
    printf 'Manifiesto firmado y verificado con OpenSSH: %s\n' "$signature"
}
