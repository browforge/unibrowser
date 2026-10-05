export const FILES_DRAG_MIME = "application/x-unibrowser-paths";

export function pathsFromDataTransfer(dt: DataTransfer | null): string[] {
  if (!dt) {
    return [];
  }
  const raw = dt.getData(FILES_DRAG_MIME);
  if (raw) {
    try {
      const parsed = JSON.parse(raw) as string[];
      if (Array.isArray(parsed)) {
        return parsed.filter((p) => typeof p === "string" && p.length > 0);
      }
    } catch {
      /* ignore */
    }
  }
  return [];
}

export function setPathsOnDataTransfer(dt: DataTransfer, paths: string[]) {
  dt.setData(FILES_DRAG_MIME, JSON.stringify(paths));
  dt.effectAllowed = "copyMove";
}
