//! Integration-style check for download → new tab disposition wiring.

use std::path::Path;

use unibrowser_desktop_lib::downloads::apply_download_disposition;
use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

#[test]
fn https_download_opens_http_tab_location() {
    let d = apply_download_disposition("https://example.org/doc", None, Path::new("/tmp"))
        .expect("disposition");
    match d {
        DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::Http { url },
        } => assert_eq!(url, "https://example.org/doc"),
        _ => panic!("expected http tab"),
    }
}

#[test]
fn rejects_data_url_download() {
    let err =
        apply_download_disposition("data:text/plain,hello", None, Path::new("/tmp")).unwrap_err();
    assert!(err.contains("unsupported"));
}
