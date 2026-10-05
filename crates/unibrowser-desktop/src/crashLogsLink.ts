import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { showErrorToast } from "./toast";

export function appendCrashLogsSection(root: HTMLElement) {
  const sec = document.createElement("section");
  sec.className = "settings-section";
  const h = document.createElement("h3");
  h.textContent = t("crashLogs.heading");
  sec.appendChild(h);
  const p = document.createElement("p");
  p.className = "viewer-note";
  p.textContent = t("crashLogs.lead");
  sec.appendChild(p);
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "recent-link";
  btn.textContent = t("crashLogs.openFolder");
  btn.onclick = () => {
    void invoke("open_crash_logs_folder").catch((err) => showErrorToast(err));
  };
  sec.appendChild(btn);
  root.appendChild(sec);
}
