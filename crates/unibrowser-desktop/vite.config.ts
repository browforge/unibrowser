import { defineConfig } from "vite";
// @ts-expect-error type error without @types/node package
import path from "node:path";
// @ts-expect-error type error without @types/node package
import process from "node:process";
// @ts-expect-error type error without @types/node package
import { fileURLToPath } from "node:url";

const host = process.env.TAURI_DEV_HOST;
const rootDir = path.dirname(fileURLToPath(import.meta.url));

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const isWeb = mode === "web";
  const shim = (name: string) => path.resolve(rootDir, "src/platform/shims", name);

  return {
    base: isWeb ? "/unibrowser/" : "/",
    define: {
      "import.meta.env.VITE_UB_PLATFORM": JSON.stringify(isWeb ? "web" : "desktop"),
    },
    build: {
      outDir: isWeb ? "dist-web" : "dist",
      rollupOptions: {
        input: isWeb ? path.resolve(rootDir, "index-web.html") : path.resolve(rootDir, "index.html"),
        output: {
          manualChunks(id) {
            if (id.includes("node_modules/monaco-editor") || id.includes("@monaco-editor")) {
              return "monaco";
            }
            if (id.includes("node_modules/pdfjs-dist")) {
              return "pdfjs";
            }
            if (id.includes("node_modules/three")) {
              return "three";
            }
          },
        },
      },
    },
    resolve: {
      alias: isWeb
        ? {
            "@tauri-apps/api/core": shim("core.ts"),
            "@tauri-apps/api/webviewWindow": shim("webviewWindow.ts"),
            "@tauri-apps/api/event": shim("event.ts"),
            "@tauri-apps/api/window": shim("window.ts"),
          }
        : {},
    },
    clearScreen: false,
    server: {
      port: isWeb ? 5173 : 1420,
      strictPort: !isWeb,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
    },
  };
});
