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
  return html
    .replace(
      "<h1>",
      '<h1 style="font-size:34px;line-height:1.12;font-weight:700;margin:0.15em 0 0.2em">',
    )
    .replace(
      "<h2>",
      '<h2 style="font-size:26px;line-height:1.15;font-weight:700;margin:0.3em 0 0.2em">',
    )
    .replace(
      "<blockquote>",
      '<blockquote style="margin:0.35em 0;padding-left:0.75em;border-left:3px solid rgba(45,100,125,0.45)">',
    )
    .trim();
}
