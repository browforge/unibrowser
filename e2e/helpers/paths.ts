import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(e2eDir, "../..");
export const DEMO_PACK = path.join(REPO_ROOT, "examples/demo-pack");
export const FIXTURES = path.join(REPO_ROOT, "e2e/fixtures");
const releaseBinary = path.join(REPO_ROOT, "target/release/unibrowser-desktop");
const debugBinary = path.join(REPO_ROOT, "target/debug/unibrowser-desktop");

/** Prefer release (dist assets, stable WebDriver); fall back to debug when only that exists. */
export const APP_BINARY =
  process.env.UNIBROWSER_E2E_BINARY ??
  (fs.existsSync(releaseBinary) ? releaseBinary : debugBinary);
