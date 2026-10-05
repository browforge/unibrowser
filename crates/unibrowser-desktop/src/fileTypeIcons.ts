/** Consistent file-type badges for the files panel (text labels, not emoji). */

export type FileTypeVisual = {
  label: string;
  className: string;
};

const EXT_MAP: Record<string, FileTypeVisual> = {
  pdf: { label: "PDF", className: "ft-pdf" },
  png: { label: "IMG", className: "ft-image" },
  jpg: { label: "IMG", className: "ft-image" },
  jpeg: { label: "IMG", className: "ft-image" },
  gif: { label: "IMG", className: "ft-image" },
  webp: { label: "IMG", className: "ft-image" },
  svg: { label: "SVG", className: "ft-vector" },
  mp4: { label: "VID", className: "ft-video" },
  webm: { label: "VID", className: "ft-video" },
  mkv: { label: "VID", className: "ft-video" },
  avi: { label: "VID", className: "ft-video" },
  mp3: { label: "AUD", className: "ft-audio" },
  wav: { label: "AUD", className: "ft-audio" },
  flac: { label: "AUD", className: "ft-audio" },
  ogg: { label: "AUD", className: "ft-audio" },
  mid: { label: "MID", className: "ft-audio" },
  midi: { label: "MID", className: "ft-audio" },
  zip: { label: "ZIP", className: "ft-archive" },
  unibrow: { label: "UBR", className: "ft-project" },
  docx: { label: "DOC", className: "ft-doc" },
  txt: { label: "TXT", className: "ft-text" },
  md: { label: "TXT", className: "ft-text" },
  json: { label: "JSN", className: "ft-code" },
  rs: { label: "RS", className: "ft-code" },
  ts: { label: "TS", className: "ft-code" },
  js: { label: "JS", className: "ft-code" },
  html: { label: "HTM", className: "ft-code" },
  css: { label: "CSS", className: "ft-code" },
  epub: { label: "EPB", className: "ft-doc" },
  csv: { label: "CSV", className: "ft-sheet" },
  tsv: { label: "CSV", className: "ft-sheet" },
  dxf: { label: "DXF", className: "ft-cad" },
};

export function fileTypeVisual(entry: {
  name: string;
  is_dir: boolean;
  is_image?: boolean;
  is_archive?: boolean;
}): FileTypeVisual {
  if (entry.is_dir) {
    return { label: "DIR", className: "ft-folder" };
  }
  if (entry.is_archive) {
    return { label: "ARC", className: "ft-archive" };
  }
  if (entry.is_image) {
    return { label: "IMG", className: "ft-image" };
  }
  const dot = entry.name.lastIndexOf(".");
  if (dot < 0) {
    return { label: "FILE", className: "ft-generic" };
  }
  const ext = entry.name.slice(dot + 1).toLowerCase();
  return EXT_MAP[ext] ?? { label: ext.slice(0, 3).toUpperCase() || "?", className: "ft-generic" };
}

export function appendFileTypeIcon(cell: HTMLElement, visual: FileTypeVisual) {
  const span = document.createElement("span");
  span.className = `ft-icon ${visual.className}`;
  span.textContent = visual.label;
  span.setAttribute("aria-hidden", "true");
  cell.appendChild(span);
}
