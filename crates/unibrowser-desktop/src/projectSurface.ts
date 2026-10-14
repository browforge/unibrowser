import type { DocumentViewerDto } from "./documentViewer";
import { t } from "./i18n";
import { renderWorkshop, type WorkshopViewer } from "./workshop";

export type ProjectSurfaceKind =
  | "empty"
  | "document"
  | "presentation"
  | "video"
  | "design"
  | "notes";

export function classifyProjectSurface(viewer: DocumentViewerDto): ProjectSurfaceKind {
  const nodes = viewer.project_nodes ?? [];
  const clips = viewer.project_clips ?? [];
  if (nodes.length === 0 && clips.length === 0) {
    return "empty";
  }
  const types = nodes.map((n) => n.node_type.toLowerCase());
  if (clips.length > 0 || types.some((ty) => ty.includes("video"))) {
    return "video";
  }
  if (types.some((ty) => ty.includes("presentation"))) {
    return "presentation";
  }
  if (types.some((ty) => ty.includes("canvas") || ty.includes("design"))) {
    return "design";
  }
  if (types.some((ty) => ty === "document" || ty.includes("frame"))) {
    if (types.every((ty) => ty === "document" || ty.includes("frame") || ty.includes("note"))) {
      return "document";
    }
  }
  if (types.some((ty) => ty.includes("note"))) {
    return "notes";
  }
  return "video";
}

export function isProjectDocumentSurface(viewer: DocumentViewerDto | null | undefined): boolean {
  return Boolean(viewer?.viewer === "unibrow" && classifyProjectSurface(viewer) === "document");
}

export async function activateProjectDocumentEditor(
  viewer: DocumentViewerDto,
  textEditorHost: HTMLElement,
): Promise<void> {
  const { syncDocumentEditorForTextTab } = await import("./documentEditorHost");
  const { setDocumentEditorMode } = await import("./documentEditorModes");
  const label = viewer.title?.trim() || "Document";
  const docPath = `${viewer.path.replace(/\/$/, "")}/assets/document.md`;
  const docContent = " ";
  document.body.dataset.cmAutoContext = "office";
  document.body.dataset.activeDocumentPath = docPath;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("set_embedded_editor_document_path", { path: docPath }).catch(() => undefined);
  setDocumentEditorMode("wysiwyg", false);
  await syncDocumentEditorForTextTab(textEditorHost, docPath, docContent);
  const { ensureProjectOfficeDocumentCard } = await import("./ui/textDocumentCard");
  await ensureProjectOfficeDocumentCard(label);
  const { refreshCompactBlockPickerLabel } = await import("./textCompactToolbar");
  refreshCompactBlockPickerLabel();
  window.dispatchEvent(new Event("unibrowser-project-document-ready"));
}

function renderEmptyProject(root: HTMLElement, viewer: DocumentViewerDto) {
  root.replaceChildren();
  root.className = "project-surface project-surface--empty";
  const card = document.createElement("section");
  card.className = "project-home-card";
  const title = document.createElement("h1");
  title.className = "project-home-title";
  title.textContent = viewer.title || t("project.emptyDefaultTitle");
  const lead = document.createElement("p");
  lead.className = "project-home-lead";
  lead.textContent = t("project.emptyLead");
  const list = document.createElement("div");
  list.className = "project-content-list";
  list.setAttribute("aria-label", t("project.contentListLabel"));
  const emptyRow = document.createElement("p");
  emptyRow.className = "project-content-empty";
  emptyRow.textContent = t("project.contentEmpty");
  list.appendChild(emptyRow);
  const actions = document.createElement("div");
  actions.className = "project-home-actions";
  const add = document.createElement("button");
  add.type = "button";
  add.className = "primary";
  add.textContent = t("project.addContent");
  add.onclick = () => {
    window.dispatchEvent(new CustomEvent("unibrowser-project-add-content"));
  };
  const template = document.createElement("button");
  template.type = "button";
  template.className = "quiet-button";
  template.textContent = t("project.newFromTemplate");
  template.onclick = () => {
    window.dispatchEvent(new CustomEvent("unibrowser-project-new-from-template"));
  };
  actions.append(add, template);
  card.append(title, lead, list, actions);
  root.appendChild(card);
}

type PresentationSlide = {
  id: string;
  title: string;
  subtitle: string;
  notes: string;
};

function renderPresentationStudio(root: HTMLElement, viewer: DocumentViewerDto) {
  root.replaceChildren();
  root.className = "project-surface project-surface--presentation";
  const nodes = viewer.project_nodes ?? [];
  const slideNodes = nodes.filter((n) => n.node_type.toLowerCase().includes("presentation"));
  const initialSlides: PresentationSlide[] =
    slideNodes.length > 0
      ? slideNodes.map((node, index) => ({
          id: node.id ?? `slide-${index + 1}`,
          title: "",
          subtitle: "",
          notes: "",
        }))
      : [{ id: "slide-1", title: "", subtitle: "", notes: "" }];

  let slides = initialSlides;
  let selectedIndex = 0;

  const layout = document.createElement("div");
  layout.className = "presentation-studio";

  const main = document.createElement("div");
  main.className = "presentation-main";

  const toolbar = document.createElement("div");
  toolbar.className = "presentation-toolbar";
  const addBtn = document.createElement("button");
  addBtn.type = "button";
  addBtn.className = "primary";
  addBtn.textContent = t("project.addSlide");
  const deleteBtn = document.createElement("button");
  deleteBtn.type = "button";
  deleteBtn.className = "quiet-button";
  deleteBtn.textContent = t("project.deleteSlide");

  const rail = document.createElement("aside");
  rail.className = "presentation-slides-rail";
  rail.setAttribute("aria-label", t("project.slidesRail"));
  const railHead = document.createElement("h2");
  railHead.textContent = t("project.slides");
  const slideList = document.createElement("ol");
  slideList.className = "presentation-slide-list";

  const stageArea = document.createElement("div");
  stageArea.className = "presentation-stage-area";

  const slideFrame = document.createElement("div");
  slideFrame.className = "presentation-slide-frame";
  const titleEl = document.createElement("div");
  titleEl.className = "presentation-slide-title";
  titleEl.contentEditable = "true";
  titleEl.setAttribute("role", "textbox");
  titleEl.setAttribute("aria-label", t("project.slideTitlePlaceholder"));
  titleEl.dataset.placeholder = t("project.slideTitlePlaceholder");
  const subtitleEl = document.createElement("div");
  subtitleEl.className = "presentation-slide-subtitle";
  subtitleEl.contentEditable = "true";
  subtitleEl.setAttribute("role", "textbox");
  subtitleEl.setAttribute("aria-label", t("project.slideSubtitlePlaceholder"));
  subtitleEl.dataset.placeholder = t("project.slideSubtitlePlaceholder");
  slideFrame.append(titleEl, subtitleEl);
  stageArea.appendChild(slideFrame);

  const notesPanel = document.createElement("section");
  notesPanel.className = "presentation-notes";
  const notesHead = document.createElement("h3");
  notesHead.textContent = t("project.speakerNotes");
  const notesBody = document.createElement("div");
  notesBody.className = "presentation-notes-body";
  notesBody.contentEditable = "true";
  notesBody.setAttribute("role", "textbox");
  notesBody.setAttribute("aria-label", t("project.speakerNotes"));
  notesPanel.append(notesHead, notesBody);

  const syncSlideFields = () => {
    const slide = slides[selectedIndex];
    if (!slide) {
      return;
    }
    titleEl.textContent = slide.title;
    subtitleEl.textContent = slide.subtitle;
    notesBody.textContent = slide.notes;
    if (!slide.title) {
      titleEl.dataset.empty = "1";
    } else {
      titleEl.removeAttribute("data-empty");
    }
    if (!slide.subtitle) {
      subtitleEl.dataset.empty = "1";
    } else {
      subtitleEl.removeAttribute("data-empty");
    }
    deleteBtn.disabled = slides.length <= 1;
    root.dataset.presentationSlideCount = String(slides.length);
    window.dispatchEvent(new CustomEvent("unibrowser-presentation-slides-changed"));
  };

  const persistFromDom = () => {
    const slide = slides[selectedIndex];
    if (!slide) {
      return;
    }
    slide.title = titleEl.textContent?.trim() ?? "";
    slide.subtitle = subtitleEl.textContent?.trim() ?? "";
    slide.notes = notesBody.textContent?.trim() ?? "";
  };

  const renderRail = () => {
    slideList.replaceChildren();
    slides.forEach((_slide, index) => {
      const li = document.createElement("li");
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "presentation-slide-thumb";
      if (index === selectedIndex) {
        btn.classList.add("selected");
      }
      btn.setAttribute("aria-label", t("project.slideDefault", { n: String(index + 1) }));
      const thumbCanvas = document.createElement("span");
      thumbCanvas.className = "presentation-slide-thumb-canvas";
      const num = document.createElement("span");
      num.className = "presentation-slide-thumb-num";
      num.textContent = String(index + 1);
      thumbCanvas.appendChild(num);
      btn.appendChild(thumbCanvas);
      btn.onclick = () => {
        persistFromDom();
        selectedIndex = index;
        syncSlideFields();
        renderRail();
      };
      li.appendChild(btn);
      slideList.appendChild(li);
    });
  };

  addBtn.onclick = () => {
    persistFromDom();
    const id = `slide-${slides.length + 1}`;
    slides = [...slides, { id, title: "", subtitle: "", notes: "" }];
    selectedIndex = slides.length - 1;
    syncSlideFields();
    renderRail();
    titleEl.focus();
  };

  deleteBtn.onclick = () => {
    if (slides.length <= 1) {
      return;
    }
    persistFromDom();
    slides = slides.filter((_, i) => i !== selectedIndex);
    selectedIndex = Math.min(selectedIndex, slides.length - 1);
    syncSlideFields();
    renderRail();
  };

  titleEl.addEventListener("input", () => {
    persistFromDom();
    syncSlideFields();
  });
  subtitleEl.addEventListener("input", () => {
    persistFromDom();
    syncSlideFields();
  });
  notesBody.addEventListener("input", () => {
    persistFromDom();
  });

  toolbar.append(addBtn, deleteBtn);
  rail.append(railHead, slideList);
  main.append(toolbar, stageArea, notesPanel);
  layout.append(rail, main);
  root.appendChild(layout);
  syncSlideFields();
  renderRail();
}

function renderDesignSpace(root: HTMLElement, viewer: DocumentViewerDto) {
  root.replaceChildren();
  root.className = "project-surface project-surface--design";
  const shell = document.createElement("div");
  shell.className = "design-space-studio";
  const toolbar = document.createElement("div");
  toolbar.className = "design-space-toolbar";
  toolbar.textContent = t("project.designToolbar");
  const canvas = document.createElement("div");
  canvas.className = "design-space-canvas";
  canvas.setAttribute("role", "img");
  canvas.setAttribute("aria-label", t("project.designCanvas"));
  const label = viewer.project_nodes?.[0]?.label ?? t("project.canvasDefault");
  const tag = document.createElement("span");
  tag.className = "design-space-canvas-label";
  tag.textContent = label;
  canvas.appendChild(tag);
  shell.append(toolbar, canvas);
  root.appendChild(shell);
}

function renderNotesBoard(root: HTMLElement, viewer: DocumentViewerDto) {
  root.replaceChildren();
  root.className = "project-surface project-surface--notes";
  const board = document.createElement("div");
  board.className = "notes-board";
  const nodes = viewer.project_nodes ?? [];
  nodes.forEach((node) => {
    const card = document.createElement("article");
    card.className = "notes-board-card";
    card.style.setProperty("--note-x", `${node.x}px`);
    card.style.setProperty("--note-y", `${node.y}px`);
    const h = document.createElement("h3");
    h.textContent = node.label;
    const body = document.createElement("p");
    body.textContent = t("project.noteBodyPlaceholder");
    card.append(h, body);
    board.appendChild(card);
  });
  root.appendChild(board);
}

export async function renderProjectSurface(
  viewer: DocumentViewerDto,
  root: HTMLElement,
  options?: { hideViewerTitle?: boolean },
): Promise<ProjectSurfaceKind> {
  const kind = classifyProjectSurface(viewer);
  if (kind !== "document") {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("set_embedded_editor_document_path", { path: null }).catch(() => undefined);
  }
  if (options?.hideViewerTitle) {
    root.closest(".document-viewer")?.querySelector(".viewer-title")?.classList.add("hidden");
  }
  const viewerHost = root.closest(".document-viewer-host");
  viewerHost?.classList.add("document-viewer-host--project-surface");
  switch (kind) {
    case "empty":
      renderEmptyProject(root, viewer);
      break;
    case "presentation":
      renderPresentationStudio(root, viewer);
      break;
    case "design":
      renderDesignSpace(root, viewer);
      break;
    case "notes":
      renderNotesBoard(root, viewer);
      break;
    case "video":
      resetWorkshopChrome(root);
      await renderWorkshop(viewer as WorkshopViewer, root);
      break;
    case "document":
      break;
  }
  return kind;
}

function resetWorkshopChrome(root: HTMLElement) {
  root.closest(".document-viewer")?.querySelector(".viewer-title")?.classList.remove("hidden");
}
