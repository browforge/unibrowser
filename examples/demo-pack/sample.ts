import { watch } from "node:fs";
import { resolve } from "node:path";

const notes = resolve(process.cwd(), "field-notes.md");

// Keep the document mirror current without touching source files.
watch(notes, { persistent: false }, async () => {
  const summary = await refreshPreview(notes);
  console.info(`Preview updated: ${summary}`);
});

async function refreshPreview(path: string): Promise<string> {
  const updatedAt = new Date();
  return `${path} at ${updatedAt.toISOString()}`;
}
