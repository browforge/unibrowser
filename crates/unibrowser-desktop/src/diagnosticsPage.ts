import { invoke } from "@tauri-apps/api/core";
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

interface DiagnosticsInfoDto {
  mods_loaded: string;
  mod_catalog_count: number;
  ffmpeg: FfmpegProbe;
  webview_stack: string;
  target_os: string;
  target_arch: string;
  recent_errors: DiagnosticLogEntry[];
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
  ].forEach((line) => {
    const li = document.createElement("li");
    li.textContent = line;
    envList.appendChild(li);
  });
  envSec.appendChild(envList);
  root.appendChild(envSec);

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
}
