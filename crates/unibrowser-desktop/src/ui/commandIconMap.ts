import type { LucideName } from "./lucideIcon";

/** Engine control `icon` strings → bundled Lucide glyphs (must cover all of `families.rs`). */
export const CONTROL_ICON_MAP: Record<string, LucideName> = {
  undo: "undo-2",
  redo: "redo-2",
  search: "search",
  download: "download",
  "spell-check": "spell-check",
  "pen-line": "pen-line",
  "folder-plus": "folder",
  "file-plus": "file-text",
  "folder-open": "folder",
  move: "move",
  pencil: "pen-line",
  archive: "package-open",
  trash: "x",
  "arrow-up-down": "arrow-up-down",
  "arrow-down-up": "arrow-down-up",
  eye: "eye",
  "zoom-in": "zoom-in",
  "zoom-out": "zoom-out",
  maximize: "maximize",
  share: "download",
  "refresh-cw": "rotate-cw",
  shield: "shield",
  globe: "globe",
  pin: "pin",
  copy: "copy",
  file: "file",
  type: "type",
  save: "save",
  bold: "bold",
  italic: "italic",
  underline: "underline",
  strikethrough: "strikethrough",
  subscript: "subscript",
  superscript: "superscript",
  palette: "palette",
  highlighter: "highlighter",
  stamp: "stamp",
  image: "image",
  extension: "package-open",
  plus: "plus",
  bookmark: "bookmark",
  play: "play",
  "circle-dot": "circle-dot",
  "circle-alert": "circle-alert",
  clock: "clock",
  "layout-grid": "layout-grid",
  ruler: "ruler",
  lasso: "lasso",
  "message-square": "message-square",
  "square-dashed": "square-dashed",
  info: "info",
  folder: "folder",
  x: "x",
};

export function lucideForControlIcon(icon: string): LucideName {
  return CONTROL_ICON_MAP[icon] ?? "file";
}

export function controlIconUsesFallback(icon: string): boolean {
  return !(icon in CONTROL_ICON_MAP);
}
