export function defaultFirstSaveFileName(
  projectTitle?: string,
  untitledLabel = "Untitled project",
): string {
  const base = (projectTitle ?? untitledLabel).trim() || untitledLabel;
  return `${base}.unibrow`;
}

/** Shown before home directory resolves (capture / first paint). */
export function defaultFirstSaveFolderDisplay(): string {
  return "Documents/Unibrowser";
}

export function formatFirstSavePathPreview(
  location: string,
  mode: "archive" | "folder",
  fileName: string,
): string {
  const folder = location.replace(/\/$/, "");
  if (mode === "folder") {
    return `${folder}/${fileName}/`;
  }
  return `${folder}/${fileName}`;
}
