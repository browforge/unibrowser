import { marked } from "marked";
import type { LiveBlock } from "./markdownLiveMapping";

marked.setOptions({ gfm: true, breaks: false });

export function renderLiveBlockHtml(block: LiveBlock): string {
  if (block.kind === "unsupported") {
    return "";
  }
  const raw = block.text.trimEnd();
  if (!raw) {
    return "<p></p>";
  }
  const html = marked.parse(raw, { async: false }) as string;
  return html.trim();
}
