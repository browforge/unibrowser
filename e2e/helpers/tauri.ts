import { browser, $ } from "@wdio/globals";

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
  await tauriInvoke("skip_onboarding_tour_forever");
  await tauriInvoke("set_restore_session_on_launch", { enabled: false });
  await tauriInvoke("set_proofing_squiggles_enabled", { enabled: true });
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

export async function waitForShellReady(timeoutMs = 120000) {
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
