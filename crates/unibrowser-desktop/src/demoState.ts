import { invoke } from "@tauri-apps/api/core";
import { setAiDemoComposerSeed, setAiPanelVisible } from "./aiAssistantPanel";
import { applyThemeFromDemoState, applyThemeOverride } from "./theme";
import { t } from "./i18n";
import { showActionToast } from "./toast";
import { isToolbarOnlyWindow } from "./toolbarOnly";

/** Tokens that need editor/compact toolbar ready (also deferred in doc-capture). */
export const DEFERRED_DEMO_TOKEN_RE =
  /^(content-kind-|cm-image-tab-pin$|cm-image-tab$|txt-block-picker-open$|txt-apply-h2$|txt-apply-h1$|txt-mod-toolbar-visible$|doc-visual-md-docx$)/;

export function bootstrapDemoStateTokens(raw: string): string {
  return raw
    .split(/[,;]+/)
    .map((t) => t.trim())
    .filter(Boolean)
    .filter((t) => !DEFERRED_DEMO_TOKEN_RE.test(t.toLowerCase()))
    .join(",");
}

/** Apply text-toolbar demo tokens after command manager / compact row are mounted. */
export async function applyDeferredTextToolbarDemoTokens(raw: string): Promise<void> {
  const demoTokens = raw.toLowerCase();
  if (
    !demoTokens.includes("txt-apply-h2") &&
    !demoTokens.includes("txt-apply-h1") &&
    !demoTokens.includes("txt-block-picker-open") &&
    !demoTokens.includes("txt-mod-toolbar-visible")
  ) {
    return;
  }
  if (demoTokens.includes("txt-mod-toolbar-visible")) {
    await applyDemoState("txt-mod-toolbar-visible");
  }
  for (const token of ["txt-apply-h2", "txt-apply-h1"] as const) {
    if (demoTokens.includes(token)) {
      await applyDemoState(token);
    }
  }
  if (demoTokens.includes("txt-apply-h2")) {
    const { textEditor } = await import("./textEditor");
    textEditor.setCursorLine(3);
  }
  if (demoTokens.includes("txt-apply-h1")) {
    const { textEditor } = await import("./textEditor");
    textEditor.setCursorLine(1);
  }
  if (demoTokens.includes("txt-block-picker-open")) {
    for (let attempt = 0; attempt < 40; attempt += 1) {
      await applyDemoState("txt-block-picker-open");
      const menu = document.querySelector(
        "#text-compact-toolbar-host .text-compact-block-menu:not(.hidden)",
      );
      if (menu) {
        break;
      }
      await new Promise((resolve) => window.setTimeout(resolve, 50));
    }
  }
}

/** Hidden preset for docs screenshots (`UNIBROWSER_DEMO_STATE`, comma-separated). */
export async function applyDemoState(raw: string) {
  const tokens = raw
    .split(/[,;]+/)
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean);
  const demoRaw = raw.toLowerCase();
  const waitForElement = async <T extends Element>(selector: string): Promise<T | null> => {
    for (let attempt = 0; attempt < 80; attempt += 1) {
      const element = document.querySelector<T>(selector);
      if (element) {
        return element;
      }
      await new Promise((resolve) => window.setTimeout(resolve, 50));
    }
    return null;
  };

  for (const token of tokens) {
    switch (token) {
      case "document-mockup":
      case "document-mockup-problems": {
        const { showProblemsBottomPanel } = await import("./problemsPanel");
        await invoke("seed_document_mockup_problems_cmd").catch(() => undefined);
        await showProblemsBottomPanel();
        break;
      }
      case "cm-text-pin": {
        await invoke("select_command_manager_tab", { tabId: "text" });
        await invoke("pin_command_manager_context");
        window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
        break;
      }
      case "txt-block-picker-open": {
        const { openBlockStyleMenuForCapture } = await import("./textCompactToolbar");
        for (let attempt = 0; attempt < 80; attempt += 1) {
          const host = document.getElementById("text-compact-toolbar-host");
          const trigger = host?.querySelector(".text-compact-block-trigger");
          if (host && trigger) {
            openBlockStyleMenuForCapture(host);
            break;
          }
          await new Promise((resolve) => window.setTimeout(resolve, 50));
        }
        break;
      }
      case "txt-apply-h2": {
        const { textEditor } = await import("./textEditor");
        for (let attempt = 0; attempt < 80; attempt += 1) {
          const positioned = await textEditor.setCursorLine(3);
          if (!positioned) {
            await new Promise((resolve) => window.setTimeout(resolve, 50));
            continue;
          }
          const applied = await textEditor.applySemanticBlockStyle(
            "unibrowser.semantic.block.heading_2",
          );
          if (applied) {
            break;
          }
          await new Promise((resolve) => window.setTimeout(resolve, 50));
        }
        {
          const { refreshCompactBlockPickerLabel } = await import("./textCompactToolbar");
          refreshCompactBlockPickerLabel();
        }
        break;
      }
      case "txt-apply-h1": {
        document.body.dataset.ubBlockStyleCommandId =
          "unibrowser.semantic.block.heading_1";
        document.body.dataset.ubBlockStyleLabel = "Heading 1";
        const { textEditor } = await import("./textEditor");
        for (let attempt = 0; attempt < 80; attempt += 1) {
          const applied = await textEditor.applySemanticBlockStyle(
            "unibrowser.semantic.block.heading_1",
          );
          if (applied) {
            break;
          }
          await new Promise((resolve) => window.setTimeout(resolve, 50));
        }
        {
          const { refreshCompactBlockPickerLabel } = await import("./textCompactToolbar");
          refreshCompactBlockPickerLabel();
        }
        break;
      }
      case "doc-mode-wysiwyg": {
        const { setDocumentEditorMode } = await import("./documentEditorModes");
        setDocumentEditorMode("wysiwyg");
        break;
      }
      case "doc-mode-source": {
        const { setDocumentEditorMode } = await import("./documentEditorModes");
        setDocumentEditorMode("source");
        break;
      }
      case "doc-mode-split": {
        const { setDocumentEditorMode } = await import("./documentEditorModes");
        setDocumentEditorMode("split");
        break;
      }
      case "doc-mode-reading": {
        const { setDocumentEditorMode } = await import("./documentEditorModes");
        setDocumentEditorMode("reading");
        break;
      }
      case "doc-merge-banner": {
        const { setDocumentEditorMode } = await import("./documentEditorModes");
        setDocumentEditorMode("split", false);
        break;
      }
      case "doc-opaque-card": {
        await invoke("open_path", { path: "demo-pack/sample-opaque.md" });
        const { setDocumentEditorMode } = await import("./documentEditorModes");
        setDocumentEditorMode("wysiwyg", false);
        const { syncTextDocumentCard } = await import("./ui/textDocumentCard");
        syncTextDocumentCard(true, "sample-opaque.md");
        break;
      }
      case "doc-history-panel": {
        await invoke("open_path", { path: "demo-pack/sample.md" }).catch(() => undefined);
        const { setDocumentEditorMode } = await import("./documentEditorModes");
        setDocumentEditorMode("source", false);
        const { syncTextDocumentCard } = await import("./ui/textDocumentCard");
        syncTextDocumentCard(true, "sample.md");
        await new Promise((r) => window.setTimeout(r, 400));
        const docPath = document.body.dataset.activeDocumentPath ?? "demo-pack/sample.md";
        await invoke("document_history_append", {
          path: docPath,
          op: {
            op_type: "base-text.block.set_style",
            block_id: "b2",
            style: "heading_2",
            emulation_note: null,
          },
        }).catch(() => undefined);
        await invoke("document_history_append", {
          path: docPath,
          op: {
            op_type: "base-text.text.replace",
            block_id: "b2",
            from: 0,
            to: 4,
            text: "Demo",
            marks: [],
          },
        }).catch(() => undefined);
        const { showHistoryPanelTab, refreshHistoryPanel } = await import("./historyPanel");
        showHistoryPanelTab();
        await refreshHistoryPanel(docPath);
        await new Promise((r) => window.setTimeout(r, 250));
        const { dismissDocCaptureChrome } = await import("./docCaptureDismiss");
        dismissDocCaptureChrome();
        break;
      }
      case "doc-visual-docx": {
        await invoke("open_path", { path: "demo-pack/sample-rich.docx" });
        break;
      }
      case "doc-visual-md-docx": {
        const mdPath = "demo-pack/sample.md";
        const { isWebPlatform } = await import("./platform/isWeb");
        const markdown =
          "# Demo markdown\n\n> Render 1 block quotes use a pale blue panel.\n\n**Bold** and _italic_\n";
        if (isWebPlatform()) {
          await invoke("open_text_with_content", { path: mdPath, content: markdown });
        } else {
          await invoke("open_path", { path: "examples/demo-pack/sample.md" }).catch(() => undefined);
          const editorHost = document.getElementById("text-editor-host");
          if (editorHost) {
            const { syncDocumentEditorForTextTab } = await import("./documentEditorHost");
            await syncDocumentEditorForTextTab(editorHost, mdPath, markdown);
          }
          const { setDocumentEditorMode } = await import("./documentEditorModes");
          setDocumentEditorMode("wysiwyg", false);
          const { syncTextDocumentCard } = await import("./ui/textDocumentCard");
          syncTextDocumentCard(true, "sample.md");
        }
        const { parseMarkdownDocument } = await import("./documentModelBridge");
        const { seedDocumentModelSessionForDemo } = await import("./documentEditorHost");
        seedDocumentModelSessionForDemo(await parseMarkdownDocument(markdown));
        const { saveMarkdownVisualAsDocx } = await import("./documentModelBridge");
        const saved = await saveMarkdownVisualAsDocx(mdPath);
        const opened = await invoke<{ tabs: unknown[] }>("open_path", { path: saved });
        const apply = (
          window as unknown as {
            __unibrowserApplyState?: (data: unknown) => Promise<void>;
          }
        ).__unibrowserApplyState;
        if (apply) {
          await apply(opened);
          const { syncTextDocumentCard } = await import("./ui/textDocumentCard");
          syncTextDocumentCard(false);
        } else {
          const refresh = (
            window as unknown as { __unibrowserRefreshState?: () => Promise<void> }
          ).__unibrowserRefreshState;
          if (refresh) {
            await refresh();
          }
        }
        break;
      }
      case "text-card-render1": {
        const { dismissDocCaptureChrome } = await import("./docCaptureDismiss");
        dismissDocCaptureChrome();
        const { setRailMode } = await import("./ui/activityRail");
        setRailMode("collapsed");
        break;
      }
      case "txt-mod-toolbar-visible":
        document.documentElement.dataset.ubModToolbarDocked = "1";
        window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
        break;
      case "cmd-proving-detached": {
        if (!isToolbarOnlyWindow()) {
          await invoke("detach_floating_toolbar_window", {
            barId: "cmd-proving.detached-markdown",
          });
        }
        break;
      }
      case "problems":
      case "problems-open": {
        const { showProblemsBottomPanel } = await import("./problemsPanel");
        await invoke("proofing_check", {
          text: "teh quick brown fox",
          surface: "plaintext",
          language: "en-NZ",
        }).catch(() => undefined);
        await showProblemsBottomPanel();
        break;
      }
      case "ai":
      case "ai-sidebar":
      case "ai-open":
        await invoke("set_ai_assistant_prefs", {
          prefs: {
            base_url: "mock://unibrowser-self-test",
            model: "docs",
            api_key: null,
            grants: [
              { permission: "read", scope_kind: "global", scope_value: null },
              { permission: "ask", scope_kind: "workspace", scope_value: "/workspace" },
            ],
            panel_visible: true,
            panel_x: 24,
            panel_y: 96,
            panel_docked: true,
          },
        });
        await setAiPanelVisible(true);
        break;
      case "live":
      case "live-preview":
        try {
          await invoke("set_unibrow_live_mode", { enabled: true });
        } catch {
          /* unibrow tab may not be active yet */
        }
        break;
      case "vsix":
      case "vsix-badge":
        await invoke("open_path", { path: "unibrowser://mods", line: null });
        break;
      case "palette":
        document.getElementById("palette")?.classList.remove("hidden");
        const pq = document.getElementById("palette-query") as HTMLInputElement | null;
        if (pq) {
          pq.value = "font";
          pq.dispatchEvent(new Event("input", { bubbles: true }));
        }
        break;
      case "settings":
        window.dispatchEvent(
          new CustomEvent("unibrowser-navigate", { detail: "unibrowser://settings" }),
        );
        await waitForElement(".settings-page");
        break;
      case "mods":
        await invoke("open_path", { path: "unibrowser://mods", line: null });
        break;
      case "theme-light":
        applyThemeOverride("light");
        {
          const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
          await applyMonacoEditorTheme("auto");
        }
        break;
      case "theme-dark":
        applyThemeOverride("dark");
        {
          const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
          await applyMonacoEditorTheme("auto");
        }
        break;
      case "title-breadcrumb":
        document.body.dataset.titleBreadcrumb = "1";
        break;
      case "second-tab-sample-pdf":
        await invoke("open_path", {
          path: "/workspace/examples/demo-pack/sample.pdf",
          line: null,
        }).catch(() => undefined);
        break;
      case "ai-mockup-composer":
        setAiDemoComposerSeed("Why does line 8 trigger a warning?");
        break;
      case "feedback-demo": {
        showActionToast(t("tabs.closedUndo"), t("tabs.undo"), () => undefined, "info");
        const host = document.getElementById("recovery-bar-host");
        if (host) {
          host.classList.remove("hidden");
          const row = document.createElement("div");
          row.className = "recovery-bar-row";
          const label = document.createElement("span");
          label.textContent = "Unsaved changes were recovered for spell-test.txt";
          const view = document.createElement("button");
          view.type = "button";
          view.textContent = "View original";
          const discard = document.createElement("button");
          discard.type = "button";
          discard.textContent = "Discard";
          row.append(label, view, discard);
          host.replaceChildren(row);
        }
        break;
      }
      case "ntp":
        await invoke("new_tab", {});
        break;
      case "files":
        await invoke("open_path", { path: "~", line: null }).catch(() => undefined);
        break;
      case "ui26-file-preview":
      case "ui13-files-preview": {
        const links = await waitForElement<HTMLButtonElement>(".files-table .file-link");
        if (!links) {
          break;
        }
        const preferred = [...document.querySelectorAll<HTMLButtonElement>(".files-table .file-link")]
          .find((link) => link.textContent?.toLowerCase().endsWith(".svg"));
        const row = (preferred ?? links).closest("tr");
        row?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
        await waitForElement(".files-side-panel:not(.hidden) .files-props-dl");
        break;
      }
      case "ui13-settings-developer-collapsed":
      case "ui13-settings-developer-expanded": {
        const details = await waitForElement<HTMLDetailsElement>(".settings-developer");
        if (!details) {
          break;
        }
        details.open = token.endsWith("expanded");
        const category = document.querySelector<HTMLButtonElement>(
          '[data-settings-category-target="developer"]',
        );
        category?.click();
        details.scrollIntoView({ block: "start" });
        break;
      }
      case "cm-large": {
        const { applyCommandManagerDensity } = await import("./accessibility");
        applyCommandManagerDensity(false);
        break;
      }
      case "cm-compact": {
        const { applyCommandManagerDensity } = await import("./accessibility");
        applyCommandManagerDensity(true);
        break;
      }
      case "cmd11-plain-proof":
      case "cmd11-markdown-proof": {
        const controlId =
          token === "cmd11-plain-proof"
            ? "unibrowser.control.text.strikethrough"
            : "unibrowser.control.text.subscript";
        await new Promise((resolve) => window.setTimeout(resolve, 50));
        const control = document.querySelector<HTMLElement>(
          `[data-control-id="${controlId}"]`,
        );
        control?.focus();
        break;
      }
      case "ui14-dxf-measure-reason":
        break;
      case "ui14-open-command-overflow": {
        const overflow = await waitForElement<HTMLButtonElement>(
          "#command-manager-overflow.visible",
        );
        overflow?.click();
        await waitForElement(".cm-overflow-menu");
        break;
      }
      case "ui14-pdf-100":
        await invoke("sync_pdf_scale", { scale: 1 }).catch(() => undefined);
        break;
      case "ui15-unibrow-title": {
        const clip = await waitForElement<HTMLElement>(
          '.workshop-clip[data-clip-id="clip-1"]',
        );
        clip?.focus();
        break;
      }
      case "ui16-live-preview":
        window.dispatchEvent(
          new CustomEvent("unibrow-workshop-visual-mode", {
            detail: "preview",
          }),
        );
        break;
      case "ui16-live-on": {
        window.dispatchEvent(
          new CustomEvent("unibrow-workshop-visual-mode", {
            detail: "live",
          }),
        );
        break;
      }
      case "ui16-live-recording": {
        window.dispatchEvent(
          new CustomEvent("unibrow-workshop-visual-mode", {
            detail: "recording",
          }),
        );
        break;
      }
      case "ui17-mod-portal": {
        const data = await invoke("open_mod_portal", { ext: "step" }).catch(
          () => null,
        );
        if (data) {
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
        }
        break;
      }
      case "ui18-layout-edit": {
        const { setLayoutEditMode } = await import("./layoutEditMode");
        setLayoutEditMode(true);
        break;
      }
      case "ui19-focus-visible": {
        const command = await waitForElement<HTMLButtonElement>(
          '[data-control-id="unibrowser.control.find"], #command-manager-groups .ui-command',
        );
        command?.focus();
        break;
      }
      case "rail-expanded": {
        const { setRailMode } = await import("./ui/activityRail");
        setRailMode("expanded");
        break;
      }
      case "rail-collapsed": {
        const { setRailMode } = await import("./ui/activityRail");
        setRailMode("collapsed");
        break;
      }
      case "scale-100":
      case "scale-150":
        {
          const { applyUiScaleToDocument } = await import("./accessibility");
          const percent = token === "scale-150" ? 150 : 100;
          applyUiScaleToDocument(percent);
          await invoke("set_ui_scale_percent", { percent }).catch(() => undefined);
        }
        break;
      case "standard-contrast": {
        const { applyHighContrastToDocument } = await import("./accessibility");
        applyHighContrastToDocument(false);
        await invoke("set_accessibility_prefs", {
          high_contrast: false,
          reduce_motion: null,
        }).catch(() => undefined);
        break;
      }
      case "high-contrast": {
        const { applyHighContrastToDocument } = await import("./accessibility");
        applyHighContrastToDocument(true);
        await invoke("set_accessibility_prefs", {
          high_contrast: true,
          reduce_motion: null,
        }).catch(() => undefined);
        break;
      }
      case "cm-collapsed":
        document.body.classList.add("cm-collapsed");
        break;
      case "cm-image-tab":
        await invoke("select_command_manager_tab", { tabId: "image" });
        window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
        break;
      case "cm-image-tab-pin":
        await invoke("select_command_manager_tab", { tabId: "image" });
        await invoke("pin_command_manager_context");
        window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
        break;
      case "content-kind-grayscale-overflow": {
        const overflow = await waitForElement<HTMLButtonElement>(
          "#command-manager-overflow.visible",
        );
        overflow?.click();
        await waitForElement(".cm-overflow-menu");
        break;
      }
      case "content-kind-md-preview": {
        const host = await waitForElement<HTMLElement>("#text-editor-host:not(.hidden)");
        if (!host) {
          break;
        }
        const { setMarkdownPreviewMode } = await import("./textEditor");
        document.documentElement.classList.add("ub-capture-md-preview");
        for (let attempt = 0; attempt < 80; attempt += 1) {
          await setMarkdownPreviewMode(true, host);
          const preview = document.querySelector(".md-preview-body");
          if (preview && (preview.textContent ?? "").trim().length >= 10) {
            break;
          }
          await new Promise((resolve) => window.setTimeout(resolve, 50));
        }
        window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
        {
          const { syncDocumentCardDirtyFlag } = await import("./ui/textDocumentCard");
          syncDocumentCardDirtyFlag(false);
        }
        break;
      }
      case "content-kind-embedded-image-ui": {
        const img = await waitForElement<HTMLImageElement>(
          ".docx-view img, .md-preview img",
        );
        if (img) {
          img.scrollIntoView({ block: "center" });
          img.click();
          img.focus();
        }
        break;
      }
      case "content-kind-grayscale-focus": {
        await new Promise((resolve) => window.setTimeout(resolve, 120));
        const overflow = document.querySelector<HTMLButtonElement>(
          "#command-manager-overflow.visible",
        );
        if (
          overflow &&
          !document.querySelector(
            '#command-manager-groups [data-control-id="image-grayscale-preview.control.grayscale"]',
          )
        ) {
          overflow.click();
          await waitForElement(".cm-overflow-menu");
        }
        const control =
          document.querySelector<HTMLElement>(
            '#command-manager-groups [data-control-id="image-grayscale-preview.control.grayscale"]',
          ) ??
          document.querySelector<HTMLElement>(
            '.cm-overflow-menu [data-control-id="image-grayscale-preview.control.grayscale"]',
          );
        control?.focus();
        break;
      }
      case "capture-ready":
        await invoke("seed_demo_ntp_fixture_cmd").catch(() => undefined);
        await invoke("clear_command_manager_context_pin").catch(() => undefined);
        for (const ctx of ["ntp", "pdf", "image"] as const) {
          try {
            localStorage.setItem(`ub-dock-ai-pref:${ctx}`, "0");
          } catch {
            /* ignore */
          }
        }
        await setAiPanelVisible(false);
        applyThemeFromDemoState(demoRaw);
        {
          const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
          await applyMonacoEditorTheme("auto");
        }
        break;
      default:
        break;
    }
  }
}
