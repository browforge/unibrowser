#!/usr/bin/env bash
# Real pointer and keyboard interaction-state sheets from the production shell.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/docs/screenshots/responsive-and-states"
BIN="$ROOT/target/release/unibrowser-desktop"
PREFS="$ROOT/scripts/fixtures/capture-preferences.json"

if [[ "${1:-}" != "--worker" ]]; then
  cd "$ROOT"
  (cd "$ROOT/crates/unibrowser-desktop" && npm run build)
  # shellcheck source=ensure-release-desktop.sh
  source "$ROOT/scripts/ensure-release-desktop.sh"
  mkdir -p "$OUT"
  rm -f "$OUT"/states-*.png
  for spec in "light:standard" "dark:standard" "dark:high"; do
    theme="${spec%%:*}"
    contrast="${spec#*:}"
    xvfb-run -a -s "-screen 0 1200x800x24 -dpi 96" \
      env STATE_THEME="$theme" STATE_CONTRAST="$contrast" \
      "$0" --worker
  done
  echo "Interaction state sheets in $OUT"
  exit 0
fi

export GDK_SCALE=1
export GDK_DPI_SCALE=1
export UNIBROWSER_REPO_ROOT="$ROOT"
export UNIBROWSER_SAFE_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

theme="$STATE_THEME"
contrast="$STATE_CONTRAST"
suffix="$theme"
contrast_token="standard-contrast"
if [[ "$contrast" == "high" ]]; then
  suffix="${theme}-hc"
  contrast_token="high-contrast"
fi

tmp="$(mktemp -d -t unibrowser-states.XXXXXX)"
export XDG_CONFIG_HOME="$tmp/config"
mkdir -p "$XDG_CONFIG_HOME/unibrowser" "$tmp/cells"
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

start_surface() {
  local demo="$1"
  local path="$2"
  UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-${theme},${contrast_token},scale-100,${demo}" \
    "$BIN" --open "$path" >"$tmp/app.log" 2>&1 &
  app_pid=$!
  window_id="$(main_window)"
  xdotool windowsize "$window_id" 1200 800
  xdotool windowmove "$window_id" 0 0
  xdotool sleep 1
}

stop_surface() {
  kill "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
  app_pid=""
  xdotool sleep 0.3
}

shot() {
  local name="$1"
  local x="$2"
  local y="$3"
  local width="$4"
  local height="$5"
  import -window root "$tmp/full.png"
  convert "$tmp/full.png" -crop "${width}x${height}+${x}+${y}" +repage \
    -bordercolor "$( [[ "$theme" == "dark" ]] && echo '#232526' || echo '#f1f0ec' )" \
    -border 1 "$tmp/cells/${name}.png"
}

capture_states() {
  local order="$1"
  local label="$2"
  local target_x="$3"
  local target_y="$4"
  local crop_x="$5"
  local crop_y="$6"
  local width="${7:-300}"
  local height="${8:-90}"

  xdotool mousemove 1180 780
  shot "${order}-default" "$crop_x" "$crop_y" "$width" "$height"

  xdotool mousemove "$target_x" "$target_y"
  xdotool sleep 0.2
  shot "${order}-hover" "$crop_x" "$crop_y" "$width" "$height"

  xdotool mousedown 1
  xdotool mousemove 1180 780
  xdotool mouseup 1
  xdotool key Tab
  xdotool key shift+Tab
  xdotool mousemove 1180 780
  shot "${order}-focus" "$crop_x" "$crop_y" "$width" "$height"

  xdotool key Escape
  xdotool mousemove "$target_x" "$target_y"
  xdotool mousedown 1
  shot "${order}-pressed" "$crop_x" "$crop_y" "$width" "$height"
  xdotool mousemove 1180 780
  xdotool mouseup 1

  for state in default hover focus pressed; do
    convert "$tmp/cells/${order}-${state}.png" \
      -gravity north -background "$( [[ "$theme" == "dark" ]] && echo '#17191a' || echo '#ffffff' )" \
      -fill "$( [[ "$theme" == "dark" ]] && echo '#f0efeb' || echo '#202223' )" \
      -splice 0x24 -pointsize 13 -annotate +0+4 "${label} / ${state}" \
      "$tmp/cells/${order}-${state}.png"
  done
}

start_surface "ui17-mod-portal" "$ROOT/examples/demo-pack/sample.step"

# Capture a genuine unsupported control with its reason tooltip before closing
# the separately-rendered floating toolbar.
xdotool mousemove 270 154
xdotool sleep 1
shot "00-disabled-reason" 150 105 360 105
convert "$tmp/cells/00-disabled-reason.png" \
  -gravity north -background "$( [[ "$theme" == "dark" ]] && echo '#17191a' || echo '#ffffff' )" \
  -fill "$( [[ "$theme" == "dark" ]] && echo '#f0efeb' || echo '#202223' )" \
  -splice 0x24 -pointsize 13 -annotate +0+4 "CM unsupported / reason tooltip" \
  "$tmp/cells/00-disabled-reason.png"
xdotool mousemove 70 134 click 1
xdotool sleep 0.3

capture_states "01-cm" "Command Manager" 115 58 0 28
capture_states "02-tab" "Document tab" 235 18 80 0
capture_states "03-rail" "Activity rail" 20 166 0 120
capture_states "04-mod-tab" "Portal tab" 158 206 35 170
capture_states "05-input" "Input" 235 260 80 220
capture_states "06-button" "Button" 492 260 360 220
capture_states "07-install" "Install" 1062 366 900 326
stop_surface

cp "$PREFS" "$XDG_CONFIG_HOME/unibrowser/preferences.json"
start_surface "settings" "$ROOT/examples/demo-pack/sample.txt"
xdotool mousemove 70 134 click 1
xdotool sleep 0.3
capture_states "08-switch" "Switch" 980 469 850 429
capture_states "09-select" "Select" 1080 469 900 429
capture_states "10-settings-input" "Settings input" 130 140 0 100
capture_states "11-settings-button" "Settings button" 1090 569 900 529

# Disable the interval select through the real Auto-save switch.
xdotool mousemove 980 469 click 1
xdotool mousemove 1180 780
shot "12-disabled-form" 850 429 350 90
convert "$tmp/cells/12-disabled-form.png" \
  -gravity north -background "$( [[ "$theme" == "dark" ]] && echo '#17191a' || echo '#ffffff' )" \
  -fill "$( [[ "$theme" == "dark" ]] && echo '#f0efeb' || echo '#202223' )" \
  -splice 0x24 -pointsize 13 -annotate +0+4 "Switch off / disabled interval select" \
  "$tmp/cells/12-disabled-form.png"

mapfile -t state_cells < <(
  for row in {01..11}; do
    for state in default hover focus pressed; do
      printf '%s\n' "$tmp/cells/${row}-"*"-${state}.png"
    done
  done
)
montage "${state_cells[@]}" \
  -tile 4x11 -geometry +8+8 \
  -background "$( [[ "$theme" == "dark" ]] && echo '#0f1112' || echo '#e7e5df' )" \
  "$OUT/states-${suffix}.png"

montage "$tmp/cells/00-disabled-reason.png" "$tmp/cells/12-disabled-form.png" \
  -tile 2x1 -geometry +8+8 \
  -background "$( [[ "$theme" == "dark" ]] && echo '#0f1112' || echo '#e7e5df' )" \
  "$OUT/states-${suffix}-disabled.png"
