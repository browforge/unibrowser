import { invoke } from "@tauri-apps/api/core";
import { appendCrashLogsSection } from "./crashLogsLink";
import { appendCrashRecoveryPanel } from "./crashRecoveryPanel";
import { t } from "./i18n";

type BuildInfoDto = {
  version: string;
  version_line: string;
  git_hash: string | null;
  bundled_mods_count: number;
  registry_commands_count: number;
};

export async function renderAbout(root: HTMLElement) {
  root.innerHTML = "";
  const build = await invoke<BuildInfoDto>("get_build_info").catch(() => null);
  if (build) {
    const ver = document.createElement("section");
    ver.className = "about-section";
    const vh = document.createElement("h3");
    vh.textContent = t("about.version");
    ver.appendChild(vh);
    const vp = document.createElement("p");
    vp.textContent = build.git_hash
      ? `Unibrowser ${build.version} (git ${build.git_hash})`
      : build.version_line;
    ver.appendChild(vp);
    const stats = document.createElement("p");
    stats.className = "viewer-note";
    stats.textContent = t("about.engineStats", {
      mods: String(build.bundled_mods_count),
      commands: String(build.registry_commands_count),
    });
    ver.appendChild(stats);
    root.appendChild(ver);
  }
  const lead = document.createElement("p");
  lead.textContent = t("about.lead");
  root.appendChild(lead);

  const licence = document.createElement("section");
  licence.className = "about-section";
  const lh = document.createElement("h3");
  lh.textContent = t("about.licenceHeading");
  licence.appendChild(lh);
  const lp = document.createElement("p");
  lp.textContent = t("about.licenceBody");
  licence.appendChild(lp);
  const licenceLink = document.createElement("p");
  const licA = document.createElement("a");
  licA.href = "https://www.apache.org/licenses/LICENSE-2.0";
  licA.target = "_blank";
  licA.rel = "noopener noreferrer";
  licA.textContent = t("about.licenceLinkApache");
  licenceLink.appendChild(licA);
  licence.appendChild(licenceLink);
  const noticeP = document.createElement("p");
  noticeP.className = "viewer-note";
  noticeP.textContent = t("about.noticePath");
  licence.appendChild(noticeP);
  root.appendChild(licence);

  const privacy = document.createElement("section");
  privacy.className = "about-section";
  const ph = document.createElement("h3");
  ph.textContent = t("about.privacyHeading");
  privacy.appendChild(ph);
  const noTel = document.createElement("p");
  noTel.className = "about-no-telemetry";
  noTel.textContent = t("about.noTelemetry");
  privacy.appendChild(noTel);
  const pp = document.createElement("p");
  pp.textContent = t("about.privacyBody");
  privacy.appendChild(pp);
  const privacyDoc = document.createElement("p");
  privacyDoc.className = "viewer-note";
  privacyDoc.textContent = t("about.privacyDocHint");
  privacy.appendChild(privacyDoc);
  root.appendChild(privacy);

  const third = document.createElement("section");
  third.className = "about-section";
  const th = document.createElement("h3");
  th.textContent = t("about.thirdPartyHeading");
  third.appendChild(th);
  const ul = document.createElement("ul");
  ul.className = "about-third-party";
  [
    "Monaco Editor — MIT",
    "pdf.js — Apache-2.0",
    "Mammoth — BSD-2-Clause",
    "Tauri / WebView — MIT / Apache-2.0",
    "ffmpeg (when used) — LGPL 2.1+",
  ].forEach((line) => {
    const li = document.createElement("li");
    li.textContent = line;
    ul.appendChild(li);
  });
  third.appendChild(ul);
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = t("about.thirdPartyNote");
  third.appendChild(note);
  root.appendChild(third);

  const recovery = document.createElement("section");
  recovery.className = "about-section";
  const rh = document.createElement("h3");
  rh.textContent = t("about.crashRecoveryHeading");
  recovery.appendChild(rh);
  const recoveryBody = document.createElement("div");
  recovery.appendChild(recoveryBody);
  await appendCrashRecoveryPanel(recoveryBody, { compact: true });
  root.appendChild(recovery);
  appendCrashLogsSection(root);
}
