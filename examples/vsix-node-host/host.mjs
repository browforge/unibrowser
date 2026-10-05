#!/usr/bin/env node
/**
 * Minimal VSIX JS host (JSON lines on stdin/stdout).
 */
import { createRequire } from "node:module";
import { createInterface } from "node:readline";
import path from "node:path";

const extensions = new Map();
let pendingExecuteCtx = null;

function pushToast(ctx, message, severity) {
  ctx.pendingToasts.push({ message: String(message), severity });
}

function makeConfiguration(configuration, section) {
  const root = configuration && typeof configuration === "object" ? configuration : {};
  const sectionKey = section || "";
  const sectionObj =
    sectionKey && root[sectionKey] && typeof root[sectionKey] === "object"
      ? root[sectionKey]
      : root;
  return {
    get(key, defaultValue) {
      if (!key) {
        return defaultValue;
      }
      const parts = String(key).split(".");
      let cur = sectionObj;
      for (const part of parts) {
        if (cur && typeof cur === "object" && part in cur) {
          cur = cur[part];
        } else {
          return defaultValue;
        }
      }
      return cur === undefined ? defaultValue : cur;
    },
  };
}

function buildShim(ctx, runtime) {
  const req = runtime.req;
  const hostCalls = runtime.hostCalls;
  const editor = req.editor
    ? {
        document: {
          getText() {
            return req.editor.text ?? "";
          },
          languageId: req.editor.language_id ?? "plaintext",
        },
      }
    : undefined;

  return {
    commands: {
      registerCommand(commandId, handler) {
        ctx.commandHandlers.set(commandId, handler);
        return {
          dispose() {
            ctx.commandHandlers.delete(commandId);
          },
        };
      },
      executeCommand(commandId) {
        const id = String(commandId);
        const callId = `call-${hostCalls.length + 1}`;
        hostCalls.push({ id: callId, command_id: id });
        return undefined;
      },
    },
    window: {
      get activeTextEditor() {
        return editor;
      },
      showInformationMessage(message) {
        pushToast(ctx, message, "info");
        return Promise.resolve(undefined);
      },
      showWarningMessage(message) {
        pushToast(ctx, message, "warning");
        return Promise.resolve(undefined);
      },
      showErrorMessage(message) {
        pushToast(ctx, message, "error");
        return Promise.resolve(undefined);
      },
    },
    workspace: {
      getConfiguration(section) {
        return makeConfiguration(req.configuration, section);
      },
    },
  };
}

function makeContext(extensionId) {
  return {
    extensionId,
    commandHandlers: new Map(),
    pendingToasts: [],
    runtime: {
      req: { editor: null, configuration: {} },
      hostCalls: [],
    },
  };
}

function vscodeApi(ctx) {
  return {
    get commands() {
      return buildShim(ctx, ctx.runtime).commands;
    },
    get window() {
      return buildShim(ctx, ctx.runtime).window;
    },
    get workspace() {
      return buildShim(ctx, ctx.runtime).workspace;
    },
  };
}

function activateExtension(installDir, extensionDir, mainRel, extensionId) {
  const key = extensionId;
  if (extensions.has(key)) {
    return extensions.get(key);
  }
  try {
    process.chdir(path.resolve(installDir));
  } catch {
    // ignore chdir failures in tests
  }
  const extRoot = path.resolve(extensionDir);
  const mainPath = path.resolve(extRoot, mainRel);
  const ctx = makeContext(extensionId);
  const req = createRequire(path.join(extRoot, "package.json"));
  const mod = req(mainPath);
  const vscode = vscodeApi(ctx);
  if (typeof mod.activate === "function") {
    mod.activate(vscode);
  } else if (typeof mod === "function") {
    mod(vscode);
  }
  extensions.set(key, ctx);
  return ctx;
}

function deactivateExtension(extensionId) {
  const ctx = extensions.get(extensionId);
  if (!ctx) {
    return;
  }
  ctx.commandHandlers.clear();
  extensions.delete(extensionId);
}

function pickToast(ctx) {
  return ctx.pendingToasts[0] ?? null;
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
      activateExtension(
        req.install_dir,
        req.extension_dir,
        req.main,
        req.extension_id,
      );
      respond({ ok: true, activated: true });
    } catch (err) {
      respond({ ok: false, error: String(err) });
    }
    return;
  }
  if (op === "deactivate") {
    deactivateExtension(req.extension_id);
    pendingExecuteCtx = null;
    respond({ ok: true });
    return;
  }
  if (op === "host_results") {
    const ctx = pendingExecuteCtx;
    pendingExecuteCtx = null;
    if (!ctx) {
      respond({ ok: false, error: "no pending execute" });
      return;
    }
    respond({ ok: true, complete: true, toast: pickToast(ctx) });
    return;
  }
  if (op === "execute") {
    try {
      activateExtension(
        req.install_dir,
        req.extension_dir,
        req.main,
        req.extension_id,
      );
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
      ctx.runtime.req = {
        editor: req.editor ?? null,
        configuration: req.configuration ?? {},
      };
      ctx.runtime.hostCalls = [];
      const result = handler();
      const finish = () => {
        if (ctx.runtime.hostCalls.length) {
          pendingExecuteCtx = ctx;
          respond({
            ok: true,
            complete: false,
            execute_token: req.execute_token,
            host_calls: ctx.runtime.hostCalls,
          });
          return;
        }
        respond({ ok: true, complete: true, toast: pickToast(ctx) });
      };
      if (result && typeof result.then === "function") {
        result.then(finish).catch((err) => {
          respond({ ok: false, error: String(err) });
        });
        return;
      }
      finish();
    } catch (err) {
      respond({ ok: false, error: String(err) });
    }
    return;
  }
  respond({ ok: false, error: `unknown op: ${op}` });
});
