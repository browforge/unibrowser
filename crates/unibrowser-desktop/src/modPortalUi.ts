import { t } from "./i18n";
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
  if (options?.featured) {
    card.classList.add("mod-result-card--featured");
  }
  const badge = document.createElement("span");
  badge.className = "mod-card-badge mod-card-badge--installable";
  badge.textContent = "Suggested";
  const title = document.createElement("strong");
  title.textContent = suggestion.title;
  const id = document.createElement("span");
  id.className = "mod-result-id";
  const ubmod = suggestion.ubmod_id ? ` · .ubmod ${suggestion.ubmod_id}` : "";
  id.textContent = `${suggestion.mod_id}${ubmod}`;
  const reason = document.createElement("span");
  reason.className = "mod-result-reason";
  reason.textContent = suggestion.reason;
  const footer = document.createElement("div");
  footer.className = "mod-card-footer";
  const meta = document.createElement("span");
  meta.className = "mod-result-meta";
  meta.textContent =
    "Source: built-in catalogue | Licence: check before install | Permissions: review before install";
  const action = document.createElement("button");
  action.type = "button";
  action.className = "mod-card-action";
  action.textContent = suggestion.open_vsx_id ? "Install" : "Open";
  action.onclick = () => onPrimary(suggestion);
  footer.append(meta, action);
  card.append(badge, title, id, reason, footer);
  parent.appendChild(card);
}

export function enableOpenVsxForSearch(): void {
  if (!isOpenVsxNetworkEnabled()) {
    setOpenVsxNetworkEnabled(true);
    showToast(t("mods.catalogOpenVsx"), "info");
  }
}
