/**
 * LIVE block markdown: block spans, render-to-source mapping, smart Enter.
 * Operates on JavaScript string indices (Monaco UTF-16).
 */

export type LiveBlockKind =
  | "paragraph"
  | "heading"
  | "blockquote"
  | "bullet_list"
  | "ordered_list"
  | "unsupported";

export interface LiveBlock {
  kind: LiveBlockKind;
  start: number;
  end: number;
  text: string;
}

export interface RenderCharMap {
  /** Visible characters when markers are hidden (inactive render / click target). */
  render: string;
  /** render[i] corresponds to source index map[i]. */
  map: number[];
  /** Marker ranges in source (for dimmed decorations in active block). */
  markers: Array<{ start: number; end: number }>;
}

export interface SmartEnterResult {
  text: string;
  cursor: number;
}

export function splitLiveBlocks(source: string): LiveBlock[] {
  if (!source) {
    return [{ kind: "paragraph", start: 0, end: 0, text: "" }];
  }
  const spans: Array<{ start: number; end: number }> = [];
  let start = 0;
  let i = 0;
  let inFence = false;
  while (i < source.length) {
    if (source.startsWith("```", i)) {
      inFence = !inFence;
      i += 3;
      continue;
    }
    if (!inFence && source.startsWith("\n\n", i)) {
      spans.push({ start, end: i });
      start = i + 2;
      i = start;
      continue;
    }
    if (!inFence && source.startsWith("\r\n\r\n", i)) {
      spans.push({ start, end: i });
      start = i + 4;
      i = start;
      continue;
    }
    i += 1;
  }
  if (start < source.length) {
    spans.push({ start, end: source.length });
  }
  if (spans.length === 0) {
    spans.push({ start: 0, end: source.length });
  }
  return spans.map(({ start: s, end: e }) => {
    const text = source.slice(s, e);
    return { kind: classifyLiveBlock(text), start: s, end: e, text };
  });
}

export function blockIndexForOffset(blocks: LiveBlock[], offset: number): number {
  for (let idx = 0; idx < blocks.length; idx += 1) {
    const b = blocks[idx]!;
    if (offset >= b.start && offset < b.end) {
      return idx;
    }
  }
  if (blocks.length === 0) {
    return 0;
  }
  return blocks.length - 1;
}

/** Returns -1 when the caret sits in inter-block whitespace (all blocks rendered). */
export function activeLiveBlockIndex(blocks: LiveBlock[], offset: number): number {
  for (let i = 0; i < blocks.length - 1; i += 1) {
    const gapStart = blocks[i]!.end;
    const gapEnd = blocks[i + 1]!.start;
    if (offset >= gapStart && offset < gapEnd) {
      return -1;
    }
  }
  return blockIndexForOffset(blocks, offset);
}

export function buildBlockRenderMap(block: LiveBlock): RenderCharMap | null {
  if (block.kind === "unsupported") {
    return null;
  }
  const markers: Array<{ start: number; end: number }> = [];
  let body = block.text;
  let bodyOffset = block.start;

  if (block.kind === "heading") {
    const m = /^(#{1,6})(\s)/.exec(body);
    if (m) {
      markers.push({ start: bodyOffset, end: bodyOffset + m[1].length + m[2].length });
      body = body.slice(m[0].length);
      bodyOffset += m[0].length;
    }
  } else if (block.kind === "blockquote") {
    const lines = body.split("\n");
    let local = 0;
    const parts: string[] = [];
    for (const line of lines) {
      const prefix = /^>\s?/.exec(line);
      if (prefix) {
        markers.push({
          start: bodyOffset + local,
          end: bodyOffset + local + prefix[0].length,
        });
        parts.push(line.slice(prefix[0].length));
      } else {
        parts.push(line);
      }
      local += line.length + 1;
    }
    body = parts.join("\n");
  } else if (block.kind === "bullet_list" || block.kind === "ordered_list") {
    const lines = body.split("\n");
    let local = 0;
    const parts: string[] = [];
    for (const line of lines) {
      const bullet = /^(-|\*|\+)\s/.exec(line);
      const ordered = /^(\d+)\.\s/.exec(line);
      const prefix = bullet ?? ordered;
      if (prefix) {
        markers.push({
          start: bodyOffset + local,
          end: bodyOffset + local + prefix[0].length,
        });
        parts.push(line.slice(prefix[0].length));
      } else {
        parts.push(line);
      }
      local += line.length + 1;
    }
    body = parts.join("\n");
  }

  const inline = buildInlineRenderMap(body, bodyOffset);
  return {
    render: inline.render,
    map: inline.map,
    markers: [...markers, ...inline.markers],
  };
}

export function buildInlineRenderMap(text: string, baseOffset = 0): RenderCharMap {
  const renderChars: string[] = [];
  const map: number[] = [];
  const markers: Array<{ start: number; end: number }> = [];
  let i = 0;
  while (i < text.length) {
    const rest = text.slice(i);
    const parsed = tryInlineToken(rest);
    if (parsed) {
      markers.push({
        start: baseOffset + i,
        end: baseOffset + i + parsed.markerBefore,
      });
      if (parsed.markerAfter > 0) {
        markers.push({
          start: baseOffset + i + parsed.consumed - parsed.markerAfter,
          end: baseOffset + i + parsed.consumed,
        });
      }
      for (let j = 0; j < parsed.visible.length; j += 1) {
        renderChars.push(parsed.visible[j]!);
        map.push(baseOffset + i + parsed.sourceIndices[j]!);
      }
      i += parsed.consumed;
      continue;
    }
    renderChars.push(rest[0]!);
    map.push(baseOffset + i);
    i += 1;
  }
  return { render: renderChars.join(""), map, markers };
}

interface ParsedInlineToken {
  visible: string;
  sourceIndices: number[];
  consumed: number;
  markerBefore: number;
  markerAfter: number;
}

function tryInlineToken(rest: string): ParsedInlineToken | null {
  if (rest.startsWith("[") && !rest.startsWith("![")) {
    return tryLink(rest);
  }
  if (rest.startsWith("`")) {
    return tryWrapped(rest, "`", "`");
  }
  if (rest.startsWith("**")) {
    return tryWrapped(rest, "**", "**");
  }
  if (rest.startsWith("__")) {
    return tryWrapped(rest, "__", "__");
  }
  if (rest.startsWith("~~")) {
    return tryWrapped(rest, "~~", "~~");
  }
  if (rest.startsWith("*")) {
    return tryWrapped(rest, "*", "*");
  }
  if (rest.startsWith("_")) {
    return tryWrapped(rest, "_", "_");
  }
  return null;
}

function tryLink(rest: string): ParsedInlineToken | null {
  const labelEnd = rest.indexOf("]", 1);
  if (labelEnd < 0) {
    return null;
  }
  const after = rest.slice(labelEnd + 1);
  if (!after.startsWith("(")) {
    return null;
  }
  const urlEnd = after.indexOf(")", 1);
  if (urlEnd < 0) {
    return null;
  }
  const label = rest.slice(1, labelEnd);
  const consumed = labelEnd + 1 + urlEnd + 1;
  const sourceIndices = label.split("").map((_, idx) => 1 + idx);
  return {
    visible: label,
    sourceIndices,
    consumed,
    markerBefore: 1,
    markerAfter: urlEnd + 1,
  };
}

function tryWrapped(rest: string, open: string, close: string): ParsedInlineToken | null {
  const innerStart = open.length;
  const closeIdx = rest.indexOf(close, innerStart);
  if (closeIdx < 0 || closeIdx === innerStart) {
    return null;
  }
  const inner = rest.slice(innerStart, closeIdx);
  const nested = buildInlineRenderMap(inner, 0);
  const sourceIndices = nested.map.map((local) => innerStart + local);
  return {
    visible: nested.render,
    sourceIndices,
    consumed: closeIdx + close.length,
    markerBefore: open.length,
    markerAfter: close.length,
  };
}

export function sourceOffsetFromRenderClick(
  block: LiveBlock,
  renderOffset: number,
): number {
  const map = buildBlockRenderMap(block);
  if (!map || map.render.length === 0) {
    return block.start;
  }
  const clamped = Math.max(0, Math.min(renderOffset, map.render.length - 1));
  return map.map[clamped] ?? block.start;
}

export function applySmartEnter(
  source: string,
  cursor: number,
  shiftKey: boolean,
): SmartEnterResult {
  if (shiftKey) {
    return { text: insertAt(source, cursor, "\n"), cursor: cursor + 1 };
  }
  const blocks = splitLiveBlocks(source);
  const bi = blockIndexForOffset(blocks, cursor);
  const block = blocks[bi]!;
  if (block.kind === "unsupported") {
    return { text: insertAt(source, cursor, "\n\n"), cursor: cursor + 2 };
  }
  const rel = cursor - block.start;
  const lineStart = block.text.lastIndexOf("\n", rel - 1) + 1;
  const lineEndIdx = block.text.indexOf("\n", rel);
  const lineEnd = lineEndIdx < 0 ? block.text.length : lineEndIdx;
  const line = block.text.slice(lineStart, lineEnd);

  if (block.kind === "bullet_list") {
    const m = /^(-|\*|\+)\s(.*)$/.exec(line);
    if (m) {
      const body = m[2] ?? "";
      if (body.trim() === "" && rel >= block.start + lineStart + m[1].length + 1) {
        const removeFrom = block.start + lineStart;
        const removeTo = block.start + lineEnd + (lineEnd < block.text.length ? 1 : 0);
        const text = source.slice(0, removeFrom) + source.slice(removeTo);
        return { text, cursor: removeFrom };
      }
      const marker = `${m[1]} `;
      const insert = `\n${marker}`;
      return {
        text: insertAt(source, block.start + lineEnd, insert),
        cursor: block.start + lineEnd + insert.length,
      };
    }
  }

  if (block.kind === "ordered_list") {
    const m = /^(\d+)\.\s(.*)$/.exec(line);
    if (m) {
      const body = m[2] ?? "";
      if (body.trim() === "" && rel >= block.start + lineStart + m[1].length + 2) {
        const removeFrom = block.start + lineStart;
        const removeTo = block.start + lineEnd + (lineEnd < block.text.length ? 1 : 0);
        const text = source.slice(0, removeFrom) + source.slice(removeTo);
        return { text, cursor: removeFrom };
      }
      const next = String(Number(m[1]) + 1);
      const insert = `\n${next}. `;
      return {
        text: insertAt(source, block.start + lineEnd, insert),
        cursor: block.start + lineEnd + insert.length,
      };
    }
  }

  if (block.kind === "blockquote") {
    const prefix = /^>\s?/.exec(line);
    const marker = prefix ? line.slice(0, prefix[0].length) : "> ";
    const insert = `\n${marker}`;
    return {
      text: insertAt(source, block.start + lineEnd, insert),
      cursor: block.start + lineEnd + insert.length,
    };
  }

  return { text: insertAt(source, cursor, "\n\n"), cursor: cursor + 2 };
}

function insertAt(text: string, offset: number, chunk: string): string {
  return text.slice(0, offset) + chunk + text.slice(offset);
}

function classifyLiveBlock(text: string): LiveBlockKind {
  const trimmed = text.trimEnd();
  if (!trimmed) {
    return "paragraph";
  }
  if (trimmed.startsWith("```")) {
    return "unsupported";
  }
  if (/^#{1,6}\s/.test(trimmed)) {
    return "heading";
  }
  if (trimmed.startsWith(">")) {
    return "blockquote";
  }
  const firstLine = trimmed.split("\n")[0] ?? "";
  if (/^(-|\*|\+)\s/.test(firstLine)) {
    return "bullet_list";
  }
  if (/^\d+\.\s/.test(firstLine)) {
    return "ordered_list";
  }
  if (/^\|.+\|\s*$/m.test(trimmed)) {
    return "unsupported";
  }
  if (trimmed.startsWith("$$") || trimmed.startsWith("$")) {
    return "unsupported";
  }
  if (trimmed.startsWith("```mermaid") || trimmed.startsWith(":::")) {
    return "unsupported";
  }
  return "paragraph";
}
