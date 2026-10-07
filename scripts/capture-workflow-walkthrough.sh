#!/usr/bin/env bash
# End-to-end user-story walkthrough at 1280 light against the production shell.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/docs/screenshots/workflow-walkthrough"
BIN="$ROOT/target/release/unibrowser-desktop"
PREFS="$ROOT/scripts/fixtures/walkthrough-preferences.json"
PACK="$ROOT/examples/demo-pack"

cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
mkdir -p "$OUT"
rm -f "$OUT"/*.png

export GDK_SCALE=1
export GDK_DPI_SCALE=1
export UNIBROWSER_REPO_ROOT="$ROOT"
export UNIBROWSER_SAFE_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

tmp="$(mktemp -d -t unibrowser-walk.XXXXXX)"
export XDG_CONFIG_HOME="$tmp/config"
mkdir -p "$XDG_CONFIG_HOME/unibrowser"
cp "$PREFS" "$XDG_CONFIG_HOME/unibrowser/preferences.json"

xfwm4 --replace >"$tmp/xfwm.log" 2>&1 &
wm_pid=$!
app_pid=""

cleanup() {
  if [[ -n "$app_pid" ]]; then
    kill "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
  fi
  kill "$wm_pid" 2>/dev/null || true
  wait "$wm_pid" 2>/dev/null || true
  rm -rf "$tmp"
}
trap cleanup EXIT

main_window() {
  local candidate
  while read -r candidate; do
    if [[ "$(xdotool getwindowgeometry --shell "$candidate" | awk -F= '/^WIDTH=/{print $2}')" -ge 1000 ]]; then
      echo "$candidate"
      return 0
    fi
  done < <(xdotool search --sync --onlyvisible --name UniBrowser)
  return 1
}

close_floating_toolbar() {
  local floating
  while read -r floating; do
    [[ -n "$floating" ]] && xdotool windowclose "$floating"
  done < <(xdotool search --onlyvisible --name "Unibrowser commands" 2>/dev/null || true)
  xdotool sleep 0.2
}

start_app() {
  local demo="$1"
  shift
  UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-light,standard-contrast,scale-100,${demo}" \
    "$BIN" "$@" >"$tmp/app.log" 2>&1 &
  app_pid=$!
  window_id="$(main_window)"
  xdotool windowsize "$window_id" 1280 800
  xdotool windowmove "$window_id" 0 0
  xdotool sleep 1.2
  close_floating_toolbar
}

stop_app() {
  if [[ -n "$app_pid" ]]; then
    kill "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
  fi
  app_pid=""
  xdotool sleep 0.3
}

shot() {
  local name="$1"
  import -window root "$OUT/${name}.png"
  convert "$OUT/${name}.png" -crop 1280x800+0+0 +repage "$OUT/${name}.png"
  echo "wrote $OUT/${name}.png"
}

reset_prefs() {
  cp "$PREFS" "$XDG_CONFIG_HOME/unibrowser/preferences.json"
}

# --- 1. Open a file from New tab (US-01) ---
reset_prefs
start_app "ntp"
xdotool sleep 0.8
shot "01-new-tab"
# sample.md recent row sits under Opened recently.
xdotool mousemove 380 318 click 1
xdotool sleep 1.2
shot "01-opened-recent"
stop_app

# --- 2. Annotate a PDF and save (US-09, US-05) ---
reset_prefs
start_app "" --open "$PACK/sample.pdf"
xdotool sleep 1
shot "02-pdf-open"
# Highlight follows the PDF Annotation context tab.
xdotool mousemove 155 70 click 1
xdotool sleep 0.3
# Drag a highlight across the first body paragraph.
xdotool mousemove 400 250
xdotool mousedown 1
xdotool mousemove 620 270
xdotool mouseup 1
xdotool sleep 0.6
shot "02-pdf-annotated"
xdotool key ctrl+s
xdotool sleep 0.8
shot "02-pdf-saved"
stop_app

# --- 3. Edit markdown (US-03, US-06) ---
reset_prefs
cp "$PACK/sample.md" "$tmp/walk.md"
start_app "" --open "$tmp/walk.md"
xdotool sleep 1
shot "03-md-open"
xdotool mousemove 400 280 click 1
xdotool sleep 0.2
xdotool type --delay 20 " Walkthrough note."
xdotool sleep 0.8
shot "03-md-edited"
xdotool key ctrl+s
xdotool sleep 0.8
shot "03-md-saved"
stop_app

# --- 4. Workshop timeline and export (US-10) ---
reset_prefs
start_app "" --open "$PACK/sample.unibrow"
xdotool sleep 1.4
shot "04-workshop-open"
# Add title is the third Project CM control.
xdotool mousemove 250 70 click 1
xdotool sleep 0.8
shot "04-workshop-title"
# Export is overflowed at 1280. Open the chevron, then the last overflow row.
xdotool mousemove 500 70 click 1
xdotool sleep 0.4
xdotool mousemove 500 280 click 1
xdotool sleep 0.8
shot "04-workshop-export"
stop_app

# --- 5. Install a mod from the portal (US-13) ---
reset_prefs
start_app "ui17-mod-portal" --open "$PACK/sample.step"
xdotool sleep 1.2
shot "05-portal-open"
# Featured Install on the CAD file support card.
xdotool mousemove 1140 340 click 1
xdotool sleep 0.8
shot "05-portal-install"
stop_app

# --- 6. Change autosave in settings (US-05) ---
reset_prefs
start_app "settings" --open "$PACK/sample.txt"
xdotool sleep 1.4
shot "06-settings-open"
# Auto-save interval select, then Every 1 minute.
xdotool mousemove 1180 518 click 1
xdotool sleep 0.3
xdotool mousemove 1180 548 click 1
xdotool sleep 0.6
shot "06-settings-autosave"
stop_app

# --- 7. Switch themes ---
reset_prefs
start_app "settings" --open "$PACK/sample.txt"
xdotool sleep 1.4
shot "07-theme-light"
# Theme lives under Appearance, not General. Close any floating CM first.
close_floating_toolbar
xdotool mousemove 95 215 click 1
xdotool sleep 0.5
xdotool mousemove 1180 200 click 1
xdotool sleep 0.3
xdotool mousemove 1180 245 click 1
xdotool sleep 0.8
shot "07-theme-dark"
stop_app

echo "Walkthrough captures in $OUT"
ls -1 "$OUT"/*.png
