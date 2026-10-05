import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";
import { textEditor, type TextDocumentDto } from "./textEditor";

interface TabDto {
  id: number;
  label: string;
  location_kind: string;
  is_directory: boolean;
  is_http: boolean;
}

interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

interface TabContentDto {
  tabs: TabDto[];
  active_index: number;
  omnibox: string;
  mods_loaded: string;
  status: string;
  directory_entries: DirEntry[] | null;
  document_status: string | null;
  text_document: TextDocumentDto | null;
}

interface ToolbarCommandDto {
  id: string;
  title: string;
  enabled: boolean;
  reason: string | null;
}

interface PaletteEntryDto {
  command_id: string;
  title: string;
  enabled: boolean;
  reason: string | null;
}

const omnibox = document.getElementById("omnibox") as HTMLInputElement;
const tabStrip = document.getElementById("tab-strip")!;
const filesPanel = document.getElementById("files-panel")!;
const docPanel = document.getElementById("doc-panel")!;
const toolbar = document.getElementById("toolbar")!;
const statusLine = document.getElementById("status-line")!;
const modsLine = document.getElementById("mods-line")!;
const palette = document.getElementById("palette")!;
const paletteQuery = document.getElementById("palette-query") as HTMLInputElement;
const paletteResults = document.getElementById("palette-results")!;
const contentHost = document.getElementById("content-host")!;
const textEditorHost = document.getElementById("text-editor-host")!;

const PALETTE_TOGGLE = "unibrowser.palette.toggle";

let paletteOpen = false;
let paletteChordLabel = "Ctrl+Shift+P";

async function syncContentBounds() {
  const rect = contentHost.getBoundingClientRect();
  await invoke("set_content_bounds", {
    x: rect.left,
    y: rect.top,
    width: rect.width,
    height: rect.height,
  });
}

function renderTabs(data: TabContentDto) {
  tabStrip.innerHTML = "";
  data.tabs.forEach((tab, index) => {
    const btn = document.createElement("button");
    btn.className = "tab" + (index === data.active_index ? " active" : "");
    btn.textContent = tab.label;
    btn.title = tab.location_kind;
    btn.onclick = () => selectTab(index);
    tabStrip.appendChild(btn);
  });
  const plus = document.createElement("button");
  plus.className = "tab new-tab";
  plus.textContent = "+";
  plus.onclick = () => invoke<TabContentDto>("new_tab").then(applyState);
  tabStrip.appendChild(plus);
  omnibox.value = data.omnibox;
  modsLine.textContent = `Mods: ${data.mods_loaded}`;
}

function renderFiles(entries: DirEntry[] | null) {
  filesPanel.innerHTML = "";
  if (!entries) {
    filesPanel.classList.add("hidden");
    return;
  }
  filesPanel.classList.remove("hidden");
  const table = document.createElement("table");
  entries.forEach((entry) => {
    const row = document.createElement("tr");
    const cell = document.createElement("td");
    const link = document.createElement("button");
    link.className = "file-link";
    link.textContent = (entry.is_dir ? "[dir] " : "[file] ") + entry.name;
    link.onclick = () =>
      invoke<TabContentDto>("open_path", { path: entry.path }).then(applyState);
    cell.appendChild(link);
    row.appendChild(cell);
    table.appendChild(row);
  });
  filesPanel.appendChild(table);
}

function renderDoc(status: string | null, hasTextEditor: boolean) {
  docPanel.innerHTML = "";
  if (!status || hasTextEditor) {
    docPanel.classList.add("hidden");
    return;
  }
  docPanel.classList.remove("hidden");
  docPanel.textContent = status;
}

async function refreshToolbar() {
  try {
    const cmds = await invoke<ToolbarCommandDto[]>("get_toolbar_commands");
    toolbar.innerHTML = "";
    cmds.forEach((cmd) => {
      const btn = document.createElement("button");
      btn.textContent = cmd.title;
      btn.disabled = !cmd.enabled;
      btn.title = cmd.reason ?? "";
      btn.className = cmd.enabled ? "" : "disabled";
      btn.onclick = () =>
        textEditor.flush().then(() =>
          invoke<string>("execute_command", { commandId: cmd.id }).then((msg) => {
            statusLine.textContent = msg;
            refreshToolbar();
            refreshState();
          }),
        );
      toolbar.appendChild(btn);
    });
  } catch {
    toolbar.innerHTML =
      "<span class='muted'>Open a document file for format commands</span>";
  }
}

async function applyState(data: TabContentDto) {
  const active = data.tabs[data.active_index];
  const showText = Boolean(data.text_document) && !active?.is_http;
  renderTabs(data);
  if (showText) {
    filesPanel.classList.add("hidden");
  } else {
    renderFiles(data.directory_entries);
  }
  renderDoc(data.document_status, showText);
  if (showText && data.text_document) {
    await textEditor.apply(data.text_document, textEditorHost);
  } else {
    textEditor.hide(textEditorHost);
  }
  statusLine.textContent = data.status;
  await refreshToolbar();
  await syncContentBounds();
  window.requestAnimationFrame(() => syncContentBounds());
}

async function refreshState() {
  const data = await invoke<TabContentDto>("get_browser_state");
  await applyState(data);
}

async function selectTab(index: number) {
  await textEditor.flush();
  await invoke("select_tab", { index });
  await refreshState();
}

omnibox.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    invoke<TabContentDto>("navigate_omnibox", { input: omnibox.value }).then(applyState);
  }
});

document.getElementById("btn-back")!.onclick = () =>
  invoke<TabContentDto>("go_back").then(applyState);
document.getElementById("btn-forward")!.onclick = () =>
  invoke<TabContentDto>("go_forward").then(applyState);
document.getElementById("btn-reload")!.onclick = () =>
  invoke<TabContentDto>("reload_tab").then(applyState);

document.getElementById("btn-demo-download")!.onclick = () =>
  invoke<TabContentDto>("handle_download_url", {
    url: "https://example.com/",
  }).then(applyState);

function togglePalette() {
  paletteOpen = !paletteOpen;
  palette.classList.toggle("hidden", !paletteOpen);
  if (paletteOpen) {
    paletteQuery.placeholder = `Search commands (${paletteChordLabel})`;
    paletteQuery.focus();
  }
}

function chordFromKeyboardEvent(e: KeyboardEvent) {
  let key = e.key;
  if (key.length === 1) {
    key = key.toLowerCase();
  } else {
    key = key.toLowerCase();
  }
  if (key === "+") {
    key = "=";
  }
  return {
    ctrl: e.ctrlKey,
    alt: e.altKey,
    shift: e.shiftKey,
    meta: e.metaKey,
    key,
  };
}

async function runKeybindingCommand(commandId: string) {
  if (commandId === PALETTE_TOGGLE) {
    togglePalette();
    return;
  }
  await textEditor.flush();
  const msg = await invoke<string>("execute_command", { commandId });
  statusLine.textContent = String(msg);
  await refreshToolbar();
  await refreshState();
}

document.addEventListener(
  "keydown",
  async (e) => {
  if (e.repeat) {
    return;
  }
  const chord = chordFromKeyboardEvent(e);
  const commandId = await invoke<string | null>("dispatch_key_chord", { chord });
  if (!commandId) {
    return;
  }
  e.preventDefault();
  await runKeybindingCommand(commandId);
},
  { capture: true },
);

async function loadKeybindingHints() {
  try {
    const bindings = await invoke<{ command_id: string; chord: string }[]>(
      "get_keybindings",
    );
    const palette = bindings.find((b) => b.command_id === PALETTE_TOGGLE);
    if (palette) {
      paletteChordLabel = palette.chord;
      paletteQuery.placeholder = `Search commands (${paletteChordLabel})`;
    }
  } catch {
    /* defaults */
  }
}

paletteQuery.addEventListener("input", async () => {
  const entries = await invoke<PaletteEntryDto[]>("palette_search", {
    query: paletteQuery.value,
  });
  paletteResults.innerHTML = "";
  entries.forEach((entry) => {
    const row = document.createElement("button");
    row.className = entry.enabled ? "palette-item" : "palette-item disabled";
    row.textContent = entry.title + (entry.enabled ? "" : " (greyed)");
    row.disabled = !entry.enabled;
    row.onclick = () => {
      invoke<string>("execute_command", { commandId: entry.command_id }).then((msg) => {
        statusLine.textContent = String(msg);
        togglePalette();
        refreshToolbar();
        refreshState();
      });
    };
    paletteResults.appendChild(row);
  });
});

window.addEventListener("resize", () => syncContentBounds());

listen<string>("unibrowser-download", (event) => {
  invoke<TabContentDto>("handle_download_url", { url: event.payload }).then(applyState);
});

loadKeybindingHints();
refreshState();
