import { t } from "./i18n";
import { fileTypeVisualForPath } from "./fileTypeIcons";
import { isOpenVsxNetworkEnabled, setOpenVsxNetworkEnabled } from "./modsOpenVsx";
import { showToast } from "./toast";

export type ModCatalogEntry = {
  id: string;
  title: string;
  kind: string;
  blurb: string;
  open_vsx_id?: string | null;
  ubmod_id?: string | null;
};

export type ModSuggestion = {
  mod_id: string;
  title: string;
  reason: string;
  licence?: string | null;
  permission_summary?: string | null;
  open_vsx_id?: string | null;
  ubmod_id?: string | null;
};

type ModCardState = "builtin" | "installed" | "installable";

function cardState(
  entry: { id: string; kind: string; ubmod_id?: string | null; open_vsx_id?: string | null },
  nativeNames: Set<string>,
  vsixIds: Set<string>,
): ModCardState {
  if (entry.ubmod_id && nativeNames.has(entry.ubmod_id)) {
    return "installed";
  }
  if (entry.open_vsx_id && vsixIds.has(entry.open_vsx_id)) {
    return "installed";
  }
  if (entry.kind === "ubmod" || entry.ubmod_id) {
    return "builtin";
  }
  return "installable";
}

function badgeLabel(state: ModCardState): string {
  if (state === "builtin") {
    return "Built-in";
  }
  if (state === "installed") {
    return "Installed";
  }
  return "Available";
}

function actionLabel(state: ModCardState, hasOpenVsx: boolean): string {
  if (state === "installed") {
    return "Open";
  }
  if (state === "builtin") {
    return "Open";
  }
  return hasOpenVsx ? "Install" : "Install";
}

export function renderModNetworkStatus(host: HTMLElement): () => void {
  const row = document.createElement("p");
  row.className = "mod-portal-network-status viewer-note";
  const refresh = () => {
    const on = isOpenVsxNetworkEnabled();
    row.textContent = on
      ? "Open VSX network is on (Mods toolbar). Search can reach the public catalogue."
      : "Open VSX network is off. Turn it on with Mods → Open VSX network to search.";
    row.dataset.state = on ? "on" : "off";
  };
  refresh();
  host.appendChild(row);
  const onChange = () => refresh();
  window.addEventListener("unibrowser-open-vsx-network", onChange);
  return () => window.removeEventListener("unibrowser-open-vsx-network", onChange);
}

export function appendModCatalogCard(
  parent: HTMLElement,
  entry: ModCatalogEntry,
  nativeNames: Set<string>,
  vsixIds: Set<string>,
  onPrimary: (entry: ModCatalogEntry) => void,
): void {
  const state = cardState(entry, nativeNames, vsixIds);
  const card = document.createElement("article");
  card.className = "mod-result-card";
  const badge = document.createElement("span");
  badge.className = `mod-card-badge mod-card-badge--${state}`;
  badge.textContent = badgeLabel(state);
  const title = document.createElement("strong");
  title.textContent = entry.title;
  const id = document.createElement("span");
  id.className = "mod-result-id";
  id.textContent = entry.id;
  const reason = document.createElement("span");
  reason.className = "mod-result-reason";
  reason.textContent = entry.blurb;
  const footer = document.createElement("div");
  footer.className = "mod-card-footer";
  const meta = document.createElement("span");
  meta.className = "mod-result-meta";
  const source = entry.open_vsx_id ? "Open VSX" : "Built-in catalogue";
  meta.textContent = `Source: ${source} | Licence: check before install | Permissions: review before install`;
  const action = document.createElement("button");
  action.type = "button";
  action.className = "mod-card-action";
  action.textContent = actionLabel(state, Boolean(entry.open_vsx_id));
  action.onclick = () => onPrimary(entry);
  footer.append(meta, action);
  card.append(badge, title, id, reason, footer);
  parent.appendChild(card);
}

export function appendModSuggestionCard(
  parent: HTMLElement,
  suggestion: ModSuggestion,
  onPrimary: (suggestion: ModSuggestion) => void,
  options?: { featured?: boolean },
): void {
  const card = document.createElement("article");
  card.className = "mod-result-card mod-result-card--suggestion";
  const featured = Boolean(options?.featured);
  if (featured) {
    card.classList.add("mod-result-card--featured");
  }
  const badge = document.createElement("span");
  badge.className = "mod-card-badge mod-card-badge--installable";
  badge.textContent = "Suggested";
  const title = document.createElement("strong");
  if (featured) {
    title.className = "mod-suggestion-card-title";
  }
  title.textContent = suggestion.title;
  const reason = document.createElement("span");
  reason.className = "mod-result-reason";
  reason.textContent = suggestion.reason;
  const footer = document.createElement("div");
  footer.className = "mod-card-footer";
  const meta = document.createElement("span");
  meta.className = "mod-result-meta";
  const licence = suggestion.licence ?? "Licence: check before install";
  const permissions =
    suggestion.permission_summary ?? "Permissions: review before install";
  meta.textContent = `Licence: ${licence.replace(/^Licence:\s*/i, "")} · ${permissions}`;
  const action = document.createElement("button");
  action.type = "button";
  action.className = featured
    ? "mod-card-install mod-card-install--flat"
    : "mod-card-action mod-card-action--primary";
  action.textContent = "Install";
  action.onclick = () => onPrimary(suggestion);
  const details = document.createElement("details");
  details.className = "mod-result-details";
  details.open = false;
  const summary = document.createElement("summary");
  summary.textContent = "Contribution details";
  const detailBody = document.createElement("span");
  detailBody.className = "mod-contribution-ids";
  const parts = [`mod ${suggestion.mod_id}`];
  if (suggestion.ubmod_id) {
    parts.push(`ubmod ${suggestion.ubmod_id}`);
  }
  if (suggestion.open_vsx_id) {
    parts.push(`Open VSX ${suggestion.open_vsx_id}`);
  }
  detailBody.textContent = parts.join(" · ");
  details.append(summary, detailBody);
  if (featured) {
    const thumb = document.createElement("div");
    thumb.className = "mod-card-thumb";
    thumb.setAttribute("aria-hidden", "true");
    const visual = fileTypeVisualForPath("sample.step");
    const ft = document.createElement("span");
    ft.className = `ft-icon ${visual.className}`;
    ft.textContent = visual.label;
    thumb.appendChild(ft);
    const body = document.createElement("div");
    body.className = "mod-card-body";
    body.append(title, reason, meta, details);
    const installSlot = document.createElement("div");
    installSlot.className = "mod-card-install-slot";
    installSlot.appendChild(action);
    card.append(thumb, body, installSlot);
  } else {
    footer.append(action);
    card.append(badge, title, reason, meta, details, footer);
  }
  parent.appendChild(card);
}

export function enableOpenVsxForSearch(): void {
  if (!isOpenVsxNetworkEnabled()) {
    setOpenVsxNetworkEnabled(true);
    showToast(t("mods.catalogOpenVsx"), "info");
  }
}
