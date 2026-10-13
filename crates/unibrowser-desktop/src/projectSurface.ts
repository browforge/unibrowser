import { invoke } from "@tauri-apps/api/core";
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
  const { syncTextDocumentCard } = await import("./ui/textDocumentCard");
  const label = viewer.title?.trim() || "Document";
  const docPath = `${viewer.path.replace(/\/$/, "")}/assets/document.md`;
  const docContent = " ";
  document.body.dataset.cmAutoContext = "office";
  document.body.dataset.activeDocumentPath = docPath;
  setDocumentEditorMode("wysiwyg", false);
  await syncDocumentEditorForTextTab(textEditorHost, docPath, docContent);
  syncTextDocumentCard(true, label);
  await invoke("select_command_manager_tab", { tabId: "office" }).catch(() => undefined);
  const { refreshCompactBlockPickerLabel } = await import("./textCompactToolbar");
  refreshCompactBlockPickerLabel();
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

function renderPresentationStudio(root: HTMLElement, viewer: DocumentViewerDto) {
  root.replaceChildren();
  root.className = "project-surface project-surface--presentation";
  const nodes = viewer.project_nodes ?? [];
  const slides = nodes.filter((n) => n.node_type.toLowerCase().includes("presentation"));
  const notes = nodes.filter((n) => n.node_type.toLowerCase().includes("note"));
  const layout = document.createElement("div");
  layout.className = "presentation-studio";
  const rail = document.createElement("aside");
  rail.className = "presentation-slides-rail";
  rail.setAttribute("aria-label", t("project.slidesRail"));
  const railHead = document.createElement("h2");
  railHead.textContent = t("project.slides");
  const slideList = document.createElement("ol");
  slideList.className = "presentation-slide-list";
  if (slides.length) {
    slides.forEach((node, index) => {
      const li = document.createElement("li");
      li.className = index === 0 ? "selected" : "";
      li.textContent = node.label || t("project.slideDefault", { n: String(index + 1) });
      slideList.appendChild(li);
    });
  } else {
    const li = document.createElement("li");
    li.className = "selected";
    li.textContent = t("project.slideDefault", { n: "1" });
    slideList.appendChild(li);
  }
  rail.append(railHead, slideList);
  const stage = document.createElement("section");
  stage.className = "presentation-stage";
  const stageTitle = document.createElement("h2");
  stageTitle.textContent = viewer.title;
  const canvas = document.createElement("div");
  canvas.className = "presentation-canvas";
  canvas.textContent = t("project.presentationCanvasHint");
  stage.append(stageTitle, canvas);
  if (notes.length) {
    const notesPanel = document.createElement("aside");
    notesPanel.className = "presentation-notes";
    const notesHead = document.createElement("h3");
    notesHead.textContent = notes[0]?.label || t("project.speakerNotes");
    const notesBody = document.createElement("p");
    notesBody.textContent = t("project.speakerNotesEmpty");
    notesPanel.append(notesHead, notesBody);
    layout.append(rail, stage, notesPanel);
  } else {
    layout.append(rail, stage);
  }
  root.appendChild(layout);
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
  if (options?.hideViewerTitle) {
    root.closest(".document-viewer")?.querySelector(".viewer-title")?.classList.add("hidden");
  }
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
