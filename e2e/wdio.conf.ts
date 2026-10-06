import path from "node:path";
import { fileURLToPath } from "node:url";
import { REPO_ROOT } from "./helpers/paths.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/** App must already be running with embedded WebDriver on :4445 (see scripts/e2e.sh). */
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
    ? []
    : [path.join(__dirname, "specs/screenshots.e2e.ts")],
  maxInstances: 1,
  hostname: "127.0.0.1",
  port: 4445,
  path: "/",
  capabilities: [
    {
      browserName: "webkit",
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
  },
  beforeSession: async () => {
    await new Promise((resolve) => setTimeout(resolve, 3000));
  },
};
