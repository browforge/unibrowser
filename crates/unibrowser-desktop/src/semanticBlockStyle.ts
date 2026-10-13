/** Cursor-line block style inference for the compact toolbar picker (no Monaco imports). */

const BLOCK_STYLE_BY_LEVEL: { commandId: string; label: string }[] = [
  { commandId: "unibrowser.semantic.block.heading_1", label: "Heading 1" },
  { commandId: "unibrowser.semantic.block.heading_2", label: "Heading 2" },
  { commandId: "unibrowser.semantic.block.heading_3", label: "Heading 3" },
  { commandId: "unibrowser.semantic.block.heading_4", label: "Heading 4" },
  { commandId: "unibrowser.semantic.block.heading_5", label: "Heading 5" },
  { commandId: "unibrowser.semantic.block.heading_6", label: "Heading 6" },
];

export type InferredBlockStyle = { commandId: string; label: string };

export function inferBlockStyleFromMarkdownLine(line: string): InferredBlockStyle {
  const heading = line.match(/^(#{1,6})\s/);
  if (heading) {
    const entry = BLOCK_STYLE_BY_LEVEL[heading[1].length - 1];
    if (entry) {
      return entry;
    }
  }
  if (/^>\s/.test(line)) {
    return {
      commandId: "unibrowser.semantic.block.blockquote",
      label: "Block quote",
    };
  }
  if (/^```/.test(line)) {
    return {
      commandId: "unibrowser.semantic.block.code_block",
      label: "Code block",
    };
  }
  return {
    commandId: "unibrowser.semantic.block.paragraph",
    label: "Paragraph",
  };
}

export function inferSemanticBlockStyleAtCursor(
  getLineContent: (line: number) => string | null,
  cursorLine: number,
  isMarkdown: boolean,
): InferredBlockStyle {
  const demoLabel = document.body.dataset.ubBlockStyleLabel?.trim();
  const demoCommand = document.body.dataset.ubBlockStyleCommandId?.trim();
  if (demoLabel && demoCommand) {
    return { commandId: demoCommand, label: demoLabel };
  }
  if (!isMarkdown || cursorLine < 1) {
    return {
      commandId: "unibrowser.semantic.block.paragraph",
      label: "Paragraph",
    };
  }
  const text = getLineContent(cursorLine);
  if (text === null) {
    return {
      commandId: "unibrowser.semantic.block.paragraph",
      label: "Paragraph",
    };
  }
  return inferBlockStyleFromMarkdownLine(text);
}
