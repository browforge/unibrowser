import { invoke } from "@tauri-apps/api/core";
import { showProblemsBottomPanel } from "./problemsPanel";
import { appendActiveEditorText } from "./textEditor";

export interface SelfTestPlan {
  scenarios: string[];
  screenshot_dir: string;
  demo_pack: string;
  fixtures: string;
  projects_dir: string;
}

interface ScenarioResult {
  id: string;
  passed: boolean;
  message: string | null;
  screenshot: string | null;
}

const INCREASE_FONT = "unibrowser.font.size.increase";
const PALETTE_TOGGLE = "unibrowser.palette.toggle";

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

function joinPath(dir: string, name: string) {
  return dir.endsWith("/") ? `${dir}${name}` : `${dir}/${name}`;
}

async function openPath(filePath: string) {
  const dto = await invoke<Record<string, unknown>>("open_path", { path: filePath, line: null });
  const apply = (window as unknown as { __selfTestApplyState?: (d: unknown) => void })
    .__selfTestApplyState;
  if (apply) {
    apply(dto);
  }
  await sleep(900);
}

async function captureShot(_scenarioId: string): Promise<string | null> {
  if (import.meta.env?.VITE_SELF_TEST_SKIP_SHOTS === "1") {
    return null;
  }
  try {
    const shot = invoke<string>("self_test_capture_screenshot", { scenarioId: _scenarioId });
    const timeout = sleep(4_000).then(() => null as string | null);
    return await Promise.race([shot, timeout]);
  } catch {
    return null;
  }
}

async function runFontTextSvg(plan: SelfTestPlan) {
  const txt = joinPath(plan.demo_pack, "sample.txt");
  const svg = joinPath(plan.demo_pack, "sample.svg");
  await openPath(txt);
  const host = document.querySelector("#text-editor-host");
  if (!host || host.classList.contains("hidden")) {
    throw new Error("text editor host not visible");
  }
  const before = document.querySelector("#status-font")?.textContent ?? "";
  await invoke<string>("execute_command", { commandId: INCREASE_FONT });
  await sleep(400);
  const after = document.querySelector("#status-font")?.textContent ?? "";
  if (after === before) {
    throw new Error("font pt unchanged on text tab");
  }
  await openPath(svg);
  const viewer = document.querySelector("#document-viewer-host");
  if (!viewer || viewer.classList.contains("hidden")) {
    throw new Error("document viewer not visible for SVG");
  }
  const msg = await invoke<string>("execute_command", { commandId: INCREASE_FONT });
  if (!msg) {
    throw new Error("empty execute_command result on SVG");
  }
}

async function runUnknownBanner(plan: SelfTestPlan) {
  const unknown = joinPath(plan.fixtures, "unknown.ubunknown");
  await openPath(unknown);
  const banner = document.getElementById("unknown-file-banner");
  if (!banner || banner.classList.contains("hidden")) {
    throw new Error("unknown-file banner missing");
  }
  const text = banner.textContent ?? "";
  if (!text.toLowerCase().includes("mod")) {
    throw new Error("banner copy unexpected");
  }
  const modal = document.querySelector(".inline-ui-layer:not(.hidden)");
  if (modal) {
    throw new Error("unexpected modal layer");
  }
}

async function runCloseUndo(plan: SelfTestPlan) {
  const txt = joinPath(plan.fixtures, "spell-test.txt");
  await openPath(txt);
  await sleep(600);
  const marker = `self-test-undo-${Date.now()}`;
  if (!appendActiveEditorText(`\n${marker}`)) {
    throw new Error("could not append to text editor");
  }
  await sleep(400);
  const closeBtn = document.querySelector(".tab-close");
  if (!closeBtn) {
    throw new Error("tab close control missing");
  }
  (closeBtn as HTMLElement).click();
  await sleep(400);
  const undoBtn = document.querySelector(".toast-host button");
  if (!undoBtn) {
    throw new Error("undo toast missing");
  }
  (undoBtn as HTMLElement).click();
  await sleep(700);
  if (!document.body.innerHTML.includes(marker)) {
    throw new Error("marker missing after undo");
  }
}

async function runProofingDictionary(plan: SelfTestPlan) {
  const txt = joinPath(plan.fixtures, "spell-test.txt");
  await openPath(txt);
  await sleep(1200);
  const before = await invoke<{ message: string }[]>("proofing_check", {
    text: "teh quick brown",
    surface: "plaintext",
    language: "en-NZ",
  });
  if (before.length === 0) {
    throw new Error("expected proofing hit for 'teh'");
  }
  await invoke("proofing_dictionary_add", {
    word: "teh",
    scope: "workspace",
    ignore: false,
  });
  const after = await invoke<{ message: string }[]>("proofing_check", {
    text: "teh quick brown",
    surface: "plaintext",
    language: "en-NZ",
  });
  const stillTeh = after.some((d) => d.message.toLowerCase().includes("teh"));
  if (stillTeh) {
    throw new Error("dictionary add did not clear teh");
  }
}

async function runPalette(plan: SelfTestPlan) {
  const txt = joinPath(plan.demo_pack, "sample.txt");
  await openPath(txt);
  await sleep(500);
  await invoke<string>("execute_command", { commandId: PALETTE_TOGGLE });
  await sleep(300);
  const palette = document.getElementById("palette");
  if (!palette || palette.classList.contains("hidden")) {
    throw new Error("palette not open");
  }
  const query = document.getElementById("palette-query") as HTMLInputElement | null;
  if (!query) {
    throw new Error("palette query missing");
  }
  query.value = "Increase font";
  query.dispatchEvent(new Event("input", { bubbles: true }));
  await sleep(350);
  const item = document.querySelector(".palette-item");
  if (!item) {
    throw new Error("palette item missing");
  }
  (item as HTMLElement).click();
  await sleep(450);
  if (!palette.classList.contains("hidden")) {
    throw new Error("palette still open after run");
  }
}

async function runCmGrey(plan: SelfTestPlan) {
  const txt = joinPath(plan.demo_pack, "sample.txt");
  const png = joinPath(plan.demo_pack, "sample.png");
  await openPath(txt);
  await sleep(800);
  if (!document.querySelector("#command-manager-commands .ui-command")) {
    throw new Error("command manager empty on text tab");
  }
  await openPath(png);
  await sleep(1200);
  const disabled = document.querySelectorAll("#command-manager-commands .ui-command.disabled");
  if (disabled.length === 0) {
    throw new Error("expected disabled commands on image tab");
  }
}

async function runUnibrowBlank(plan: SelfTestPlan) {
  await invoke("create_unibrow_project", {
    parentDir: plan.projects_dir,
    name: `self-test-blank-${Date.now()}`,
    template: "blank",
  });
  await sleep(600);
  const dest = await invoke<string>("autosave_unibrow_active_project");
  if (!dest || dest.length < 4) {
    throw new Error("autosave returned empty path");
  }
}

async function runProblemsJump(plan: SelfTestPlan) {
  const txt = joinPath(plan.fixtures, "spell-test.txt");
  await openPath(txt);
  await sleep(800);
  await invoke("proofing_check", {
    text: "teh quick brown fox",
    surface: "plaintext",
    language: "en-NZ",
  });
  showProblemsBottomPanel();
  await sleep(800);
  const problems = await invoke<
    { navigation: { kind: string; line?: number }; message: string }[]
  >("list_problems_panel", { severity: null, source: null });
  if (problems.length === 0) {
    throw new Error("no problems listed");
  }
  const navPromise = new Promise<void>((resolve, reject) => {
    const t = window.setTimeout(() => reject(new Error("navigate event timeout")), 8000);
    const handler = () => {
      window.clearTimeout(t);
      window.removeEventListener("unibrowser-navigate-text", handler);
      resolve();
    };
    window.addEventListener("unibrowser-navigate-text", handler, { once: true });
  });
  const first = document.querySelector(".problems-list .problems-item");
  if (!first) {
    throw new Error("problems list empty in DOM");
  }
  (first as HTMLElement).click();
  await navPromise;
}

const RUNNERS: Record<string, (plan: SelfTestPlan) => Promise<void>> = {
  "font-text-svg": runFontTextSvg,
  "unknown-file-banner": runUnknownBanner,
  "close-undo": runCloseUndo,
  "proofing-dictionary": runProofingDictionary,
  "palette-command": runPalette,
  "cm-grey-tab-switch": runCmGrey,
  "unibrow-blank-save": runUnibrowBlank,
  "problems-jump": runProblemsJump,
};

async function ping(phase: string) {
  await invoke("self_test_ping", { phase }).catch(() => undefined);
}

export async function runSelfTests(plan: SelfTestPlan) {
  const results: ScenarioResult[] = [];
  for (const id of plan.scenarios) {
    await ping(`scenario-${id}-start`);
    const runner = RUNNERS[id];
    if (!runner) {
      const screenshot = await captureShot(id);
      results.push({
        id,
        passed: false,
        message: `unknown scenario: ${id}`,
        screenshot,
      });
      continue;
    }
    try {
      await runner(plan);
      const screenshot = await captureShot(id);
      results.push({ id, passed: true, message: null, screenshot });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      const screenshot = await captureShot(id);
      results.push({ id, passed: false, message, screenshot });
    }
    await ping(`scenario-${id}-done`);
    await sleep(250);
  }
  const ok = results.every((r) => r.passed);
  await ping("finish");
  await invoke("self_test_finish", {
    report: {
      ok,
      scenarios: results,
      at_ms: Date.now(),
    },
  });
}
