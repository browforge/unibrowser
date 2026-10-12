import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 120_000,
  use: {
    baseURL: process.env.UB_WEB_BASE_URL ?? "http://127.0.0.1:4173/unibrowser/",
    headless: true,
    channel: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH ? undefined : "chrome",
    launchOptions: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH
      ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH }
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
