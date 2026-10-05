# Bundled external binaries

Tauri expects `unibrowser-media-sidecar-<target-triple>` in this folder when bundling.

`build.rs` copies `target/<profile>/unibrowser-media-sidecar` here when present.
CI builds the sidecar before `cargo build -p unibrowser-desktop`.
