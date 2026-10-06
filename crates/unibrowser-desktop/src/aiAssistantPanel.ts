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
  panel_width?: number;
}

interface AiStatus {
  configured: boolean;
  panel_visible: boolean;
  can_see_tab: boolean;
  pending: { id: number; summary: string; preview: string }[];
}

interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

const DEFAULT_PANEL_WIDTH = 320;
const MIN_PANEL_WIDTH = 220;
const MAX_PANEL_WIDTH_RATIO = 0.4;

let panelEl: HTMLElement | null = null;
let workAreaEl: HTMLElement | null = null;
let chatMessages: ChatMessage[] = [];
let chatStreaming = false;
let chatListenersReady = false;

function configured(prefs: AiPrefs): boolean {
  return Boolean(prefs.base_url && prefs.base_url.trim().length > 0);
}

function viewerHost(): HTMLElement | null {
  return document.querySelector<HTMLElement>("#document-viewer-host");
}

function panelWidth(prefs: AiPrefs): number {
  const w = prefs.panel_width ?? DEFAULT_PANEL_WIDTH;
  return Math.max(MIN_PANEL_WIDTH, w);
}

function applyDockLayout(visible: boolean, widthPx: number) {
  if (!workAreaEl || !panelEl) {
    return;
  }
  workAreaEl.classList.toggle("ai-open", visible);
  document.documentElement.style.setProperty(
    "--ub-ai-sidebar-width",
    visible ? `${widthPx}px` : "0px",
  );
  window.dispatchEvent(new Event("unibrowser-shell-layout-changed"));
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
  const extras = host
    ? gatherProjectionExtras(host)
    : {
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

function updateHeader(prefs: AiPrefs, status: AiStatus) {
  const provider = panelEl?.querySelector(".ai-assistant-provider");
  if (!provider) {
    return;
  }
  if (!configured(prefs)) {
    provider.textContent = t("aiAssistant.notConfigured");
    return;
  }
  const model = prefs.model?.trim() || t("aiAssistant.defaultModel");
  const grants = prefs.grants.length
    ? t("aiAssistant.grantsSummary", { count: String(prefs.grants.length) })
    : t("aiAssistant.noGrants");
  provider.textContent = `${model} · ${grants}${
    status.can_see_tab ? "" : ` · ${t("aiAssistant.cannotSeeTab")}`
  }`;
}

async function refreshPanel() {
  if (!panelEl) {
    return;
  }
  const prefs = await invoke<AiPrefs>("get_ai_assistant_prefs");
  const status = await invoke<AiStatus>("ai_assistant_status");
  updateHeader(prefs, status);
  const body = panelEl.querySelector(".ai-assistant-body");
  if (!body) {
    return;
  }
  body.innerHTML = "";

  if (!configured(prefs)) {
    const hint = document.createElement("p");
    hint.className = "muted";
    hint.textContent = t("aiAssistant.configureHint");
    body.appendChild(hint);
    return;
  }

  if (prefs.grants.length > 0) {
    const grantsDetails = document.createElement("details");
    grantsDetails.className = "ai-assistant-grants-collapse";
    const grantsSummary = document.createElement("summary");
    grantsSummary.textContent = t("aiAssistant.grantsSummary", {
      count: String(prefs.grants.length),
    });
    grantsDetails.appendChild(grantsSummary);
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
    grantsDetails.appendChild(grantsTable);
    body.appendChild(grantsDetails);
  }

  const chatLog = document.createElement("div");
  chatLog.className = "ai-assistant-chat-log";
  for (const msg of chatMessages) {
    const row = document.createElement("div");
    row.className = `ai-assistant-chat-msg ai-assistant-chat-msg--${msg.role}`;
    const label = document.createElement("strong");
    label.textContent =
      msg.role === "user" ? t("aiAssistant.chatYou") : t("aiAssistant.chatAssistant");
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

  for (const p of status.pending) {
    const card = document.createElement("details");
    card.className = "ai-assistant-pending-card";
    card.open = true;
    const title = document.createElement("summary");
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
}

export async function setAiPanelVisible(visible: boolean) {
  const prefs = await invoke<AiPrefs>("get_ai_assistant_prefs");
  prefs.panel_visible = visible;
  prefs.panel_docked = true;
  await invoke("set_ai_assistant_prefs", { prefs });
  if (panelEl) {
    panelEl.classList.toggle("hidden", !visible);
  }
  applyDockLayout(visible, panelWidth(prefs));
  if (visible) {
    await refreshPanel();
  } else {
    const content = document.getElementById("content-host");
    content?.focus({ preventScroll: true });
  }
}

function bindResize() {
  const grip = document.getElementById("ai-assistant-resize");
  if (!grip || !panelEl) {
    return;
  }
  let startX = 0;
  let startW = 0;
  const onMove = async (e: PointerEvent) => {
    const delta = startX - e.clientX;
    const maxW = window.innerWidth * MAX_PANEL_WIDTH_RATIO;
    const next = Math.min(maxW, Math.max(MIN_PANEL_WIDTH, startW + delta));
    document.documentElement.style.setProperty("--ub-ai-sidebar-width", `${next}px`);
    window.dispatchEvent(new Event("unibrowser-shell-layout-changed"));
  };
  const onUp = async (e: PointerEvent) => {
    grip.releasePointerCapture(e.pointerId);
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    const w = panelEl?.getBoundingClientRect().width ?? DEFAULT_PANEL_WIDTH;
    const prefs = await invoke<AiPrefs>("get_ai_assistant_prefs");
    prefs.panel_width = w;
    prefs.panel_docked = true;
    await invoke("set_ai_assistant_prefs", { prefs });
  };
  grip.addEventListener("pointerdown", (e) => {
    startX = e.clientX;
    startW = panelEl?.getBoundingClientRect().width ?? DEFAULT_PANEL_WIDTH;
    grip.setPointerCapture(e.pointerId);
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  });
}

export function initAiAssistantPanel() {
  panelEl = document.getElementById("ai-assistant-panel");
  workAreaEl = document.getElementById("work-area");
  if (!panelEl) {
    return;
  }
  bindResize();
  panelEl.querySelector(".ai-assistant-close")?.addEventListener("click", () => {
    void setAiPanelVisible(false);
  });
  void Promise.all([
    invoke<AiPrefs>("get_ai_assistant_prefs"),
    invoke<string | null>("take_doc_capture_path"),
  ]).then(([prefs, docCapture]) => {
    if (!prefs.panel_docked) {
      prefs.panel_docked = true;
    }
    const visible = prefs.panel_visible && !docCapture;
    panelEl!.classList.toggle("hidden", !visible);
    applyDockLayout(visible, panelWidth(prefs));
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
