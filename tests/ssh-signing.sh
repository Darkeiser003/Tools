#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
source "$ROOT_DIR/scripts/lib/ssh-signing.sh"
command -v ssh-keygen >/dev/null 2>&1 || { printf 'SSH SIGN TEST ERROR: falta ssh-keygen.\n' >&2; exit 1; }
TEST_TMP="$(mktemp -d "/tmp/ltools-ssh-signing-$$-XXXXXX")"
trap 'rm -rf -- "$TEST_TMP"' EXIT
umask 077

ssh-keygen -q -t ed25519 -N '' -C 'ltools-test@example.invalid' -f "$TEST_TMP/test-key"
printf '89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567  package.zip\n' >"$TEST_TMP/SHA256SUMS.txt"
export LTOOLS_SSH_SIGNING_KEY_FILE="$TEST_TMP/test-key"
export LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE="$TEST_TMP/test-key.pub"
export LTOOLS_SSH_SIGNING_IDENTITY='ltools-test@example.invalid'
ltools_ssh_sign_manifest "$TEST_TMP/SHA256SUMS.txt" "$TEST_TMP/SHA256SUMS.txt.sshsig" >/dev/null
ltools_ssh_verify_manifest "$TEST_TMP/SHA256SUMS.txt" "$TEST_TMP/SHA256SUMS.txt.sshsig" >/dev/null

printf 'tampered\n' >>"$TEST_TMP/SHA256SUMS.txt"
if ltools_ssh_verify_manifest "$TEST_TMP/SHA256SUMS.txt" "$TEST_TMP/SHA256SUMS.txt.sshsig" >/dev/null 2>&1; then
    printf 'SSH SIGN TEST ERROR: se aceptó un manifiesto alterado.\n' >&2
    exit 1
fi
printf 'SSH signing/verification and tamper rejection passed.\n'
