/** Notification history (last 20) + drawer from the status bar; toasts still show in chrome. */

import { copyText } from "./clipboard";
import { t } from "./i18n";

export type NotificationKind = "error" | "info";

export interface NotificationEntry {
  id: number;
  atMs: number;
  message: string;
  kind: NotificationKind;
  details?: string;
}

const MAX_HISTORY = 20;
const history: NotificationEntry[] = [];
let nextId = 1;

let drawerEl: HTMLElement | null = null;
let listEl: HTMLElement | null = null;
let toggleBtn: HTMLButtonElement | null = null;
let drawerOpen = false;

function formatTime(atMs: number): string {
  const d = new Date(atMs);
  return d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

function unreadCount(): number {
  return history.length;
}

function updateToggleLabel() {
  if (!toggleBtn) {
    return;
  }
  const n = unreadCount();
  toggleBtn.textContent = n > 0 ? `! ${n}` : "";
  toggleBtn.title = n > 0
    ? t("notifications.buttonWithCount", { count: n })
    : t("notifications.buttonHint");
  toggleBtn.setAttribute(
    "aria-label",
    n > 0 ? t("notifications.buttonWithCount", { count: n }) : t("notifications.button"),
  );
  toggleBtn.classList.toggle("hidden", n === 0);
  toggleBtn.classList.toggle("status-notifications-has-items", n > 0);
}

function renderDrawerList() {
  if (!listEl) {
    return;
  }
  listEl.innerHTML = "";
  if (!history.length) {
    const empty = document.createElement("p");
    empty.className = "notification-drawer-empty";
    empty.textContent = t("notifications.empty");
    listEl.appendChild(empty);
    return;
  }
  const ul = document.createElement("ul");
  ul.className = "notification-drawer-list";
  for (const entry of history) {
    const li = document.createElement("li");
    li.className = `notification-drawer-item notification-${entry.kind}`;
    const head = document.createElement("div");
    head.className = "notification-drawer-item-head";
    const time = document.createElement("time");
    time.dateTime = new Date(entry.atMs).toISOString();
    time.textContent = formatTime(entry.atMs);
    head.appendChild(time);
    if (entry.details && entry.kind === "error") {
      const copyBtn = document.createElement("button");
      copyBtn.type = "button";
      copyBtn.className = "notification-drawer-copy";
      copyBtn.textContent = t("toast.copyDetails");
      copyBtn.onclick = () => {
        void copyText(entry.details ?? "", () => {
          copyBtn.textContent = t("toast.copied");
        });
      };
      head.appendChild(copyBtn);
    }
    li.appendChild(head);
    const msg = document.createElement("p");
    msg.className = "notification-drawer-message";
    msg.textContent = entry.message;
    li.appendChild(msg);
    ul.appendChild(li);
  }
  listEl.appendChild(ul);
}

function setDrawerOpen(open: boolean) {
  drawerOpen = open;
  drawerEl?.classList.toggle("hidden", !open);
  toggleBtn?.setAttribute("aria-expanded", open ? "true" : "false");
  if (open) {
    renderDrawerList();
  }
}

export function pushNotification(
  message: string,
  kind: NotificationKind,
  details?: string,
): NotificationEntry {
  const entry: NotificationEntry = {
    id: nextId++,
    atMs: Date.now(),
    message,
    kind,
    details: details?.trim() || undefined,
  };
  history.unshift(entry);
  while (history.length > MAX_HISTORY) {
    history.pop();
  }
  updateToggleLabel();
  if (drawerOpen) {
    renderDrawerList();
  }
  return entry;
}

export function initNotificationCenter(statusBar: HTMLElement) {
  toggleBtn = document.createElement("button");
  toggleBtn.type = "button";
  toggleBtn.id = "status-notifications";
  toggleBtn.className = "status-notifications-btn";
  toggleBtn.setAttribute("aria-expanded", "false");
  toggleBtn.title = t("notifications.buttonHint");
  toggleBtn.onclick = () => setDrawerOpen(!drawerOpen);
  updateToggleLabel();

  const hint = statusBar.querySelector("#status-hint");
  if (hint) {
    statusBar.insertBefore(toggleBtn, hint);
  } else {
    statusBar.appendChild(toggleBtn);
  }

  drawerEl = document.createElement("aside");
  drawerEl.id = "notification-drawer";
  drawerEl.className = "notification-drawer hidden";
  drawerEl.setAttribute("aria-label", t("notifications.drawerLabel"));
  const head = document.createElement("div");
  head.className = "notification-drawer-head";
  const title = document.createElement("h2");
  title.textContent = t("notifications.title");
  head.appendChild(title);
  const clearBtn = document.createElement("button");
  clearBtn.type = "button";
  clearBtn.className = "notification-drawer-clear";
  clearBtn.textContent = t("notifications.clear");
  clearBtn.onclick = () => {
    history.length = 0;
    updateToggleLabel();
    renderDrawerList();
  };
  head.appendChild(clearBtn);
  const closeBtn = document.createElement("button");
  closeBtn.type = "button";
  closeBtn.className = "notification-drawer-close";
  closeBtn.textContent = t("notifications.close");
  closeBtn.onclick = () => setDrawerOpen(false);
  head.appendChild(closeBtn);
  drawerEl.appendChild(head);
  listEl = document.createElement("div");
  listEl.className = "notification-drawer-body";
  drawerEl.appendChild(listEl);
  document.body.appendChild(drawerEl);

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && drawerOpen) {
      setDrawerOpen(false);
    }
  });
}
