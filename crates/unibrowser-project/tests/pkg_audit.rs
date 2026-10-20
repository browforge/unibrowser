//! PKG-01..08 audit probes (ADR 0032). Tests document current behaviour on main.

use std::fs::{self, File};
use std::path::Path;

use zip::read::ZipArchive;
use zip::CompressionMethod;

use unibrowser_project::{
    add_asset_copy, create_package, export_package_to_zip, import_share_package_zip,
    is_unibrow_path, list_zip_archive_entries, materialize_zip_package, open_package,
    resolve_package_root, DIR_ASSETS, DIR_REFS, PROJECT_FILE,
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
fn pkg01_exported_zip_lacks_opc_content_types() {
    let dir = temp_root("opc");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Opc").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let listed = list_zip_archive_entries(&zip_path, 200).expect("list");
    assert!(
        !listed.iter().any(|e| e.path == "[Content_Types].xml"),
        "ADR 0032 OPC part missing today"
    );
    assert!(
        !listed.iter().any(|e| e.path.starts_with("_rels/")),
        "OPC relationships missing today"
    );
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_export_has_project_json_not_manifest_json() {
    let dir = temp_root("manifest");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "M").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let listed = list_zip_archive_entries(&zip_path, 200).expect("list");
    assert!(listed.iter().any(|e| e.path == "project.json" || e.path.ends_with("project.json")));
    assert!(
        !listed.iter().any(|e| e.path == "manifest.json"),
        "ADR 0032 root manifest.json not implemented"
    );
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_png_in_zip_uses_deflate_not_store() {
    let dir = temp_root("compress");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Media").expect("create");
    let png = dir.join("frame.png");
    fs::write(&png, [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0]).expect("png");
    add_asset_copy(&mut pkg, &png, "image", "frame").expect("asset");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let method = zip_compression_for(&zip_path, "assets/frame.png");
    assert_eq!(
        method,
        CompressionMethod::Deflated,
        "ADR 0032 expects STORED for PNG; export uses DEFLATE for all parts"
    );
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_identical_asset_bytes_are_not_content_hash_deduped() {
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
    let assets: Vec<_> = fs::read_dir(dir.join(DIR_ASSETS))
        .expect("assets dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(assets.len(), 2, "content-hash dedupe not implemented");
    assert!(
        !assets.iter().any(|n| n.len() == 64 && n.chars().all(|c| c.is_ascii_hexdigit())),
        "assets use original filenames, not assets/<hash>"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg01_zip_has_no_unibrow_history_part() {
    let dir = temp_root("hist-part");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "H").expect("create");
    let zip_path = dir.with_extension("unibrow");
    export_package_to_zip(&dir, &zip_path).expect("export");
    let listed = list_zip_archive_entries(&zip_path, 200).expect("list");
    assert!(
        !listed.iter().any(|e| e.path.contains(".unibrow-history")),
        "archive history part not serialised yet"
    );
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(&zip_path);
}

#[test]
fn pkg01_zip_roundtrip_preserves_project_json() {
    let dir = temp_root("rt");
    let _ = fs::remove_dir_all(&dir);
    let mut pkg = create_package(&dir, "Round").expect("create");
    pkg.manifest.title = "Round trip".into();
    unibrowser_project::write_manifest(&dir, &pkg.manifest).expect("write");
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

// --- PKG-02: Folder mode on-disk IO ---

#[test]
fn pkg02_unpacked_bundle_uses_project_json_at_root() {
    let dir = temp_root("folder-unpacked");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Folder").expect("create");
    assert!(dir.join(PROJECT_FILE).is_file());
    assert!(!dir.join("manifest.json").exists());
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
    assert!(!show.join(".unibrow-history").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pkg02_create_package_scaffolds_full_tree_not_plain_folder() {
    let dir = temp_root("scaffold");
    let _ = fs::remove_dir_all(&dir);
    let _pkg = create_package(&dir, "Scaffold").expect("create");
    assert!(dir.join(PROJECT_FILE).exists(), "ADR 2a defers project.json until unpack/collect");
    assert!(dir.join(DIR_REFS).exists());
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
    assert!(!unpacked.join("manifest.json").exists());
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
    assert!(!plain.join("manifest.json").exists());
    assert!(!plain.join(PROJECT_FILE).exists());
    assert!(is_unibrow_path(&out));
    let _ = fs::remove_dir_all(&dir);
}

// --- PKG-08: Referenced media ---

#[test]
fn pkg08_refs_directory_exists_but_no_collect_api() {
    let dir = temp_root("refs");
    let _ = fs::remove_dir_all(&dir);
    let pkg = create_package(&dir, "Refs").expect("create");
    assert!(pkg.root.join(DIR_REFS).is_dir());
    assert!(fs::read_dir(pkg.root.join(DIR_REFS)).expect("read").next().is_none());
    let _ = fs::remove_dir_all(&dir);
}
