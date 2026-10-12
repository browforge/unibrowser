const handleKey = (path: string) => `ub-file-handle:${path}`;

type FileHandleLike = FileSystemFileHandle;

export async function pickOpenTextFile(): Promise<{ path: string; content: string } | null> {
  const accept: Record<string, string[]> = {
    "text/plain": [".txt", ".md", ".markdown", ".json", ".ts", ".rs", ".css", ".html"],
  };
  if ("showOpenFilePicker" in window) {
    try {
      const handles = await (
        window as unknown as {
          showOpenFilePicker: (o: object) => Promise<FileHandleLike[]>;
        }
      ).showOpenFilePicker({
        multiple: false,
        types: [{ description: "Text", accept }],
      });
      const handle = handles[0];
      const file = await handle.getFile();
      const path = file.name;
      localStorage.setItem(handleKey(path), JSON.stringify({ kind: "fsa", name: path }));
      (window as unknown as { __ubFileHandles?: Map<string, FileHandleLike> }).__ubFileHandles ??=
        new Map();
      (window as unknown as { __ubFileHandles: Map<string, FileHandleLike> }).__ubFileHandles.set(
        path,
        handle,
      );
      const content = await file.text();
      return { path, content };
    } catch {
      return null;
    }
  }
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".txt,.md,.markdown,.json,.ts,.rs,.css,.html";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      resolve({ path: file.name, content: await file.text() });
    };
    input.click();
  });
}

export async function saveTextFile(path: string, content: string): Promise<void> {
  const handles = (window as unknown as { __ubFileHandles?: Map<string, FileHandleLike> })
    .__ubFileHandles;
  const handle = handles?.get(path);
  if (handle && "createWritable" in handle) {
    const writable = await handle.createWritable();
    await writable.write(content);
    await writable.close();
    return;
  }
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = path.split("/").pop() ?? "download.txt";
  a.click();
  URL.revokeObjectURL(url);
}

export async function fetchDemoPack(relativePath: string): Promise<string> {
  const base = import.meta.env.BASE_URL ?? "/";
  const url = `${base}demo-pack/${relativePath.replace(/^demo-pack\//, "")}`;
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`Demo file not found: ${relativePath}`);
  }
  return res.text();
}
