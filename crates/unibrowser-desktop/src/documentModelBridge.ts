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

export type DocumentSessionPayload = {
  model: DocumentModelDto;
  source_map: SourceMapDto;
  original_base64: string;
  dirty_block_ids?: string[];
  touched_block_ids?: string[];
};

function sessionRequest(session: DocumentSessionPayload) {
  return {
    model: session.model,
    source_map: session.source_map,
    original_base64: session.original_base64,
    dirty_block_ids: session.dirty_block_ids ?? [],
    touched_block_ids: session.touched_block_ids ?? [],
  };
}

export async function sourceProjectionDocument(session: DocumentSessionPayload): Promise<string> {
  const res = await invoke<{ content: string }>("document_source_projection", sessionRequest(session));
  return res.content;
}

export async function mergeSourceDocument(
  session: DocumentSessionPayload,
  newContent: string,
): Promise<{
  clean: boolean;
  conflict_reason?: string | null;
  model: DocumentModelDto;
  source_map: SourceMapDto;
  original_base64: string;
  dirty_block_ids: string[];
  touched_block_ids: string[];
  projectionContent?: string;
}> {
  const merged = await invoke<{
    clean: boolean;
    conflict_reason?: string | null;
    model: DocumentModelDto;
    source_map: SourceMapDto;
    original_base64: string;
    dirty_block_ids: string[];
    touched_block_ids: string[];
  }>("document_merge_source", {
    ...sessionRequest(session),
    new_content: newContent,
  } as Record<string, unknown>);
  const projection = merged.clean
    ? await sourceProjectionDocument({
        model: merged.model,
        source_map: merged.source_map,
        original_base64: merged.original_base64,
        dirty_block_ids: merged.dirty_block_ids,
        touched_block_ids: merged.touched_block_ids,
      })
    : undefined;
  return { ...merged, projectionContent: projection };
}

export async function appendDocumentHistoryOp(
  path: string,
  op: { op_type: string; [key: string]: unknown },
): Promise<void> {
  await invoke("document_history_append", { path, op });
}

export async function listDocumentHistory(path: string): Promise<
  {
    seq: number;
    op_type: string;
    timestamp: string;
    plugin_id: string;
    payload?: Record<string, unknown>;
  }[]
> {
  return invoke("document_history_list", { path });
}

export async function jumpDocumentHistory(path: string, seq: number): Promise<string> {
  return invoke("document_history_jump_to_seq", { path, seq });
}

export async function trimDocumentHistoryBefore(path: string, keepFromSeq: number): Promise<void> {
  await invoke("document_history_trim_before", { path, keepFromSeq });
}

export async function squashDocumentHistoryRange(
  path: string,
  fromSeq: number,
  toSeq: number,
): Promise<void> {
  await invoke("document_history_squash_range", { path, fromSeq, toSeq });
}

export async function saveDocxFromModel(path: string, model: DocumentModelDto): Promise<unknown> {
  return invoke("document_save_docx_from_model", { path, model });
}

export async function appendDocumentHistorySnapshot(
  path: string,
  session: DocumentSessionPayload,
): Promise<void> {
  await invoke("document_history_append_snapshot", {
    path,
    model: session.model,
    sourceMap: session.source_map,
    originalBase64: session.original_base64,
  });
}

export async function saveMarkdownVisualAsDocx(markdownPath: string): Promise<string> {
  const { getDocumentModelSession } = await import("./documentEditorHost");
  const session = getDocumentModelSession();
  if (!session) {
    throw new Error("No document model session");
  }
  const saved = await invoke<string>("document_save_markdown_as_docx", {
    markdownPath,
    model: session.model,
  });
  const { finalizeVisualDocxAfterSave } = await import("./ui/textDocumentCard");
  await finalizeVisualDocxAfterSave(saved);
  return saved;
}
