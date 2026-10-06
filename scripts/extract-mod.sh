#!/usr/bin/env bash
# Split an in-tree mod directory into its own git repo (git filter-repo).
# Default: dry-run. Pass --execute to rewrite history.
set -euo pipefail

usage() {
  echo "Usage: $0 <mod-id> [--execute] [--target DIR]"
  echo "  mod-id matches mods/<mod-id> or registry path=mods/<mod-id>"
  exit 1
}

mod_id=""
execute=0
target=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --execute) execute=1; shift ;;
    --target) target="$2"; shift 2 ;;
    -h|--help) usage ;;
    *)
      if [[ -z "$mod_id" ]]; then mod_id="$1"; else usage; fi
      shift ;;
  esac
done

[[ -n "$mod_id" ]] || usage

root="$(cd "$(dirname "$0")/.." && pwd)"
src="$root/mods/$mod_id"
[[ -d "$src" ]] || { echo "missing $src"; exit 1; }

target="${target:-$root/../ubmod-$mod_id}"

echo "Mod: $mod_id"
echo "Source tree: $src"
echo "Target repo: $target"
echo "Mode: $([[ $execute -eq 1 ]] && echo EXECUTE || echo dry-run)"

if [[ $execute -eq 0 ]]; then
  echo "Would run: git filter-repo --path mods/$mod_id --path-rename mods/$mod_id/:"
  echo "Pass --execute after backing up the repository."
  exit 0
fi

if ! command -v git-filter-repo >/dev/null 2>&1; then
  echo "Install git-filter-repo (pip install git-filter-repo) before --execute."
  exit 1
fi

tmp="$(mktemp -d)"
git clone "$root" "$tmp/out"
cd "$tmp/out"
git filter-repo --force --path "mods/$mod_id" --path-rename "mods/$mod_id/:" 
mv "$tmp/out" "$target"
echo "Extracted to $target"
