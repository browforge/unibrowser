import type { languages } from "monaco-editor";
import { Registry } from "monaco-textmate";
import { loadWASM } from "vscode-oniguruma";
import { wireTmGrammars } from "monaco-editor-textmate";
import { invoke } from "@tauri-apps/api/core";
import onigWasmUrl from "vscode-oniguruma/release/onig.wasm?url";

export interface VsixMonacoGrammarDto {
  language: string;
  scope_name: string;
  grammar_json: string;
  extension_id: string;
}

type MonacoModule = typeof import("monaco-editor");

let onigReady = false;

async function ensureOnig(): Promise<void> {
  if (onigReady) {
    return;
  }
  const response = await fetch(onigWasmUrl);
  const bytes = await response.arrayBuffer();
  await loadWASM(bytes);
  onigReady = true;
}

/** Apply enabled VSIX TextMate grammars (Oniguruma WASM). Safe to call after VSIX install/enable. */
export async function refreshVsixTextmateGrammars(monaco: MonacoModule): Promise<void> {
  const grammars = await invoke<VsixMonacoGrammarDto[]>("list_monaco_grammars_cmd");
  if (!grammars.length) {
    return;
  }
  await ensureOnig();
  for (const g of grammars) {
    if (!monaco.languages.getLanguages().some((l: languages.ILanguageExtensionPoint) => l.id === g.language)) {
      monaco.languages.register({ id: g.language });
    }
  }
  const registry = new Registry({
    getGrammarDefinition: async (scopeName: string, _dependentScope: string) => {
      const hit = grammars.find((g) => g.scope_name === scopeName);
      if (!hit) {
        throw new Error(`VSIX grammar missing for scope ${scopeName}`);
      }
      return { format: "json", content: hit.grammar_json };
    },
  });
  const languageMap = new Map(grammars.map((g) => [g.language, g.scope_name]));
  await wireTmGrammars(monaco, registry, languageMap);
}
