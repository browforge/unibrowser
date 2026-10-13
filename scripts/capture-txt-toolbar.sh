#!/usr/bin/env bash
# Render 1 text toolbar evidence (replaces legacy boxed chrome captures).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec "$ROOT/scripts/capture-bear-render1.sh"
