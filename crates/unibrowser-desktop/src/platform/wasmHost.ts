type WasmModule = {
  default: () => Promise<unknown>;
  host_init: () => void;
  host_invoke: (cmd: string, argsJson: string) => string;
};

let wasmReady: Promise<WasmModule> | null = null;

export async function initWasmHost(): Promise<WasmModule> {
  if (!wasmReady) {
    const wasmUrl = `${import.meta.env.BASE_URL}wasm/unibrowser_web_host.js`;
    wasmReady = import(/* @vite-ignore */ wasmUrl).then(async (mod) => {
      const wasm = mod as WasmModule;
      await wasm.default();
      wasm.host_init();
      return wasm;
    });
  }
  return wasmReady;
}

export async function wasmInvoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const wasm = await initWasmHost();
  const argsJson = args ? JSON.stringify(args) : "null";
  const raw = wasm.host_invoke(cmd, argsJson);
  const envelope = JSON.parse(raw) as { ok: boolean; value: T; error?: string };
  if (!envelope.ok) {
    throw new Error(envelope.error ?? `WASM invoke failed: ${cmd}`);
  }
  return envelope.value;
}
