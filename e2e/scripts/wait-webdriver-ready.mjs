#!/usr/bin/env node
/** Wait for embedded WebDriver /status ready before WDIO (no session create; that races WDIO). */
const BASE = process.env.WDIO_BASE_URL ?? "http://127.0.0.1:4445";

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

for (let i = 0; i < 120; i++) {
  try {
    const res = await fetch(`${BASE}/status`);
    const body = await res.json();
    const ready = body?.value?.ready ?? body?.ready;
    if (ready === true) {
      console.log("WebDriver /status ready");
      await sleep(2000);
      process.exit(0);
    }
  } catch {
    /* server still booting */
  }
  await sleep(500);
}

console.error("Timed out waiting for WebDriver /status ready");
process.exit(1);
