import { fetchDemoPack, pickOpenTextFile, saveTextFile } from "./webFiles";
import { wasmInvoke } from "./wasmHost";

function notifyBrowserState() {
  window.dispatchEvent(new CustomEvent("ub-browser-state-updated"));
}

export async function webInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  switch (cmd) {
    case "take_launch_open_paths":
      return [] as T;
    case "get_demo_state":
      return null as T;
    case "pick_open_file": {
      const picked = await pickOpenTextFile();
      if (!picked) {
        return null as T;
      }
      const state = await wasmInvoke<unknown>("open_text_with_content", {
        path: picked.path,
        content: picked.content,
      });
      notifyBrowserState();
      return state as T;
    }
    case "pick_browse_folder":
      throw new Error("Folder browse needs the desktop app or Chromium folder picker (not in v1)");
    case "navigate_omnibox": {
      const input = String(args?.input ?? "").trim();
      if (input.startsWith("demo-pack/") || input.includes("demo-pack/")) {
        return webInvoke<T>("open_path", { path: input });
      }
      break;
    }
    case "open_path": {
      const path = String(args?.path ?? "");
      if (path.startsWith("demo-pack/") || path.includes("/demo-pack/")) {
        const rel = path.replace(/^.*demo-pack\//, "demo-pack/");
        const content = await fetchDemoPack(rel);
        const fileName = rel.split("/").pop() ?? "sample.md";
        const fullPath = `demo-pack/${fileName}`;
        const state = await wasmInvoke<unknown>("open_text_with_content", {
          path: fullPath,
          content,
        });
        notifyBrowserState();
        return state as T;
      }
      if (args?.content) {
        const state = await wasmInvoke<unknown>("open_text_with_content", args);
        notifyBrowserState();
        return state as T;
      }
      return wasmInvoke<T>(cmd, args);
    }
    case "save_active_document":
    case "write_document_bytes": {
      const pending = await wasmInvoke<{ path: string; content: string }>("save_active_document", args);
      if (pending?.path) {
        await saveTextFile(pending.path, pending.content);
      }
      notifyBrowserState();
      return null as T;
    }
    case "open_releases_page":
    case "open_report_issue_page":
      window.open("https://github.com/browforge/unibrowser/releases", "_blank", "noopener");
      return undefined as T;
    default:
      break;
  }
  const value = await wasmInvoke<T>(cmd, args);
  if (
    cmd === "new_tab" ||
    cmd === "select_tab" ||
    cmd === "close_tab" ||
    cmd === "navigate_omnibox" ||
    cmd === "undo_close_tab" ||
    cmd === "reorder_tabs" ||
    cmd === "toggle_tab_pin" ||
    cmd === "dismiss_onboarding"
  ) {
    notifyBrowserState();
  }
  return value;
}
