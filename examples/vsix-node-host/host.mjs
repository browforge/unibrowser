#!/usr/bin/env node
/**
 * Minimal VSIX JS host (JSON lines on stdin/stdout).
 */
import { createRequire } from "node:module";
import { createInterface } from "node:readline";
import path from "node:path";

const extensions = new Map();
let pendingExecuteCtx = null;

class Position {
  constructor(line, character) {
    this.line = line;
    this.character = character;
  }
}

class Range {
  constructor(start, end) {
    this.start = start;
    this.end = end;
  }
}

class Selection extends Range {}

class TextEdit {
  static replace(range, newText) {
    return { range: rangeToDto(range), newText: String(newText) };
  }
  static insert(position, newText) {
    const pos = positionToDto(position);
    return {
      range: { start: pos, end: pos },
      newText: String(newText),
    };
  }
  static delete(range) {
    return TextEdit.replace(range, "");
  }
}

class WorkspaceEdit {
  constructor() {
    this._edits = [];
  }
  replace(uri, range, newText) {
    this._edits.push(TextEdit.replace(range, newText));
  }
  insert(uri, position, newText) {
    this._edits.push(TextEdit.insert(position, newText));
  }
  delete(uri, range) {
    this._edits.push(TextEdit.delete(range));
  }
  get edits() {
    return this._edits;
  }
}

function rangeToDto(range) {
  return {
    start: positionToDto(range.start),
    end: positionToDto(range.end),
  };
}

function positionToDto(pos) {
  return { line: pos.line, character: pos.character };
}

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

function documentFacade(editorSnap) {
  return {
    getText() {
      return editorSnap?.text ?? "";
    },
    languageId: editorSnap?.language_id ?? "plaintext",
    uri: { fsPath: editorSnap?.path ?? "" },
  };
}

function selectorMatches(selector, languageId) {
  if (!selector) {
    return true;
  }
  if (typeof selector === "string") {
    return selector === languageId || selector === "*";
  }
  if (Array.isArray(selector)) {
    return selector.some((s) => selectorMatches(s, languageId));
  }
  if (typeof selector === "object" && selector.language) {
    return selector.language === languageId || selector.language === "*";
  }
  return false;
}

function queueHostCall(ctx, call) {
  ctx.runtime.hostCalls.push(call);
  return new Promise((resolve, reject) => {
    ctx.pendingHostResolvers.set(call.id, { resolve, reject });
  });
}

function buildShim(ctx, runtime) {
  const req = runtime.req;
  const hostCalls = runtime.hostCalls;
  const editorSnap = req.editor;

  const docFacade = editorSnap ? documentFacade(editorSnap) : null;

  const editor = editorSnap
    ? {
        document: docFacade,
        edit(callback) {
          const collected = [];
          const builder = {
            replace(range, newText) {
              collected.push(TextEdit.replace(range, newText));
            },
            insert(position, newText) {
              collected.push(TextEdit.insert(position, newText));
            },
            delete(range) {
              collected.push(TextEdit.delete(range));
            },
          };
          callback(builder);
          if (collected.length) {
            hostCalls.push({
              id: `edit-${hostCalls.length + 1}`,
              kind: "workspace_edit",
              edits: collected,
            });
          }
          return Promise.resolve(collected.length > 0);
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
        hostCalls.push({ id: callId, kind: "builtin", command_id: id });
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
      applyEdit(edit) {
        const edits = edit?.edits ?? [];
        if (edits.length) {
          hostCalls.push({
            id: `ws-${hostCalls.length + 1}`,
            kind: "workspace_edit",
            edits,
          });
        }
        return Promise.resolve(edits.length > 0);
      },
    },
    languages: {
      registerDocumentFormattingEditProvider(selector, provider) {
        ctx.formatProviders.push({ selector, provider });
        return { dispose() {} };
      },
    },
    Position,
    Range,
    Selection,
    TextEdit,
    WorkspaceEdit,
  };
}

function makeContext(extensionId) {
  return {
    extensionId,
    commandHandlers: new Map(),
    formatProviders: [],
    pendingToasts: [],
    pendingHostResolvers: new Map(),
    runtime: {
      req: { editor: null, configuration: {} },
      hostCalls: [],
    },
  };
}

function vscodeApi(ctx) {
  const shim = () => buildShim(ctx, ctx.runtime);
  return {
    get commands() {
      return shim().commands;
    },
    get window() {
      return shim().window;
    },
    get workspace() {
      return shim().workspace;
    },
    get languages() {
      return shim().languages;
    },
    Position,
    Range,
    Selection,
    TextEdit,
    WorkspaceEdit,
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
  ctx.formatProviders.length = 0;
  extensions.delete(extensionId);
}

function pickToast(ctx) {
  return ctx.pendingToasts[0] ?? null;
}

function resolveHostResults(ctx, results) {
  for (const res of results ?? []) {
    const pending = ctx.pendingHostResolvers.get(res.id);
    if (!pending) {
      continue;
    }
    ctx.pendingHostResolvers.delete(res.id);
    if (res.ok) {
      pending.resolve(res.result);
    } else {
      pending.reject(new Error(res.error || "host call failed"));
    }
  }
}

function collectFormattingEdits(languageId, text) {
  const doc = documentFacade({ language_id: languageId, text });
  for (const ctx of extensions.values()) {
    for (const { selector, provider } of ctx.formatProviders) {
      if (!selectorMatches(selector, languageId)) {
        continue;
      }
      const edits = provider.provideDocumentFormattingEdits(doc);
      if (edits && edits.length) {
        return edits.map((e) => ({
          range: e.range,
          new_text: e.newText ?? e.new_text ?? "",
        }));
      }
    }
  }
  return [];
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
  if (op === "format_document") {
    try {
      const edits = collectFormattingEdits(req.language_id ?? "plaintext", req.text ?? "");
      respond({ ok: true, edits });
    } catch (err) {
      respond({ ok: false, error: String(err) });
    }
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
    resolveHostResults(ctx, req.results);
    ctx.runtime.hostCalls = [];
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
