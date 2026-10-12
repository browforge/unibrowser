import { t } from "./i18n";

const NOTICE_ID = "ub-web-privacy-notice";

export function mountWebPrivacyNotice() {
  if (document.getElementById(NOTICE_ID)) {
    return;
  }
  const bar = document.createElement("div");
  bar.id = NOTICE_ID;
  bar.className = "ub-web-privacy-notice";
  bar.setAttribute("role", "status");
  const text = document.createElement("p");
  text.className = "ub-web-privacy-notice-text";
  text.textContent = t("web.privacyLead");
  const link = document.createElement("a");
  link.href = "https://github.com/browforge/unibrowser/releases/latest";
  link.rel = "noopener noreferrer";
  link.target = "_blank";
  link.className = "ub-web-privacy-notice-link";
  link.textContent = t("web.desktopDownload");
  bar.append(text, " ", link);
  const chrome = document.getElementById("chrome");
  if (chrome?.parentElement) {
    chrome.parentElement.insertBefore(bar, chrome);
  } else {
    document.body.prepend(bar);
  }
  document.body.classList.add("ub-platform-web");
}
