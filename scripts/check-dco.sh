#!/usr/bin/env bash
# Verify commit messages include a Developer Certificate of Origin sign-off.
set -euo pipefail

range="${1:-HEAD~20..HEAD}"
missing=0
while IFS= read -r sha; do
  [[ -z "$sha" ]] && continue
  if ! git log -1 --format=%B "$sha" | grep -qiE 'signed-off-by:'; then
    echo "missing Signed-off-by: $sha $(git log -1 --format=%s "$sha")"
    missing=1
  fi
done < <(git rev-list "$range" 2>/dev/null || true)

if [[ "$missing" -ne 0 ]]; then
  echo "Add 'Signed-off-by: Name <email>' to commit bodies (see CONTRIBUTING.md)."
  exit 1
fi
echo "DCO sign-off OK for $range"
