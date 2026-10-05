import { invoke } from "@tauri-apps/api/core";

let timer: ReturnType<typeof setInterval> | null = null;

export function stopProjectAutoSaveTimer() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
}

export function configureProjectAutoSave(enabled: boolean, intervalSecs: number) {
  stopProjectAutoSaveTimer();
  if (!enabled || intervalSecs < 30) {
    return;
  }
  const ms = Math.min(intervalSecs, 600) * 1000;
  timer = setInterval(() => {
    void invoke<string>("autosave_unibrow_active_project").catch(() => {});
  }, ms);
}
