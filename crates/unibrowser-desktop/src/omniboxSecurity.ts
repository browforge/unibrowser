export function isInsecureHttpNavigation(input: string): boolean {
  const trimmed = input.trim().toLowerCase();
  return trimmed.startsWith("http://");
}
