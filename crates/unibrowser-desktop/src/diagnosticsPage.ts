import { invoke } from "@tauri-apps/api/core";
import { appendCrashLogsSection } from "./crashLogsLink";
import { t } from "./i18n";

interface FfmpegProbe {
  available: boolean;
  binary: string | null;
  version_line: string | null;
}

interface DiagnosticLogEntry {
  at_ms: number;
  message: string;
  details?: string | null;
}

interface VsixActivationDto {
  extension_id: string;
  matched_event: string;
  command_id: string;
  at_ms: number;
}

interface ExtensionHostDiagnosticsDto {
  host_version: string;
  host_running: boolean;
  host_message: string;
  activated_extensions: VsixActivationDto[];
}

interface DiagnosticsInfoDto {
  mods_loaded: string;
  mod_catalog_count: number;
  ffmpeg: FfmpegProbe;
  webview_stack: string;
  web_capture_native: boolean;
  web_capture_detail: string;
  web_download_hook_wired: boolean;
  web_download_hook_detail: string;
  extension_host: ExtensionHostDiagnosticsDto;
  target_os: string;
  target_arch: string;
  recent_errors: DiagnosticLogEntry[];
  user_mods_disabled: boolean;
  safe_mode_env: boolean;
  memory_rss_kb: number | null;
  memory_detail: string;
  crash_logs_dir: string | null;
}

export async function renderDiagnostics(root: HTMLElement) {
  root.innerHTML = "";
  const intro = document.createElement("p");
  intro.className = "viewer-note";
  intro.textContent = t("diagnostics.lead");
  root.appendChild(intro);

  let info: DiagnosticsInfoDto | null = null;
  try {
    info = await invoke<DiagnosticsInfoDto>("get_diagnostics_info");
  } catch {
    const err = document.createElement("p");
    err.textContent = t("diagnostics.loadError");
    root.appendChild(err);
    return;
  }

  const envSec = document.createElement("section");
  envSec.className = "settings-section";
  const envH = document.createElement("h3");
  envH.textContent = t("diagnostics.environment");
  envSec.appendChild(envH);
  const envList = document.createElement("ul");
  envList.className = "diagnostics-list";
  [
    `${t("diagnostics.webview")}: ${info.webview_stack}`,
    `${t("diagnostics.platform")}: ${info.target_os} (${info.target_arch})`,
    `${t("diagnostics.mods")}: ${info.mods_loaded} (${info.mod_catalog_count} bundled)`,
    info.user_mods_disabled
      ? t("diagnostics.safeModeOn", {
          env: info.safe_mode_env ? t("diagnostics.safeModeEnv") : t("diagnostics.safeModePref"),
        })
      : t("diagnostics.safeModeOff"),
    `${t("diagnostics.memory")}: ${info.memory_detail}`,
  ].forEach((line) => {
    const li = document.createElement("li");
    li.textContent = line;
    envList.appendChild(li);
  });
  envSec.appendChild(envList);
  root.appendChild(envSec);

  const capSec = document.createElement("section");
  capSec.className = "settings-section";
  const capH = document.createElement("h3");
  capH.textContent = t("diagnostics.webCapture");
  capSec.appendChild(capH);
  const capP = document.createElement("p");
  capP.textContent = info.web_capture_native
    ? t("diagnostics.webCaptureOk")
    : t("diagnostics.webCaptureGap");
  capSec.appendChild(capP);
  const capDetail = document.createElement("p");
  capDetail.className = "viewer-note";
  capDetail.textContent = info.web_capture_detail;
  capSec.appendChild(capDetail);
  root.appendChild(capSec);

  const dlSec = document.createElement("section");
  dlSec.className = "settings-section";
  const dlH = document.createElement("h3");
  dlH.textContent = t("diagnostics.webDownload");
  dlSec.appendChild(dlH);
  const dlP = document.createElement("p");
  dlP.textContent = info.web_download_hook_wired
    ? t("diagnostics.webDownloadOk")
    : t("diagnostics.webDownloadGap");
  dlSec.appendChild(dlP);
  const dlDetail = document.createElement("p");
  dlDetail.className = "viewer-note";
  dlDetail.textContent = info.web_download_hook_detail;
  dlSec.appendChild(dlDetail);
  root.appendChild(dlSec);

  const extSec = document.createElement("section");
  extSec.className = "settings-section";
  const extH = document.createElement("h3");
  extH.textContent = t("diagnostics.extensionHost");
  extSec.appendChild(extH);
  const extP = document.createElement("p");
  extP.textContent = `${info.extension_host.host_version} — ${info.extension_host.host_message}`;
  extSec.appendChild(extP);
  const actList = document.createElement("ul");
  actList.className = "diagnostics-list";
  if (!info.extension_host.activated_extensions.length) {
    const li = document.createElement("li");
    li.textContent = t("diagnostics.vsixNotActivated");
    actList.appendChild(li);
  } else {
    info.extension_host.activated_extensions.forEach((a) => {
      const li = document.createElement("li");
      li.textContent = `${a.extension_id} via ${a.matched_event} (${a.command_id})`;
      actList.appendChild(li);
    });
  }
  extSec.appendChild(actList);
  root.appendChild(extSec);

  const ffSec = document.createElement("section");
  ffSec.className = "settings-section";
  const ffH = document.createElement("h3");
  ffH.textContent = t("diagnostics.ffmpeg");
  ffSec.appendChild(ffH);
  const ffP = document.createElement("p");
  if (info.ffmpeg.available) {
    ffP.textContent = [
      info.ffmpeg.binary,
      info.ffmpeg.version_line,
    ]
      .filter(Boolean)
      .join(" · ");
  } else {
    ffP.textContent = t("diagnostics.ffmpegMissing");
  }
  ffSec.appendChild(ffP);
  root.appendChild(ffSec);

  const errSec = document.createElement("section");
  errSec.className = "settings-section";
  const errH = document.createElement("h3");
  errH.textContent = t("diagnostics.errors");
  errSec.appendChild(errH);
  if (!info.recent_errors.length) {
    const empty = document.createElement("p");
    empty.className = "viewer-note";
    empty.textContent = t("diagnostics.errorsEmpty");
    errSec.appendChild(empty);
  } else {
    const pre = document.createElement("pre");
    pre.className = "diagnostics-log";
    pre.textContent = info.recent_errors
      .slice()
      .reverse()
      .map((e) => {
        const when = new Date(e.at_ms).toISOString();
        const detail = e.details ? `\n  ${e.details}` : "";
        return `[${when}] ${e.message}${detail}`;
      })
      .join("\n\n");
    errSec.appendChild(pre);
  }
  root.appendChild(errSec);
  appendCrashLogsSection(root);
}
