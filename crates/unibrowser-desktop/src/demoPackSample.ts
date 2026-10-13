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

const CAPTURE_SAMPLE_MD_PATHS = [
  "examples/demo-pack/sample.md",
  DEMO_PACK_SAMPLE_MD_REL,
  "demo-pack/sample.md",
] as const;

export async function readDemoPackSampleMarkdownDesktopCapture(): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  for (const path of CAPTURE_SAMPLE_MD_PATHS) {
    try {
      const bytes = await invoke<number[]>("read_document_bytes", { path });
      const text = new TextDecoder().decode(Uint8Array.from(bytes));
      if (text.trim().length > 0) {
        return text;
      }
    } catch {
      /* try next path */
    }
  }
  const { getActiveTextContent } = await import("./textEditor");
  const fromEditor = getActiveTextContent() ?? "";
  if (fromEditor.trim().length > 20) {
    return fromEditor;
  }
  throw new Error("examples/demo-pack/sample.md did not load for doc-visual-md-docx");
}
