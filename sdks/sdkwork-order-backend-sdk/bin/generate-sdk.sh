#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LANGUAGES="${LANGUAGES:-typescript}"
exec powershell -NoProfile -ExecutionPolicy Bypass -File "$SCRIPT_DIR/generate-sdk.ps1" -Languages "$LANGUAGES"
