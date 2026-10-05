type PdfJs = typeof import("pdfjs-dist");
type ThreeModule = typeof import("three");

let pdfPromise: Promise<PdfJs> | null = null;
let threePromise: Promise<ThreeModule> | null = null;

export async function loadPdfJs(): Promise<PdfJs> {
  if (!pdfPromise) {
    pdfPromise = import("pdfjs-dist").then((pdfjsLib) => {
      pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
        "pdfjs-dist/build/pdf.worker.min.mjs",
        import.meta.url,
      ).toString();
      return pdfjsLib;
    });
  }
  return pdfPromise;
}

export async function loadThree(): Promise<ThreeModule> {
  if (!threePromise) {
    threePromise = import("three");
  }
  return threePromise;
}
