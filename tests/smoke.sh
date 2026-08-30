#!/usr/bin/env bash
# Compatibility launcher for the Linux smoke suite.
set -Eeuo pipefail
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
exec "$ROOT_DIR/tests/linux/smoke.sh" "$@"
