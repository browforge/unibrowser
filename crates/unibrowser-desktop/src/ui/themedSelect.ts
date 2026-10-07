const OPTION_CLASS = "ub-select-option";
const LIST_CLASS = "ub-select-list";

let openList: HTMLUListElement | null = null;
let openSelect: HTMLSelectElement | null = null;
let highlightIndex = 0;

function closeThemedList() {
  openList?.remove();
  openList = null;
  openSelect = null;
}

function optionElements(): HTMLButtonElement[] {
  if (!openList) {
    return [];
  }
  return Array.from(openList.querySelectorAll<HTMLButtonElement>(`.${OPTION_CLASS}`));
}

function setHighlight(index: number) {
  const options = optionElements();
  if (options.length === 0) {
    return;
  }
  highlightIndex = (index + options.length) % options.length;
  options.forEach((option, i) => {
    option.classList.toggle("is-active", i === highlightIndex);
    option.tabIndex = -1;
  });
  const active = options[highlightIndex];
  if (active && openSelect) {
    if (!active.id) {
      active.id = `ub-select-opt-${highlightIndex}`;
    }
    openSelect.setAttribute("aria-activedescendant", active.id);
  }
}

function applyOption(select: HTMLSelectElement, value: string) {
  if (select.disabled) {
    return;
  }
  const previous = select.value;
  select.value = value;
  if (previous !== value) {
    select.dispatchEvent(new Event("input", { bubbles: true }));
    select.dispatchEvent(new Event("change", { bubbles: true }));
  }
  closeThemedList();
  select.focus();
}

function openThemedList(select: HTMLSelectElement) {
  if (select.disabled) {
    return;
  }
  if (openSelect === select && openList) {
    closeThemedList();
    select.focus();
    return;
  }
  closeThemedList();
  const list = document.createElement("ul");
  list.className = LIST_CLASS;
  list.setAttribute("role", "listbox");
  list.setAttribute("aria-label", select.getAttribute("aria-label") || "Options");
  const rect = select.getBoundingClientRect();
  list.style.left = `${Math.round(rect.left)}px`;
  list.style.top = `${Math.round(rect.bottom + 2)}px`;
  list.style.minWidth = `${Math.round(rect.width)}px`;

  Array.from(select.options).forEach((option, index) => {
    const item = document.createElement("li");
    item.setAttribute("role", "presentation");
    const button = document.createElement("button");
    button.type = "button";
    button.className = OPTION_CLASS;
    button.id = `ub-select-opt-${index}`;
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", option.selected ? "true" : "false");
    button.dataset.value = option.value;
    button.textContent = option.textContent ?? option.value;
    if (option.selected) {
      button.classList.add("is-selected");
      highlightIndex = index;
    }
    button.addEventListener("mousedown", (event) => {
      event.preventDefault();
    });
    button.addEventListener("click", () => {
      applyOption(select, option.value);
    });
    item.appendChild(button);
    list.appendChild(item);
  });

  document.body.appendChild(list);
  openList = list;
  openSelect = select;
  setHighlight(highlightIndex);
}

export function bindThemedSelect(select: HTMLSelectElement) {
  if (select.dataset.ubThemed === "1") {
    return;
  }
  select.dataset.ubThemed = "1";
  select.setAttribute("aria-haspopup", "listbox");

  select.addEventListener("mousedown", (event) => {
    if (select.disabled) {
      return;
    }
    event.preventDefault();
    select.focus();
    openThemedList(select);
  });

  select.addEventListener("keydown", (event) => {
    if (select.disabled) {
      return;
    }
    const listOpen = openSelect === select && openList;
    if (
      event.key === " " ||
      event.key === "Enter" ||
      event.key === "ArrowDown" ||
      event.key === "ArrowUp"
    ) {
      event.preventDefault();
      if (!listOpen) {
        openThemedList(select);
        if (event.key === "ArrowUp") {
          setHighlight(select.options.length - 1);
        }
        return;
      }
      if (event.key === "Enter" || event.key === " ") {
        const current = optionElements()[highlightIndex];
        if (current?.dataset.value != null) {
          applyOption(select, current.dataset.value);
        }
        return;
      }
      setHighlight(highlightIndex + (event.key === "ArrowUp" ? -1 : 1));
      return;
    }
    if (event.key === "Escape" && listOpen) {
      event.preventDefault();
      closeThemedList();
      select.focus();
    }
  });

  select.addEventListener("blur", () => {
    window.setTimeout(() => {
      if (!openList) {
        return;
      }
      const next = document.activeElement;
      if (next === openSelect || openList.contains(next)) {
        return;
      }
      closeThemedList();
    }, 0);
  });
}

export function enhanceSelectsIn(root: ParentNode = document) {
  root.querySelectorAll("select").forEach((select) => {
    bindThemedSelect(select);
  });
}

export function initThemedSelects() {
  if (!document.body) {
    document.addEventListener("DOMContentLoaded", () => initThemedSelects(), {
      once: true,
    });
    return;
  }
  enhanceSelectsIn(document);
  const observer = new MutationObserver((records) => {
    for (const record of records) {
      record.addedNodes.forEach((node) => {
        if (!(node instanceof HTMLElement)) {
          return;
        }
        if (node instanceof HTMLSelectElement) {
          bindThemedSelect(node);
        }
        enhanceSelectsIn(node);
      });
    }
  });
  observer.observe(document.body, { childList: true, subtree: true });
  document.addEventListener("mousedown", (event) => {
    if (!openList || !openSelect) {
      return;
    }
    const target = event.target;
    if (!(target instanceof Node)) {
      return;
    }
    if (openList.contains(target) || openSelect.contains(target)) {
      return;
    }
    closeThemedList();
  });
}
