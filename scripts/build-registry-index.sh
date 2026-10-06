#!/usr/bin/env bash
# Build registry/index.json and SHA-256 checksums from registry/mods/*.toml
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
registry="$root/registry"
mods_dir="$registry/mods"
out="$registry/index.json"
checksums="$registry/checksums.sha256"

mkdir -p "$mods_dir"
python3 - "$mods_dir" "$out" "$checksums" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

mods_dir, out_path, sums_path = map(Path, sys.argv[1:4])
entries = []
for toml in sorted(mods_dir.glob("*.toml")):
    raw = toml.read_bytes()
    entries.append(
        {
            "id": toml.stem,
            "toml": f"mods/{toml.name}",
            "sha256": hashlib.sha256(raw).hexdigest(),
        }
    )
index = {"generated_at": __import__("datetime").datetime.utcnow().isoformat() + "Z", "mods": entries}
out_path.write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")
lines = [f"{e['sha256']}  mods/{Path(e['toml']).name}" for e in entries]
sums_path.write_text("\n".join(lines) + ("\n" if lines else ""), encoding="utf-8")
print(f"wrote {out_path} ({len(entries)} mods)")
PY
