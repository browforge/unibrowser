import { cpSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const publicDir = path.join(root, "public");
const demoSrc = path.resolve(root, "../../examples/demo-pack");
const demoDest = path.join(publicDir, "demo-pack");

mkdirSync(publicDir, { recursive: true });
rmSync(demoDest, { recursive: true, force: true });
cpSync(demoSrc, demoDest, { recursive: true });
