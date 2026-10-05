import { invoke } from "@tauri-apps/api/core";
import { showToast } from "./toast";
import { t } from "./i18n";
import { textEditor } from "./textEditor";

let timer: ReturnType<typeof setInterval> | null = null;

export function stopAutoSaveTimer() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
}

export function configureAutoSave(enabled: boolean, intervalSecs: number) {
  stopAutoSaveTimer();
  if (!enabled || intervalSecs < 30) {
    return;
  }
  const ms = Math.min(intervalSecs, 600) * 1000;
  timer = setInterval(() => {
    void textEditor.flush();
    void invoke<string>("auto_save_active_document")
      .then((msg) => {
        if (msg) {
          showToast(t("autosave.saved"), "info");
        }
      })
      .catch(() => {});
  }, ms);
}
