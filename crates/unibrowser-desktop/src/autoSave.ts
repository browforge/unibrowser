import { invoke } from "@tauri-apps/api/core";
import { textEditor } from "./textEditor";

const INTERVAL_SECS = [60, 120, 300, 600, 900, 1800];

let timer: ReturnType<typeof setInterval> | null = null;

export function autosaveIntervalOptions(): number[] {
  return INTERVAL_SECS;
}

export function stopAutoSaveTimer() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
}

export function configureAutoSave(enabled: boolean, intervalSecs: number) {
  stopAutoSaveTimer();
  if (!enabled) {
    return;
  }
  const secs = INTERVAL_SECS.includes(intervalSecs) ? intervalSecs : 300;
  const ms = secs * 1000;
  timer = setInterval(() => {
    void textEditor.flush();
    void invoke<{ wrote_slot: boolean; last_autosave_at_ms: number }>("auto_save_active_document")
      .then((dto) => {
        if (dto.wrote_slot) {
          window.dispatchEvent(
            new CustomEvent("unibrowser-autosave-tick", {
              detail: dto.last_autosave_at_ms,
            }),
          );
        }
      })
      .catch(() => {});
  }, ms);
}
