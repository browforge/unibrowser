/** Non-modal confirm / prompt / pick-list overlays (principles: no window.confirm/prompt/alert). */

export interface PickListItem {
  id: string;
  label: string;
  detail?: string;
}

let layer: HTMLElement | null = null;

function ensureLayer(): HTMLElement {
  if (!layer) {
    layer = document.getElementById("inline-ui-layer");
  }
  if (!layer) {
    layer = document.createElement("div");
    layer.id = "inline-ui-layer";
    layer.className = "inline-ui-layer hidden";
    document.body.appendChild(layer);
  }
  return layer;
}

function closeLayer() {
  const el = ensureLayer();
  el.classList.add("hidden");
  el.classList.remove("inline-ui-layer--pick");
  el.innerHTML = "";
}

export function showInlineConfirm(message: string, confirmLabel = "OK", cancelLabel = "Cancel"): Promise<boolean> {
  const el = ensureLayer();
  el.classList.remove("hidden");
  el.innerHTML = "";
  const card = document.createElement("div");
  card.className = "inline-ui-card";
  const p = document.createElement("p");
  p.textContent = message;
  card.appendChild(p);
  const actions = document.createElement("div");
  actions.className = "inline-ui-actions";
  return new Promise((resolve) => {
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = cancelLabel;
    cancel.onclick = () => {
      closeLayer();
      resolve(false);
    };
    const ok = document.createElement("button");
    ok.type = "button";
    ok.textContent = confirmLabel;
    ok.onclick = () => {
      closeLayer();
      resolve(true);
    };
    actions.appendChild(cancel);
    actions.appendChild(ok);
    card.appendChild(actions);
    el.appendChild(card);
  });
}

export function showInlinePrompt(
  message: string,
  defaultValue = "",
  submitLabel = "OK",
): Promise<string | null> {
  const el = ensureLayer();
  el.classList.remove("hidden");
  el.innerHTML = "";
  const card = document.createElement("div");
  card.className = "inline-ui-card";
  const p = document.createElement("p");
  p.textContent = message;
  const input = document.createElement("input");
  input.type = "text";
  input.value = defaultValue;
  card.appendChild(p);
  card.appendChild(input);
  const actions = document.createElement("div");
  actions.className = "inline-ui-actions";
  return new Promise((resolve) => {
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = "Cancel";
    cancel.onclick = () => {
      closeLayer();
      resolve(null);
    };
    const ok = document.createElement("button");
    ok.type = "button";
    ok.textContent = submitLabel;
    ok.onclick = () => {
      closeLayer();
      resolve(input.value.trim());
    };
    actions.appendChild(cancel);
    actions.appendChild(ok);
    card.appendChild(actions);
    el.appendChild(card);
    input.focus();
  });
}

export function showPickList(title: string, items: PickListItem[]): Promise<PickListItem | null> {
  const el = ensureLayer();
  el.classList.remove("hidden");
  el.classList.add("inline-ui-layer--pick");
  el.innerHTML = "";
  const card = document.createElement("div");
  card.className = "inline-ui-card inline-ui-card--list";
  const h = document.createElement("h3");
  h.textContent = title;
  card.appendChild(h);
  const list = document.createElement("ul");
  list.className = "inline-ui-pick-list";
  return new Promise((resolve) => {
    for (const item of items) {
      const li = document.createElement("li");
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "inline-ui-pick-item";
      btn.textContent = item.label;
      if (item.detail) {
        btn.title = item.detail;
      }
      btn.onclick = () => {
        closeLayer();
        resolve(item);
      };
      li.appendChild(btn);
      list.appendChild(li);
    }
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = "Cancel";
    cancel.className = "inline-ui-cancel";
    cancel.onclick = () => {
      closeLayer();
      resolve(null);
    };
    card.appendChild(list);
    card.appendChild(cancel);
    el.appendChild(card);
  });
}

export function showInlineAlert(message: string) {
  void showInlineConfirm(message, "OK", "Dismiss");
}
