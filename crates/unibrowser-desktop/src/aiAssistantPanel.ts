import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { gatherProjectionExtras } from "./documentViewer";
import { showErrorToast } from "./toast";
import { t } from "./i18n";

interface AiPrefs {
  base_url?: string | null;
  model?: string | null;
  api_key?: string | null;
  grants: { permission: string; scope_kind: string; scope_value?: string | null }[];
  panel_visible: boolean;
  panel_x: number;
  panel_y: number;
  panel_docked: boolean;
}

interface AiStatus {
  configured: boolean;
  panel_visible: boolean;
  can_see_tab: boolean;
  pending: { id: number; summary: string; preview: string }[];
}

interface AiTool {
  command_id: string;
  title: string;
  enabled: boolean;
  reason: string | null;
}

interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

let panelEl: HTMLElement | null = null;
let dragOffsetX = 0;
let dragOffsetY = 0;
let chatMessages: ChatMessage[] = [];
let chatStreaming = false;
let chatListenersReady = false;

function configured(prefs: AiPrefs): boolean {
  return Boolean(prefs.base_url && prefs.base_url.trim().length > 0);
}

function viewerHost(): HTMLElement | null {
  return document.querySelector<HTMLElement>("#document-viewer-host");
}

async function ensureChatListeners() {
  if (chatListenersReady) {
    return;
  }
  chatListenersReady = true;
  await listen<{ delta: string }>("ai-assistant-chat-delta", (ev) => {
    const streamEl = panelEl?.querySelector(".ai-assistant-chat-stream");
    if (streamEl) {
      streamEl.textContent = (streamEl.textContent ?? "") + ev.payload.delta;
    }
  });
  await listen<{ content: string }>("ai-assistant-chat-done", (ev) => {
    chatStreaming = false;
    chatMessages.push({ role: "assistant", content: ev.payload.content });
    void refreshPanel();
  });
  await listen<{ message: string }>("ai-assistant-chat-error", (ev) => {
    chatStreaming = false;
    showErrorToast(ev.payload.message);
    void refreshPanel();
  });
}

async function sendChat(userText: string) {
  if (!userText.trim() || chatStreaming) {
    return;
  }
  await ensureChatListeners();
  chatMessages.push({ role: "user", content: userText.trim() });
  chatStreaming = true;
  const host = viewerHost();
  const extras = host ? gatherProjectionExtras(host) : {
    web_dom_text: null,
    ocr_text: null,
    epub_chapter_text: null,
    pdf_page_text: null,
    docx_preview_text: null,
  };
  const status = await invoke<AiStatus>("ai_assistant_status");
  try {
    await invoke("ai_assistant_chat", {
      req: {
        messages: chatMessages.map((m) => ({ role: m.role, content: m.content })),
        extras,
        include_tab_context: status.can_see_tab,
      },
    });
  } catch (e) {
    chatStreaming = false;
    showErrorToast(e);
    void refreshPanel();
  }
}

async function refreshPanel() {
  if (!panelEl) {
    return;
  }
  const prefs = await invoke<AiPrefs>("get_ai_assistant_prefs");
  const status = await invoke<AiStatus>("ai_assistant_status");
  const body = panelEl.querySelector(".ai-assistant-body");
  if (!body) {
    return;
  }
  body.innerHTML = "";
  const indicator = document.createElement("p");
  indicator.className = "ai-assistant-visibility";
  indicator.textContent = status.can_see_tab
    ? t("aiAssistant.canSeeTab")
    : t("aiAssistant.cannotSeeTab");
  body.appendChild(indicator);

  if (!configured(prefs)) {
    const hint = document.createElement("p");
    hint.className = "muted";
    hint.textContent = t("aiAssistant.configureHint");
    body.appendChild(hint);
    return;
  }

  if (prefs.grants.length > 0) {
    const grantsWrap = document.createElement("div");
    grantsWrap.className = "ai-assistant-grants";
    const grantsTable = document.createElement("div");
    grantsTable.className = "ai-grants-table";
    for (const g of prefs.grants) {
      const row = document.createElement("div");
      row.className = "ai-grants-row";
      row.textContent = `${g.permission} · ${g.scope_kind}${
        g.scope_value ? ` (${g.scope_value})` : ""
      }`;
      grantsTable.appendChild(row);
    }
    grantsWrap.appendChild(grantsTable);
    body.appendChild(grantsWrap);
  }

  const chatLog = document.createElement("div");
  chatLog.className = "ai-assistant-chat-log";
  for (const msg of chatMessages) {
    const row = document.createElement("div");
    row.className = `ai-assistant-chat-msg ai-assistant-chat-msg--${msg.role}`;
    const label = document.createElement("strong");
    label.textContent = msg.role === "user" ? t("aiAssistant.chatYou") : t("aiAssistant.chatAssistant");
    row.appendChild(label);
    const pre = document.createElement("pre");
    pre.textContent = msg.content;
    row.appendChild(pre);
    chatLog.appendChild(row);
  }
  if (chatStreaming) {
    const streamRow = document.createElement("div");
    streamRow.className = "ai-assistant-chat-msg ai-assistant-chat-msg--assistant";
    const streamPre = document.createElement("pre");
    streamPre.className = "ai-assistant-chat-stream";
    streamRow.appendChild(streamPre);
    chatLog.appendChild(streamRow);
  }
  body.appendChild(chatLog);

  const chatForm = document.createElement("div");
  chatForm.className = "ai-assistant-chat-form";
  const input = document.createElement("textarea");
  input.rows = 2;
  input.placeholder = t("aiAssistant.chatPlaceholder");
  input.disabled = chatStreaming;
  const sendBtn = document.createElement("button");
  sendBtn.type = "button";
  sendBtn.textContent = chatStreaming ? t("aiAssistant.chatStreaming") : t("aiAssistant.chatSend");
  sendBtn.disabled = chatStreaming;
  sendBtn.addEventListener("click", () => {
    const text = input.value;
    input.value = "";
    void sendChat(text);
  });
  chatForm.appendChild(input);
  chatForm.appendChild(sendBtn);
  body.appendChild(chatForm);

  const readBtn = document.createElement("button");
  readBtn.type = "button";
  readBtn.textContent = t("aiAssistant.readTab");
  readBtn.addEventListener("click", async () => {
    await invoke("ai_assistant_invoke");
    const host = viewerHost();
    const extras = host
      ? gatherProjectionExtras(host)
      : {
          web_dom_text: null,
          ocr_text: null,
          epub_chapter_text: null,
          pdf_page_text: null,
          docx_preview_text: null,
        };
    const text = await invoke<string>("ai_assistant_read_tab_text", { extras });
    const pre = document.createElement("pre");
    pre.className = "ai-assistant-projection";
    pre.textContent = text;
    body.appendChild(pre);
  });
  body.appendChild(readBtn);

  for (const p of status.pending) {
    const card = document.createElement("div");
    card.className = "ai-assistant-pending-card";
    const title = document.createElement("strong");
    title.textContent = p.summary;
    card.appendChild(title);
    const preview = document.createElement("pre");
    preview.textContent = p.preview;
    card.appendChild(preview);
    const approve = document.createElement("button");
    approve.type = "button";
    approve.textContent = t("aiAssistant.approve");
    approve.addEventListener("click", async () => {
      await invoke("ai_assistant_approve_pending", { id: p.id });
      await refreshPanel();
    });
    card.appendChild(approve);
    body.appendChild(card);
  }

  const toolsHead = document.createElement("h4");
  toolsHead.textContent = t("aiAssistant.tools");
  body.appendChild(toolsHead);
  const tools = await invoke<AiTool[]>("ai_assistant_tools");
  const ul = document.createElement("ul");
  ul.className = "ai-assistant-tools";
  for (const tool of tools.slice(0, 40)) {
    const li = document.createElement("li");
    li.textContent = tool.enabled ? tool.title : `${tool.title} (${tool.reason ?? "disabled"})`;
    if (!tool.enabled) {
      li.classList.add("muted");
    }
    ul.appendChild(li);
  }
  body.appendChild(ul);
}

export async function setAiPanelVisible(visible: boolean) {
  const prefs = await invoke<AiPrefs>("get_ai_assistant_prefs");
  prefs.panel_visible = visible;
  await invoke("set_ai_assistant_prefs", { prefs });
  if (panelEl) {
    panelEl.classList.toggle("hidden", !visible);
  }
  if (visible) {
    await refreshPanel();
  }
}

export function initAiAssistantPanel() {
  panelEl = document.getElementById("ai-assistant-panel");
  if (!panelEl) {
    return;
  }
  const handle = panelEl.querySelector(".ai-assistant-drag-handle") as HTMLElement | null;
  handle?.addEventListener("pointerdown", (e) => {
    if (!panelEl) {
      return;
    }
    dragOffsetX = e.clientX - panelEl.offsetLeft;
    dragOffsetY = e.clientY - panelEl.offsetTop;
    handle.setPointerCapture(e.pointerId);
  });
  handle?.addEventListener("pointermove", (e) => {
    if (!panelEl || !handle.hasPointerCapture(e.pointerId)) {
      return;
    }
    panelEl.style.left = `${e.clientX - dragOffsetX}px`;
    panelEl.style.top = `${e.clientY - dragOffsetY}px`;
  });
  handle?.addEventListener("pointerup", async (e) => {
    if (!panelEl) {
      return;
    }
    handle?.releasePointerCapture(e.pointerId);
    const prefs = await invoke<AiPrefs>("get_ai_assistant_prefs");
    prefs.panel_x = panelEl.offsetLeft;
    prefs.panel_y = panelEl.offsetTop;
    await invoke("set_ai_assistant_prefs", { prefs });
  });
  panelEl.querySelector(".ai-assistant-close")?.addEventListener("click", () => {
    void setAiPanelVisible(false);
  });
  void Promise.all([
    invoke<AiPrefs>("get_ai_assistant_prefs"),
    invoke<string | null>("take_doc_capture_path"),
  ]).then(([prefs, docCapture]) => {
    panelEl!.style.left = `${prefs.panel_x || 48}px`;
    panelEl!.style.top = `${prefs.panel_y || 96}px`;
    const visible = prefs.panel_visible && !docCapture;
    panelEl!.classList.toggle("hidden", !visible);
    if (visible) {
      void refreshPanel();
    }
  });
}

export async function refreshAiAssistantChrome() {
  const status = await invoke<AiStatus>("ai_assistant_status");
  const el = document.getElementById("status-ai-visibility");
  if (el) {
    el.textContent = status.can_see_tab ? t("aiAssistant.statusSeeing") : "";
    el.classList.toggle("hidden", !status.can_see_tab);
  }
}
