import { invoke } from "@tauri-apps/api/core";

export interface DocumentModelDto {
  schema_id: string;
  schema_version: string;
  metadata: {
    title?: string | null;
    host_format?: string;
    serializer_plugin_id?: string;
  };
  blocks: unknown[];
}

export interface SourceMapDto {
  entries: {
    block_id: string;
    byte_start: number;
    byte_end: number;
    digest: string;
  }[];
}

export interface DocumentParseResponse {
  model: DocumentModelDto;
  source_map: SourceMapDto;
  original_base64: string;
}

export async function parseMarkdownDocument(content: string): Promise<DocumentParseResponse> {
  return invoke<DocumentParseResponse>("document_parse_markdown", { content });
}

export async function serializeMarkdownDocument(request: {
  model: DocumentModelDto;
  source_map: SourceMapDto;
  original_base64: string;
  dirty_block_ids: string[];
  touched_block_ids: string[];
}): Promise<string> {
  const result = await invoke<unknown>("document_serialize_markdown", request);
  if (typeof result === "string") {
    return result;
  }
  if (result && typeof result === "object" && "content" in result) {
    return String((result as { content: string }).content);
  }
  return "";
}
