import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dirname, "..");
const indexPath = join(root, "dist", "index.html");
const html = readFileSync(indexPath);
const sha = createHash("sha256").update(html).digest("hex");
writeFileSync(join(root, "dist", ".ub-frontend-stamp"), `${sha}\n`, "utf8");
