#!/usr/bin/env bash
# Compatibility launcher for the Linux menu E2E suite.
set -Eeuo pipefail
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
exec "$ROOT_DIR/tests/linux/menu-e2e.sh" "$@"
