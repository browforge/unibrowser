//! PKG-01..08 audit probes (ADR 0032).

use std::fs::{self, File};
use std::path::Path;
use std::process::Command;

use zip::read::ZipArchive;
use zip::CompressionMethod;

use unibrowser_project::{
    add_asset_copy, add_media_ref, archive_zip_is_readable, collect_all_media_refs,
    content_hash_hex, create_package, export_package_to_zip, gc_content_addressed_assets,
    import_share_package_zip, is_legacy_unibrow_zip, is_opc_unibrow_zip, is_unibrow_path,
    is_unpacked_native_package, list_media_refs, list_referenced_package_parts,
    list_zip_archive_entries, materialize_zip_package, open_package, opc_part_bytes_equal,
    pending_folder_save_id, recover_incomplete_folder_save, resolve_package_root,
    save_archive_package, save_folder_package, set_save_fault_hook, write_legacy_plain_zip,
    write_manifest, SaveFaultPoint, DIR_ASSETS, HISTORY_FILE, MANIFEST_FILE, PROJECT_FILE,
};

fn zip_compression_for(path: &Path, entry_name: &str) -> CompressionMethod {
    let file = File::open(path).expect("open zip");
    let mut archive = ZipArchive::new(file).expect("zip archive");
    for i in 0..archive.len() {
        let entry = archive.by_index(i).expect("entry");
        let name = entry.name().replace('\\', "/");
        if name == entry_name || name.ends_with(&format!("/{}", entry_name)) {
            return entry.compression();
        }
    }
    panic!("zip entry not found: {}", entry_name);
}

fn temp_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ub-pkg-{}-{}", label, std::process::id()))
}

// --- PKG-01: OPC archive layout and compression policy ---

#[test]
fn pkg01_exported_zip_has_opc_content_types_and_rels() {
    let dir = temp_root("opc");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Opc").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    assert!(is_opc_unibrow_zip(&zip_path));
    let listed = list_zip_archive_entries(&zip_path, 200).expect("list");
    assert!(listed.iter().any(|e| e.path == "[Content_Types].xml"));
    assert!(listed.iter().any(|e| e.path == "_rels/.rels"));
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_export_has_manifest_and_project_json() {
    let dir = temp_root("manifest");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "M").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let listed = list_zip_archive_entries(&zip_path, 200).expect("list");
    assert!(listed.iter().any(|e| e.path == "project.json"));
    assert!(listed.iter().any(|e| e.path == "manifest.json"));
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_png_in_zip_uses_store_not_deflate() {
    let dir = temp_root("compress");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Media").expect("create");
    let png = dir.join("frame.png");
    fs::write(&png, [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0]).expect("png");
    add_asset_copy(&mut pkg, &png, "image", "frame").expect("asset");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let hash = pkg.manifest.nodes[0].asset.clone().unwrap();
    let method = zip_compression_for(&zip_path, &format!("assets/{}", hash));
    assert_eq!(method, CompressionMethod::Stored);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_identical_asset_bytes_are_content_hash_deduped() {
    let dir = temp_root("dedup");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Dedup").expect("create");
    let bytes = b"same-bytes";
    let a = dir.join("a.bin");
    let b = dir.join("b.bin");
    fs::write(&a, bytes).expect("a");
    fs::write(&b, bytes).expect("b");
    add_asset_copy(&mut pkg, &a, "file", "A").expect("a");
    add_asset_copy(&mut pkg, &b, "file", "B").expect("b");
    let hash = content_hash_hex(bytes);
    assert_eq!(pkg.manifest.nodes[0].asset.as_deref(), Some(hash.as_str()));
    assert_eq!(pkg.manifest.nodes[1].asset.as_deref(), Some(hash.as_str()));
    let assets: Vec<_> = fs::read_dir(dir.join(DIR_ASSETS))
        .expect("assets dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(assets.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg01_zip_has_unibrow_history_part() {
    let dir = temp_root("hist-part");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "H").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let listed = list_zip_archive_entries(&zip_path, 200).expect("list");
    assert!(listed.iter().any(|e| e.path == HISTORY_FILE));
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_zip_opens_in_python_zipfile() {
    let dir = temp_root("pyzip");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Py").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let out = Command::new("python3")
        .args([
            "-c",
            "import zipfile,sys; z=zipfile.ZipFile(sys.argv[1]); print('[Content_Types].xml' in z.namelist())",
        ])
        .arg(&zip_path)
        .output()
        .expect("python");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "True");
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_zip_roundtrip_preserves_project_json() {
    let dir = temp_root("rt");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Round").expect("create");
    pkg.manifest.title = "Round trip".into();
    write_manifest(&dir, &pkg.manifest).expect("write");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let cache = dir.join("cache");
    let materialized = materialize_zip_package(&zip_path, &cache).expect("materialize");
    let again = open_package(&materialized).expect("open");
    assert_eq!(again.manifest.title, "Round trip");
    let before = fs::read_to_string(dir.join(PROJECT_FILE)).expect("before");
    let after = fs::read_to_string(materialized.join(PROJECT_FILE)).expect("after");
    assert_eq!(before, after);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_archive_folder_archive_parts_lossless() {
    let dir = temp_root("lossless");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Lossless").expect("create");
    let png = dir.join("x.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").expect("png");
    add_asset_copy(&mut pkg, &png, "image", "x").expect("asset");
    let zip_a = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_a).expect("export a");
    let folder = dir.join("unpacked");
    let materialized = materialize_zip_package(&zip_a, &folder).expect("materialize");
    let zip_b = dir.join("repacked.unibrow");
    export_package_to_zip(&materialized, &zip_b).expect("export b");
    assert!(opc_part_bytes_equal(&zip_a, &zip_b).expect("map"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg01_legacy_plain_zip_opens_and_upgrades_on_save() {
    let dir = temp_root("legacy");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Legacy").expect("create");
    let legacy_zip = dir.with_extension("legacy.unibrow");
    write_legacy_plain_zip(&dir, &legacy_zip).expect("legacy zip");
    assert!(is_legacy_unibrow_zip(&legacy_zip));
    assert!(!is_opc_unibrow_zip(&legacy_zip));
    let cache = dir.join("cache");
    let root = materialize_zip_package(&legacy_zip, &cache).expect("open legacy");
    let pkg = open_package(&root).expect("package");
    write_manifest(&root, &pkg.manifest).expect("save upgrade");
    let upgraded = dir.join("upgraded.unibrow");
    export_package_to_zip(&root, &upgraded).expect("export opc");
    assert!(is_opc_unibrow_zip(&upgraded));
    let _ = fs::remove_dir_all(&dir);
}

// --- PKG-02: Folder mode on-disk IO ---

#[test]
fn pkg02_unpacked_bundle_has_manifest_and_project_json() {
    let dir = temp_root("folder-unpacked");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Folder").expect("create");
    assert!(dir.join(PROJECT_FILE).is_file());
    assert!(dir.join(MANIFEST_FILE).is_file());
    assert!(dir.join(HISTORY_FILE).is_file());
    assert!(is_unpacked_native_package(&dir));
    assert!(dir.join(DIR_ASSETS).is_dir());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg02_plain_directory_without_project_json_is_not_a_package() {
    let dir = temp_root("plain");
    let _ = fs::remove_dir_all(&dir);
    let show = dir.join("MyShow");
    fs::create_dir_all(&show).expect("mkdir");
    fs::write(show.join("clip.mp4"), b"fake").expect("media");
    assert!(open_package(&show).is_err());
    assert!(!show.join(HISTORY_FILE).exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg02_create_package_writes_unpacked_layout_not_plain_scaffold_only() {
    let dir = temp_root("scaffold");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Scaffold").expect("create");
    assert!(dir.join(PROJECT_FILE).exists());
    assert!(dir.join(MANIFEST_FILE).exists());
    assert!(dir.join(HISTORY_FILE).exists());
    assert!(dir.join("refs").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg02_share_import_writes_unpacked_layout() {
    let dir = temp_root("import");
    let _ = fs::remove_dir_all(&dir);
    let src = dir.join("src");
    let _pkg = create_package(&src, "ImportMe").expect("create");
    let zip_path = dir.join("share.zip");
    unibrowser_project::export_share_package_to_zip(&src, &zip_path).expect("export");
    let parent = dir.join("parent");
    fs::create_dir_all(&parent).expect("parent");
    let unpacked = import_share_package_zip(&zip_path, &parent, "Imported").expect("import");
    assert!(unpacked.join(PROJECT_FILE).is_file());
    assert!(unpacked.join(MANIFEST_FILE).is_file());
    assert!(unpacked.join(HISTORY_FILE).is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg02_native_package_uses_root_history_not_per_file_sidecar() {
    let dir = temp_root("pkg-hist");
    let _ = fs::remove_dir_all(&dir);
    let pkg_dir = dir.join("Pkg");
    let _pkg = create_package(&pkg_dir, "Pkg").expect("create");
    let note = pkg_dir.join("notes.md");
    fs::write(&note, "# note\n").expect("write");
    let parsed = unibrowser_document_model::parse_markdown(b"# note\n");
    let op = unibrowser_document_model::DocumentHistoryOp::Snapshot {
        model: parsed.model,
        source_map_blob: String::new(),
    };
    unibrowser_document_model::append_document_op(&note, &op, "0.1.0").expect("append");
    assert!(pkg_dir.join(HISTORY_FILE).is_file());
    assert!(!unibrowser_document_model::sidecar_path_for_file(&note).exists());
    let _ = fs::remove_dir_all(&dir);
}

// --- PKG-03: Storage mode state ---

#[test]
fn pkg03_opening_zip_materialises_cache_on_disk() {
    let dir = temp_root("materialize");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Cache").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let cache = dir.join("zip-cache");
    let _ = fs::remove_dir_all(&cache);
    let root = resolve_package_root(&zip_path, &cache).expect("resolve");
    assert!(root.join(PROJECT_FILE).is_file());
    assert!(cache.exists(), "view-only open still extracts archive to cache today");
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

// --- PKG-05: Unpack / Pack (library-level zip only) ---

#[test]
fn pkg05_pack_from_directory_does_not_touch_sibling_plain_folder() {
    let dir = temp_root("pack");
    let _ = fs::remove_dir_all(&dir);
    let plain = dir.join("PlainShow");
    fs::create_dir_all(&plain).expect("plain");
    fs::write(plain.join("take.mp4"), b"video").expect("video");
    let pkg_dir = dir.join("Pkg");
    let _pkg = create_package(&pkg_dir, "Pkg").expect("create");
    let out = dir.join("out.unibrow");
    export_package_to_zip(&pkg_dir, &out).expect("pack");
    assert!(!plain.join(MANIFEST_FILE).exists());
    assert!(!plain.join(PROJECT_FILE).exists());
    assert!(is_unibrow_path(&out));
    let _ = fs::remove_dir_all(&dir);
}

// --- PKG-07: Atomic journalled save ---

#[test]
fn pkg07_archive_save_is_atomic_and_readable() {
    let dir = temp_root("arc-save");
    let _ = fs::remove_dir_all(&dir);
    let pkg_dir = dir.join("bundle");
    let _pkg = create_package(&pkg_dir, "Arc").expect("create");
    let zip_path = dir.join("pkg.unibrow");
    save_archive_package(&zip_path, &pkg_dir).expect("save");
    assert!(archive_zip_is_readable(&zip_path));
    let before = fs::read(&zip_path).expect("read");
    set_save_fault_hook(Some(Box::new(|p| p == SaveFaultPoint::BeforeArchiveRename)));
    let mut pkg = open_package(&pkg_dir).expect("open");
    pkg.manifest.title = "New".into();
    unibrowser_project::write_manifest(&pkg_dir, &pkg.manifest).expect("write");
    assert!(save_archive_package(&zip_path, &pkg_dir).is_err());
    set_save_fault_hook(None);
    assert_eq!(fs::read(&zip_path).expect("read"), before);
    assert!(archive_zip_is_readable(&zip_path));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg07_folder_save_recovers_after_interrupted_journal() {
    let dir = temp_root("fold-recover");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Fold").expect("create");
    set_save_fault_hook(Some(Box::new(|p| p == SaveFaultPoint::BeforeSaveCommitJournal)));
    pkg.manifest.title = "Broken".into();
    assert!(save_folder_package(&pkg).is_err());
    set_save_fault_hook(None);
    assert!(pending_folder_save_id(&dir).expect("pending").is_some());
    recover_incomplete_folder_save(&dir).expect("recover");
    let again = open_package(&dir).expect("open");
    assert_eq!(again.manifest.title, "Fold");
    let _ = fs::remove_dir_all(&dir);
}

// --- PKG-08: Referenced media and Collect ---

#[test]
fn pkg08_collect_api_embeds_linked_media() {
    let dir = temp_root("refs");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Refs").expect("create");
    let external = dir.join("clip.bin");
    fs::write(&external, b"linked-bytes").expect("write");
    add_media_ref(&mut pkg, &external, "Clip").expect("ref");
    assert_eq!(list_media_refs(&dir).expect("list").len(), 1);
    let result = collect_all_media_refs(&mut pkg).expect("collect");
    assert_eq!(result.embedded_hashes.len(), 1);
    assert!(list_media_refs(&dir).expect("list").is_empty());
    let parts = list_referenced_package_parts(&dir, &pkg.manifest);
    assert!(parts.iter().any(|p| p.starts_with("assets/")));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg08_gc_retains_hashes_referenced_in_history() {
    let dir = temp_root("gc-hist");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Hist").expect("create");
    let src = dir.join("keep.bin");
    fs::write(&src, b"keep-me").expect("write");
    add_asset_copy(&mut pkg, &src, "file", "Keep").expect("asset");
    let hash = pkg.manifest.nodes[0].asset.clone().expect("hash");
    pkg.manifest.nodes.clear();
    write_manifest(&dir, &pkg.manifest).expect("clear nodes");
    let hist_path = dir.join(HISTORY_FILE);
    let (_, ops) =
        unibrowser_document_model::history_file::read_header_and_ops(&hist_path).expect("read");
    let entry = unibrowser_document_model::history_file::HistoryLogEntry {
        op_id: "snap-1".into(),
        seq: ops.last().map(|o| o.seq + 1).unwrap_or(1),
        parent_seq: ops.last().map(|o| o.seq).unwrap_or(0),
        timestamp: unibrowser_document_model::history_file::now_rfc3339(),
        plugin_id: "unibrowser-core".into(),
        plugin_version: "0.1.0".into(),
        op_type: "unibrowser.history.snapshot".into(),
        op_schema_version: 1,
        payload: serde_json::json!({ "asset_hash": hash }),
    };
    unibrowser_document_model::history_file::push_op_entry(&hist_path, entry).expect("push");
    let removed = gc_content_addressed_assets(&dir, &pkg.manifest, None).expect("gc");
    assert!(!removed.contains(&hash));
    assert!(dir.join(DIR_ASSETS).join(&hash).is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg08_gc_drops_unreferenced_hashes_not_in_history() {
    let dir = temp_root("gc");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Gc").expect("create");
    let src = dir.join("a.bin");
    fs::write(&src, b"a").expect("write");
    add_asset_copy(&mut pkg, &src, "file", "A").expect("asset");
    let hash = pkg.manifest.nodes[0].asset.clone().expect("hash");
    pkg.manifest.nodes.clear();
    write_manifest(&dir, &pkg.manifest).expect("write");
    let orphan = "1111111111111111111111111111111111111111111111111111111111111111";
    fs::write(dir.join(DIR_ASSETS).join(orphan), b"orphan").expect("orphan");
    let removed = gc_content_addressed_assets(&dir, &pkg.manifest, None).expect("gc");
    assert!(removed.contains(&hash));
    assert!(removed.contains(&orphan.to_string()));
    assert!(dir.join(HISTORY_FILE).is_file());
    let _ = fs::remove_dir_all(&dir);
}
