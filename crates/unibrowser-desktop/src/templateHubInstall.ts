export type HubTemplateCatalogEntry = {
  id: string;
  title: string;
  version: string;
  blurb: string;
  fixture_id: string;
  signed: boolean;
};

export type HubInstallPath = "offline" | "needs-network";

/** Hub template install routing (offline fixture vs blocked fetch). */
export function hubTemplateInstallPath(
  entry: HubTemplateCatalogEntry,
  offlineMode: boolean,
): HubInstallPath {
  if (entry.fixture_id) {
    return "offline";
  }
  return offlineMode ? "needs-network" : "needs-network";
}
