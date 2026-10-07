export type CatalogInstallPath = "local" | "network-off" | "network-on";

export function catalogInstallPath(
  entry: { open_vsx_id?: string | null; ubmod_id?: string | null },
  networkOn: boolean,
): CatalogInstallPath {
  if (entry.ubmod_id) {
    return "local";
  }
  if (entry.open_vsx_id && !networkOn) {
    return "network-off";
  }
  return "network-on";
}
