/** Canonical demo-pack sample.md used by doc-feature captures (web + desktop). */

export const DEMO_PACK_SAMPLE_MD_REL = "demo-pack/sample.md";

export async function fetchDemoPackSampleMarkdownWeb(): Promise<string> {
  const base = import.meta.env.BASE_URL ?? "/";
  const url = `${base.replace(/\/?$/, "/")}demo-pack/sample.md`;
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`Failed to load demo-pack sample.md (${res.status})`);
  }
  return res.text();
}
