import { browser, $ } from "@wdio/globals";

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

type TauriInvoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

export async function tauriInvoke<T = unknown>(
  cmd: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  return browser.execute(
    (command, payload) => {
      const tauri = (window as unknown as { __TAURI__?: { core: { invoke: TauriInvoke } } })
        .__TAURI__;
      if (!tauri?.core?.invoke) {
        throw new Error("Tauri invoke API not available");
      }
      return tauri.core.invoke(command, payload) as Promise<T>;
    },
    cmd,
    args,
  ) as Promise<T>;
}

export async function prepareCleanShell() {
  // Rust `setup` already applies UNIBROWSER_E2E prefs (onboarding off, no session restore).
  try {
    await tauriInvoke("set_proofing_squiggles_enabled", { enabled: true });
  } catch {
    /* loopback shell may not expose invoke in some WebDriver attach timings */
  }
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
  await browser.waitUntil(
    async () => {
      try {
        const has = await browser.execute(() => !!document.getElementById("omnibox"));
        if (has) {
          return true;
        }
      } catch {
        /* session not ready yet */
      }
      const html = await browser.getPageSource();
      return html.includes("omnibox") || html.includes("id=\"omnibox\"");
    },
    { timeout: timeoutMs, interval: 500, timeoutMsg: "Unibrowser shell HTML did not load" },
  );
}

export async function waitForSelector(sel: string, timeoutMs = 60000) {
  await waitForShellReady(timeoutMs);
  const el = await $(sel);
  await el.waitForExist({ timeout: timeoutMs });
  return el;
}
