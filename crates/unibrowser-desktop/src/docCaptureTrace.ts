import { invoke } from "@tauri-apps/api/core";

function stamp(): string {
  return new Date().toISOString();
}

export function traceDocCaptureStep(step: string, detail?: string): void {
  const line = detail
    ? `[doc-capture-trace] ${stamp()} ${step} ${detail}`
    : `[doc-capture-trace] ${stamp()} ${step}`;
  console.error(line);
  void invoke("log_shell_ui_event", {
    level: "error",
    message: "doc-capture-trace",
    detail: line,
  }).catch(() => undefined);
}

export async function withDocCaptureTimeout<T>(
  step: string,
  timeoutMs: number,
  work: () => Promise<T>,
): Promise<T> {
  traceDocCaptureStep(step, "start");
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    const result = await Promise.race([
      work(),
      new Promise<T>((_, reject) => {
        timer = setTimeout(() => {
          reject(new Error(`doc-capture stalled at step: ${step} (${timeoutMs}ms)`));
        }, timeoutMs);
      }),
    ]);
    traceDocCaptureStep(step, "done");
    return result;
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    traceDocCaptureStep(step, `failed: ${msg}`);
    throw err;
  } finally {
    if (timer !== undefined) {
      clearTimeout(timer);
    }
  }
}

let consoleForwardInstalled = false;

/** Forward JS console to Rust logs during headless doc capture (RUST_LOG=debug on host). */
export function installDocCaptureConsoleForwarder(): void {
  if (consoleForwardInstalled || typeof window === "undefined") {
    return;
  }
  consoleForwardInstalled = true;
  const forward = (level: "error" | "warn" | "info", args: unknown[]) => {
    const detail = args
      .map((a) => {
        if (typeof a === "string") {
          return a;
        }
        try {
          return JSON.stringify(a);
        } catch {
          return String(a);
        }
      })
      .join(" ");
    void invoke("log_shell_ui_event", {
      level,
      message: "js-console",
      detail: `${stamp()} ${detail}`,
    }).catch(() => undefined);
  };
  const origError = console.error.bind(console);
  const origWarn = console.warn.bind(console);
  const origLog = console.log.bind(console);
  console.error = (...args: unknown[]) => {
    origError(...args);
    forward("error", args);
  };
  console.warn = (...args: unknown[]) => {
    origWarn(...args);
    forward("warn", args);
  };
  console.log = (...args: unknown[]) => {
    origLog(...args);
    forward("info", args);
  };
}
