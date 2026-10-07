#!/usr/bin/env bash
# Fresh per-run user config for doc captures (no persisted prefs from the host).
# Sets XDG_CONFIG_HOME so config_dir() resolves to $XDG_CONFIG_HOME/unibrowser.
set -euo pipefail

capture_config_prepare() {
  if [[ -n "${CAPTURE_CONFIG_TMP:-}" ]]; then
    capture_config_cleanup
  fi
  CAPTURE_CONFIG_TMP=$(mktemp -d -t unibrowser-capture-xdg.XXXXXX)
  export XDG_CONFIG_HOME="$CAPTURE_CONFIG_TMP"
  mkdir -p "$XDG_CONFIG_HOME/unibrowser"
}

capture_config_cleanup() {
  if [[ -n "${CAPTURE_CONFIG_TMP:-}" ]]; then
    rm -rf "$CAPTURE_CONFIG_TMP"
  fi
  unset CAPTURE_CONFIG_TMP
  unset XDG_CONFIG_HOME
}

# Run a capture command with a fresh XDG_CONFIG_HOME (cleans up on success or failure).
with_capture_isolated_config() {
  capture_config_prepare
  set +e
  "$@"
  local status=$?
  set -e
  capture_config_cleanup
  return "$status"
}
