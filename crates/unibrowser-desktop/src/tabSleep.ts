/** UI-only background tab sleep (no webview teardown in blitz). */

export const TAB_SLEEP_IDLE_MS = 10 * 60 * 1000;

const lastActiveAt = new Map<number, number>();

export function touchTabActivity(tabId: number) {
  lastActiveAt.set(tabId, Date.now());
}

export function isTabSleeping(
  tabId: number,
  isHttp: boolean,
  isActive: boolean,
  now = Date.now(),
): boolean {
  if (!isHttp || isActive) {
    return false;
  }
  const last = lastActiveAt.get(tabId);
  if (last === undefined) {
    lastActiveAt.set(tabId, now);
    return false;
  }
  return now - last >= TAB_SLEEP_IDLE_MS;
}

export function pruneTabActivity(knownTabIds: number[]) {
  const keep = new Set(knownTabIds);
  for (const id of lastActiveAt.keys()) {
    if (!keep.has(id)) {
      lastActiveAt.delete(id);
    }
  }
}
