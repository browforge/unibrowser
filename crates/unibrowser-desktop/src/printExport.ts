const PRINT_STYLES = `
  :root { color-scheme: light dark; }
  body {
    margin: 0;
    padding: 1.5rem;
    font-family: Georgia, "Times New Roman", serif;
    font-size: 11pt;
    line-height: 1.5;
    color: #111;
    background: #fff;
  }
  .ub-print-doc {
    max-width: 42rem;
    margin: 0 auto;
  }
  .ub-print-doc pre {
    font-family: ui-monospace, monospace;
    font-size: 10pt;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .ub-print-doc img {
    max-width: 100%;
    height: auto;
  }
  .ub-print-doc table {
    border-collapse: collapse;
    width: 100%;
  }
  .ub-print-doc th,
  .ub-print-doc td {
    border: 1px solid #ccc;
    padding: 0.25rem 0.4rem;
    text-align: left;
  }
  h1, h2, h3 { page-break-after: avoid; }
  p, li { orphans: 3; widows: 3; }
  @media print {
    body { padding: 0; }
    .ub-print-doc { max-width: none; }
  }
`;

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function printerFriendlyHtmlDocument(title: string, bodyInnerHtml: string): string {
  const safeTitle = escapeHtml(title);
  return `<!DOCTYPE html>
<html lang="en-NZ">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${safeTitle}</title>
  <style>${PRINT_STYLES}</style>
</head>
<body>
  <main class="ub-print-doc">${bodyInnerHtml}</main>
</body>
</html>`;
}

export function openPrintFriendlyDocument(title: string, bodyInnerHtml: string): void {
  const w = window.open("", "_blank", "noopener");
  if (!w) {
    return;
  }
  w.document.write(printerFriendlyHtmlDocument(title, bodyInnerHtml));
  w.document.close();
  w.focus();
  w.print();
}

export function downloadPrintFriendlyHtml(
  title: string,
  bodyInnerHtml: string,
  downloadBasename: string,
): void {
  const blob = new Blob([printerFriendlyHtmlDocument(title, bodyInnerHtml)], {
    type: "text/html;charset=utf-8",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  const base = downloadBasename.replace(/\.[^.]+$/i, "") || "document";
  a.download = `${base}-print.html`;
  a.click();
  URL.revokeObjectURL(url);
}
