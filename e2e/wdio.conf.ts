import path from "node:path";
import { fileURLToPath } from "node:url";
import { REPO_ROOT } from "./helpers/paths.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const APP_BINARY = path.join(REPO_ROOT, "target/release/unibrowser-desktop");

/** Dist static server on :18765 must be up before WDIO (see scripts/e2e.sh). */
export const config = {
  runner: "local",
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      project: path.join(__dirname, "tsconfig.json"),
      transpileOnly: true,
    },
  },
  specs: [path.join(__dirname, "specs/**/*.e2e.ts")],
  exclude: process.env.CAPTURE_SCREENSHOTS
    ? [path.join(__dirname, "specs/shell.e2e.ts")]
    : [
        path.join(__dirname, "specs/screenshots.e2e.ts"),
        path.join(__dirname, "specs/shell.e2e.ts"),
      ],
  maxInstances: 1,
  services: [
    [
      "@wdio/tauri-service",
      {
        driverProvider: "external",
        appBinaryPath: APP_BINARY,
        tauriDriverPort: 4444,
        startTimeout: 120000,
        captureBackendLogs: process.env.CAPTURE_SCREENSHOTS === "1",
        env: {
          UNIBROWSER_E2E: "1",
          UNIBROWSER_E2E_ASSET_BASE: "http://127.0.0.1:18765/index.html",
        },
      },
    ],
  ],
  capabilities: [
    {
      browserName: "tauri",
      "tauri:options": {
        application: APP_BINARY,
      },
      "wdio:tauriServiceOptions": {
        windowLabel: "main",
      },
    },
  ],
  logLevel: "info",
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 120000,
  },
  waitforTimeout: 60000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  before: async () => {
    process.chdir(REPO_ROOT);
    process.env.UNIBROWSER_E2E = "1";
    const { waitForWindowHandle } = await import("./helpers/tauri.js");
    await waitForWindowHandle();
  },
  beforeSession: async () => {
    await new Promise((resolve) => setTimeout(resolve, 500));
  },
};
