//! WebView2 `ICoreWebView2_4::DownloadStarting` → `unibrowser-download` (Windows).
//!
//! Compiles on CI `windows-latest`; behaviour not verified in the Linux cloud VM.

use std::collections::HashSet;
use std::sync::Mutex;

use tauri::{AppHandle, Webview};
use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};
use windows::core::PWSTR;

use crate::downloads::DownloadEventPayload;

fn take_pwstr(pw: PWSTR) -> String {
    if pw.is_null() {
        return String::new();
    }
    unsafe { pw.to_string() }.unwrap_or_default()
}
use crate::webview_download::emit_unibrowser_download;

static ATTACHED: Mutex<HashSet<String>> = Mutex::new(HashSet::new());

/// Register `DownloadStarting` once per content webview label.
pub fn try_attach_download_starting(app: AppHandle, webview: &Webview, label: &str) {
    {
        let mut set = ATTACHED.lock().unwrap_or_else(|e| e.into_inner());
        if set.contains(label) {
            return;
        }
    }
    let app2 = app.clone();
    let label2 = label.to_string();
    let attached = webview
        .with_webview(move |platform| {
            attach_inner(&app2, platform)?;
            Ok(())
        })
        .is_ok();
    if attached {
        let mut set = ATTACHED.lock().unwrap_or_else(|e| e.into_inner());
        set.insert(label2);
    }
}

fn attach_inner(
    app: &AppHandle,
    platform: tauri::webview::PlatformWebview,
) -> Result<(), String> {
    let controller = platform.controller();
    let core = controller.CoreWebView2().map_err(|e| e.to_string())?;
    let core4: ICoreWebView2_4 = core.cast().map_err(|e| e.to_string())?;
    let app = app.clone();
    let mut token = EventRegistrationToken::default();
    core4
        .add_DownloadStarting(
            &DownloadStartingEventHandler::create(Box::new(move |_sender, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                let uri = {
                    let mut uri = PWSTR::null();
                    let op = args.DownloadOperation()?;
                    op.Uri(&mut uri)?;
                    take_pwstr(uri)
                };
                let path = {
                    let mut path = PWSTR::null();
                    args.ResultFilePath(&mut path)?;
                    take_pwstr(path)
                };
                let payload = DownloadEventPayload {
                    url: uri,
                    destination: if path.is_empty() {
                        None
                    } else {
                        Some(path)
                    },
                };
                emit_unibrowser_download(&app, payload);
                args.SetCancel(true)?;
                Ok(())
            })),
            &mut token,
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
