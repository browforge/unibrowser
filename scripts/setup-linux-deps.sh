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
  fuse \
  libfuse2 \
  patchelf

if command -v rustup >/dev/null 2>&1; then
  if [[ -n "${SUDO_USER:-}" ]]; then
    su - "$SUDO_USER" -c 'rustup target add wasm32-unknown-unknown'
  else
    rustup target add wasm32-unknown-unknown
  fi
else
  echo "rustup not found; install Rust (see rust-toolchain.toml) then run: rustup target add wasm32-unknown-unknown"
fi

echo "Linux desktop dependencies installed."
