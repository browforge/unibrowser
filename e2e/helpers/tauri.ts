import { browser, $ } from "@wdio/globals";

type TauriInvoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

export async function waitForWindowHandle(timeoutMs = 120000) {
  await browser.waitUntil(
    async () => {
      try {
        const handles = await browser.getWindowHandles();
        return handles.length > 0;
      } catch {
        return false;
      }
    },
    { timeout: timeoutMs, interval: 250, timeoutMsg: "No WebDriver window handles" },
  );
}

export async function tauriInvoke<T = unknown>(
  cmd: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const tauri = (browser as unknown as { tauri?: { execute: Function } }).tauri;
  if (tauri?.execute) {
    try {
      return (await tauri.execute(
        ({ core }: { core: TauriInvoke }) => core.invoke(cmd, args),
      )) as T;
    } catch {
      /* fall through to browser.execute */
    }
  }
  return browser.execute(
    (command, payload) => {
      const api = (window as unknown as { __TAURI__?: { core: TauriInvoke } }).__TAURI__;
      if (!api?.core?.invoke) {
        throw new Error("Tauri invoke API not available");
      }
      return api.core.invoke(command, payload) as Promise<T>;
    },
    cmd,
    args,
  ) as Promise<T>;
}

export async function prepareCleanShell() {
  await browser.waitUntil(
    async () => {
      try {
        await tauriInvoke("set_proofing_squiggles_enabled", { enabled: true });
        return true;
      } catch {
        return false;
      }
    },
    { timeout: 60000, interval: 500, timeoutMsg: "Tauri invoke API not available on loopback shell" },
  );
}

export async function openPath(filePath: string, line: number | null = null) {
  await tauriInvoke("open_path", { path: filePath, line });
  await browser.pause(800);
}

export async function runCommand(commandId: string) {
  const msg = await tauriInvoke<string>("execute_command", { commandId });
  await browser.pause(400);
  return msg;
}

export async function readStatusFontPt(): Promise<string> {
  const el = await $("#status-font");
  return (await el.getText()) || "";
}

export async function waitForUbReady(timeoutMs = 120000) {
  await browser.waitUntil(
    async () => {
      try {
        const ready = await browser.execute(() => document.body?.dataset?.ubReady === "1");
        if (ready) {
          return true;
        }
      } catch {
        /* execute not ready yet */
      }
      try {
        const html = await browser.getPageSource();
        return html.includes("omnibox") || html.includes('id="omnibox"');
      } catch {
        return false;
      }
    },
    {
      timeout: timeoutMs,
      interval: 250,
      timeoutMsg: "Shell did not load (ubReady marker or omnibox missing)",
    },
  );
}

export async function waitForShellReady(timeoutMs = 120000) {
  await waitForUbReady(timeoutMs);
}

export async function waitForSelector(sel: string, timeoutMs = 60000) {
  await waitForShellReady(timeoutMs);
  const el = await $(sel);
  await el.waitForExist({ timeout: timeoutMs });
  return el;
}
