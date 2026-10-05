#!/usr/bin/env node
/**
 * Minimal VSIX JS host (JSON lines on stdin/stdout).
 * Loads one extension `main` module with a tiny `vscode` shim.
 */
import { createRequire } from "node:module";
import { createInterface } from "node:readline";
import path from "node:path";
import { fileURLToPath } from "node:url";

const extensions = new Map();

function makeContext(extensionId) {
  const subscriptions = [];
  const commandHandlers = new Map();
  const pendingToasts = [];

  const vscode = {
    commands: {
      registerCommand(commandId, handler) {
        commandHandlers.set(commandId, handler);
        const disposable = {
          dispose() {
            commandHandlers.delete(commandId);
          },
        };
        subscriptions.push(disposable);
        return disposable;
      },
    },
    window: {
      showInformationMessage(message) {
        pendingToasts.push({ message: String(message), severity: "info" });
        return Promise.resolve(undefined);
      },
    },
  };

  return {
    extensionId,
    subscriptions,
    commandHandlers,
    pendingToasts,
    vscode,
  };
}

function activateExtension(extensionDir, mainRel, extensionId) {
  const key = extensionId;
  if (extensions.has(key)) {
    return extensions.get(key);
  }
  const extRoot = path.resolve(extensionDir);
  const mainPath = path.resolve(extRoot, mainRel);
  const ctx = makeContext(extensionId);
  const req = createRequire(path.join(extRoot, "package.json"));
  const mod = req(mainPath);
  if (typeof mod.activate === "function") {
    mod.activate(ctx.vscode);
  } else if (typeof mod === "function") {
    mod(ctx.vscode);
  }
  extensions.set(key, ctx);
  return ctx;
}

function respond(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

const rl = createInterface({ input: process.stdin, terminal: false });
rl.on("line", (line) => {
  const trimmed = line.trim();
  if (!trimmed) {
    return;
  }
  let req;
  try {
    req = JSON.parse(trimmed);
  } catch (err) {
    respond({ ok: false, error: `invalid json: ${err}` });
    return;
  }
  const op = req.op;
  if (op === "ping") {
    respond({ ok: true, pong: true });
    return;
  }
  if (op === "activate") {
    try {
      activateExtension(req.extension_dir, req.main, req.extension_id);
      respond({ ok: true, activated: true });
    } catch (err) {
      respond({ ok: false, error: String(err) });
    }
    return;
  }
  if (op === "execute") {
    try {
      const ctx = extensions.get(req.extension_id);
      if (!ctx) {
        respond({ ok: false, error: "extension not activated" });
        return;
      }
      const handler = ctx.commandHandlers.get(req.command_id);
      if (!handler) {
        respond({ ok: false, error: `no handler for ${req.command_id}` });
        return;
      }
      ctx.pendingToasts.length = 0;
      const result = handler();
      if (result && typeof result.then === "function") {
        result
          .then(() => {
            const toast = ctx.pendingToasts[0] ?? null;
            respond({ ok: true, toast });
          })
          .catch((err) => {
            respond({ ok: false, error: String(err) });
          });
        return;
      }
      const toast = ctx.pendingToasts[0] ?? null;
      respond({ ok: true, toast });
    } catch (err) {
      respond({ ok: false, error: String(err) });
    }
    return;
  }
  respond({ ok: false, error: `unknown op: ${op}` });
});
