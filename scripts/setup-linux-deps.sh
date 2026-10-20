#!/usr/bin/env bash
# Linux packages for Tauri 2 desktop builds, headless captures (xvfb), and workspace tests.
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Re-run with sudo: sudo $0" >&2
  exit 1
fi

export DEBIAN_FRONTEND=noninteractive

apt-get update
export UCF_FORCE_CONFFOLD=1
dpkg --configure -a || true
apt-get -f install -y -o Dpkg::Options::="--force-confold" || true

apt-get install -y \
  build-essential \
  pkg-config \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-0 \
  libssl-dev \
  xvfb \
  imagemagick \
  python3-pil \
  libfuse2 \
  patchelf

# fuse metapackage is not on all Ubuntu releases (exit 100); AppImage tooling only.
if apt-cache show fuse3 >/dev/null 2>&1; then
  apt-get install -y fuse3 2>/dev/null || echo "fuse3 optional (AppImage); skipped"
elif apt-cache show fuse >/dev/null 2>&1; then
  apt-get install -y fuse 2>/dev/null || echo "fuse optional (AppImage); skipped"
else
  echo "No fuse/fuse3 package; AppImage mount may be unavailable on this image"
fi
dpkg --configure -a 2>/dev/null || true

if command -v rustup >/dev/null 2>&1; then
  if [[ -n "${SUDO_USER:-}" ]]; then
    su - "$SUDO_USER" -c 'rustup target add wasm32-unknown-unknown'
  else
    rustup target add wasm32-unknown-unknown
  fi
else
  echo "rustup not found; install Rust (see rust-toolchain.toml) then run: rustup target add wasm32-unknown-unknown"
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
build_wasm_as() {
  if [[ -n "${SUDO_USER:-}" ]]; then
    su - "$SUDO_USER" -c "cd '$REPO_ROOT' && $*"
  else
    (cd "$REPO_ROOT" && eval "$*")
  fi
}

if command -v cargo >/dev/null 2>&1; then
  echo "Building example WASM mod (unibrowser-mod-host tests)..."
  build_wasm_as "cargo build -p ub-mod-wasm --target wasm32-unknown-unknown" || true
  echo "Building unibrowser-media sidecar binary (media crate tests)..."
  build_wasm_as "cargo build -p unibrowser-media --bin unibrowser-media-sidecar" || true
  build_wasm_as "cargo build -p unibrowser-media --bin unibrowser-media-sidecar --release" || true
else
  echo "cargo not found; wasm/media sidecar builds skipped"
fi

echo "Linux desktop dependencies installed."
