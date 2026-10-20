export function defaultFirstSaveFileName(
  projectTitle?: string,
  untitledLabel = "Untitled project",
): string {
  const base = (projectTitle ?? untitledLabel).trim() || untitledLabel;
  return `${base}.unibrow`;
}
