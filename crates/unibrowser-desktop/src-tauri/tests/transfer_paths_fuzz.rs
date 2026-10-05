//! Adversarial names for copy/move must not escape destination or touch config.

use std::fs;
use std::path::PathBuf;

use unibrowser_desktop_lib::file_ops::transfer_paths;

fn adversarial_file_names() -> Vec<String> {
    vec![
        "normal.txt".into(),
        ".hidden".into(),
        "name with spaces.txt".into(),
        "unicode-文件名.txt".into(),
        "a".repeat(200),
    ]
}

#[test]
fn fuzz_transfer_rejects_traversal_names() {
    let root = std::env::temp_dir().join(format!("ub-transfer-fuzz-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("root");
    let dest = root.join("dest");
    fs::create_dir_all(&dest).expect("dest");

    for name in ["..", ".", "foo/bar", "foo\\bar"] {
        assert!(
            unibrowser_desktop_lib::file_ops::validate_entry_name(name).is_err(),
            "name {:?} should be rejected",
            name
        );
    }

    for name in adversarial_file_names() {
        let src = root.join("good-src");
        let _ = fs::remove_dir_all(&src);
        fs::create_dir_all(&src).expect("good-src");
        let file = src.join(name);
        fs::write(&file, b"data").expect("write");
        let before: Vec<_> = fs::read_dir(&dest)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert!(transfer_paths(&[file], &dest, true).is_ok());
        let after: Vec<_> = fs::read_dir(&dest)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(after.len(), before.len() + 1);
        let dest_canon = dest.canonicalize().expect("dest canon");
        for entry in fs::read_dir(&dest).unwrap() {
            let p = entry.unwrap().path();
            let c = p.canonicalize().unwrap_or(p);
            assert!(c.starts_with(&dest_canon));
        }
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn move_into_self_rejected() {
    let root = std::env::temp_dir().join(format!("ub-transfer-self-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let src = root.join("folder");
    fs::create_dir_all(&src).expect("mkdir");
    let err = transfer_paths(&[PathBuf::from(&src)], &src, false);
    assert!(err.is_err());
    let _ = fs::remove_dir_all(&root);
}
