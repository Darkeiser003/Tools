#!/usr/bin/env bash
# Compatibility launcher. The Linux builder lives in platform/linux.
set -Eeuo pipefail
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
exec "$ROOT_DIR/platform/linux/build.sh" "$@"
