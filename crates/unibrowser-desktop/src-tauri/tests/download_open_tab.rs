//! Integration-style check for download → new tab disposition wiring.

use std::path::Path;

use unibrowser_desktop_lib::downloads::apply_download_disposition;
use unibrowser_desktop_lib::preferences::DownloadPreference;
use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

#[test]
fn https_download_opens_http_tab_location() {
    let d = apply_download_disposition(
        "https://example.org/doc",
        None,
        Path::new("/tmp"),
        DownloadPreference::OpenInNewTab,
    )
        .expect("disposition");
    match d {
        DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::Http { url },
        } => assert_eq!(url, "https://example.org/doc"),
        _ => panic!("expected http tab"),
    }
}

#[test]
fn save_pref_returns_save_to_disk() {
    let d = apply_download_disposition(
        "https://example.org/a.bin",
        None,
        Path::new("/tmp"),
        DownloadPreference::SaveToDisk,
    )
    .expect("disposition");
    assert!(matches!(d, DownloadDisposition::SaveToDisk { .. }));
}

#[test]
fn platform_destination_hint_is_http_tab_at_disposition_layer() {
    let d = apply_download_disposition(
        "https://example.org/file.bin",
        Some(std::path::PathBuf::from("/tmp/Downloads/file.bin")),
        Path::new("/tmp"),
        DownloadPreference::OpenInNewTab,
    )
    .expect("disposition");
    match d {
        DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::Http { url },
        } => assert_eq!(url, "https://example.org/file.bin"),
        _ => panic!("disposition layer still returns http tab; shell fetches when destination hint present"),
    }
}

#[test]
fn rejects_data_url_download() {
    let err =
        apply_download_disposition(
            "data:text/plain,hello",
            None,
            Path::new("/tmp"),
            DownloadPreference::OpenInNewTab,
        )
        .unwrap_err();
    assert!(err.contains("unsupported"));
}
