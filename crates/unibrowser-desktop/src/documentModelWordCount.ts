import type { DocumentModelDto } from "./documentModelBridge";
import { textReadingStats } from "./textStats";

type BlockDto = {
  type: string;
  text?: string;
  inlines?: { text: string }[];
  items?: { inlines: { text: string }[] }[];
};

export function plainTextFromDocumentModel(model: DocumentModelDto): string {
  const chunks: string[] = [];
  for (const block of model.blocks as BlockDto[]) {
    switch (block.type) {
      case "paragraph":
      case "heading":
      case "blockquote":
        for (const span of block.inlines ?? []) {
          if (span.text) {
            chunks.push(span.text);
          }
        }
        break;
      case "code_block":
        if (block.text) {
          chunks.push(block.text);
        }
        break;
      case "bullet_list":
      case "ordered_list":
        for (const item of block.items ?? []) {
          for (const span of item.inlines ?? []) {
            if (span.text) {
              chunks.push(span.text);
            }
          }
        }
        break;
      default:
        break;
    }
  }
  return chunks.join(" ");
}

export function documentModelWordCount(model: DocumentModelDto | null | undefined): number | null {
  if (!model) {
    return null;
  }
  return textReadingStats(plainTextFromDocumentModel(model)).words;
}
