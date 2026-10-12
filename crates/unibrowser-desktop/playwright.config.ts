import { existsSync } from "node:fs";
import { defineConfig } from "@playwright/test";

const systemChrome =
  process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH ??
  (existsSync("/usr/bin/google-chrome-stable")
    ? "/usr/bin/google-chrome-stable"
    : undefined);

export default defineConfig({
  testDir: "./tests",
  timeout: 120_000,
  use: {
    baseURL: process.env.UB_WEB_BASE_URL ?? "http://127.0.0.1:4173/unibrowser/",
    headless: true,
    channel: systemChrome ? undefined : "chrome",
    launchOptions: systemChrome
      ? { executablePath: systemChrome, args: ["--no-sandbox", "--disable-gpu"] }
      : undefined,
  },
  webServer: process.env.UB_WEB_SKIP_SERVER
    ? undefined
    : {
        command: "npx vite preview --mode web --host 127.0.0.1 --port 4173",
        cwd: ".",
        port: 4173,
        reuseExistingServer: true,
      },
});
