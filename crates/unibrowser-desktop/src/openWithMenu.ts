import { invoke } from "@tauri-apps/api/core";
import type { OpenWithOption } from "./capabilityProviders";
import { sortOpenWithOptions } from "./capabilityProviders";
import { t } from "./i18n";
import { showErrorToast } from "./toast";
export async function showOpenWithMenu<T>(
  path: string,
  anchor: HTMLElement,
  onApplied: (dto: T) => void,
) {
  let options: OpenWithOption[];
  try {
    options = await invoke<OpenWithOption[]>("list_open_with_providers", { path });
  } catch (err) {
    showErrorToast(String(err));
    return;
  }
  if (!options.length) {
    showErrorToast(t("openWith.none"));
    return;
  }
  const menu = document.createElement("div");
  menu.className = "open-with-menu";
  menu.setAttribute("role", "menu");
  const title = document.createElement("p");
  title.className = "open-with-menu-title";
  title.textContent = t("openWith.title");
  menu.appendChild(title);
  for (const option of sortOpenWithOptions(options)) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "open-with-menu-item" + (option.active ? " active" : "");
    btn.textContent = `${option.mod_title} (${option.mod_name})`;
    btn.onclick = () => {
      menu.remove();
      void invoke<T>("open_path_with_provider", {
        path,
        modName: option.mod_name,
        remember: "path",
      })
        .then(onApplied)
        .catch((e) => showErrorToast(String(e)));
    };
    menu.appendChild(btn);
  }
  const rect = anchor.getBoundingClientRect();
  menu.style.position = "fixed";
  menu.style.left = `${rect.left}px`;
  menu.style.top = `${rect.bottom + 4}px`;
  const dismiss = (ev: MouseEvent) => {
    if (!menu.contains(ev.target as Node)) {
      menu.remove();
      document.removeEventListener("click", dismiss, true);
    }
  };
  document.body.appendChild(menu);
  document.addEventListener("click", dismiss, true);
}
