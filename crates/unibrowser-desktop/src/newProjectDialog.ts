import { t } from "./i18n";

export type ProjectTemplateId = "blank" | "notes" | "video";

export function openNewProjectDialog(): Promise<{
  name: string;
  template: ProjectTemplateId;
} | null> {
  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "new-project-overlay";
    const panel = document.createElement("div");
    panel.className = "new-project-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");
    panel.setAttribute("aria-labelledby", "new-project-title");

    const title = document.createElement("h2");
    title.id = "new-project-title";
    title.textContent = t("project.newTitle");
    panel.appendChild(title);

    const nameLabel = document.createElement("label");
    nameLabel.textContent = t("project.nameLabel");
    const nameInput = document.createElement("input");
    nameInput.type = "text";
    nameInput.value = "MyProject";
    nameInput.setAttribute("aria-label", t("project.nameLabel"));
    nameLabel.appendChild(nameInput);
    panel.appendChild(nameLabel);

    const tplLegend = document.createElement("p");
    tplLegend.className = "viewer-note";
    tplLegend.textContent = t("project.templateLead");
    panel.appendChild(tplLegend);

    const templates: { id: ProjectTemplateId; labelKey: string; hintKey: string }[] = [
      { id: "blank", labelKey: "project.templateBlank", hintKey: "project.templateBlankHint" },
      { id: "notes", labelKey: "project.templateNotes", hintKey: "project.templateNotesHint" },
      { id: "video", labelKey: "project.templateVideo", hintKey: "project.templateVideoHint" },
    ];
    let selected: ProjectTemplateId = "notes";
    const group = document.createElement("div");
    group.className = "new-project-templates";
    group.setAttribute("role", "radiogroup");
    group.setAttribute("aria-label", t("project.templateLead"));
    templates.forEach((tpl) => {
      const row = document.createElement("label");
      row.className = "new-project-template-row";
      const radio = document.createElement("input");
      radio.type = "radio";
      radio.name = "project-template";
      radio.value = tpl.id;
      radio.checked = tpl.id === selected;
      radio.onchange = () => {
        selected = tpl.id;
      };
      const strong = document.createElement("strong");
      strong.textContent = t(tpl.labelKey as "project.templateBlank");
      const hint = document.createElement("span");
      hint.textContent = ` — ${t(tpl.hintKey as "project.templateBlankHint")}`;
      row.appendChild(radio);
      row.appendChild(strong);
      row.appendChild(hint);
      group.appendChild(row);
    });
    panel.appendChild(group);

    const actions = document.createElement("div");
    actions.className = "new-project-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = t("project.cancel");
    const create = document.createElement("button");
    create.type = "button";
    create.className = "primary";
    create.textContent = t("project.create");
    const close = (result: { name: string; template: ProjectTemplateId } | null) => {
      overlay.remove();
      resolve(result);
    };
    cancel.onclick = () => close(null);
    create.onclick = () => {
      const name = nameInput.value.trim();
      if (!name) {
        nameInput.focus();
        return;
      }
      close({ name, template: selected });
    };
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) {
        close(null);
      }
    });
    actions.appendChild(cancel);
    actions.appendChild(create);
    panel.appendChild(actions);
    overlay.appendChild(panel);
    document.body.appendChild(overlay);
    nameInput.focus();
    nameInput.select();
  });
}
