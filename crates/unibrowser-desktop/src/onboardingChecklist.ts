import { invoke } from "@tauri-apps/api/core";
import enNz from "./i18n/en-NZ.json";
import { t } from "./i18n";

type I18nKey = keyof typeof enNz;

export const CHECKLIST_STEP_IDS = [
  "palette",
  "browse_folder",
  "open_file",
  "split_view",
  "settings_tour",
] as const;

export type ChecklistStepId = (typeof CHECKLIST_STEP_IDS)[number];

const STEP_I18N: Record<ChecklistStepId, I18nKey> = {
  palette: "onboarding.stepPalette",
  browse_folder: "onboarding.stepBrowse",
  open_file: "onboarding.stepOpenFile",
  split_view: "onboarding.stepSplit",
  settings_tour: "onboarding.stepSettings",
};

export async function markChecklistStep(stepId: ChecklistStepId) {
  try {
    await invoke("complete_onboarding_checklist_step", { step_id: stepId });
  } catch {
    /* prefs unavailable */
  }
}

export function appendOnboardingChecklistSection(
  root: HTMLElement,
  prefs: {
    onboarding_checklist_skip_forever: boolean;
    onboarding_checklist_completed: string[];
  },
) {
  if (prefs.onboarding_checklist_skip_forever) {
    return;
  }

  const section = document.createElement("section");
  section.className = "settings-section onboarding-checklist";
  const heading = document.createElement("h3");
  heading.textContent = t("onboarding.checklistHeading");
  section.appendChild(heading);

  const lead = document.createElement("p");
  lead.className = "viewer-note";
  lead.textContent = t("onboarding.checklistLead");
  section.appendChild(lead);

  const list = document.createElement("ul");
  list.className = "onboarding-checklist-list";

  const completed = new Set(prefs.onboarding_checklist_completed);

  CHECKLIST_STEP_IDS.forEach((id) => {
    const li = document.createElement("li");
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.id = `onboarding-step-${id}`;
    cb.checked = completed.has(id);
    cb.onchange = async () => {
      if (!cb.checked) {
        cb.checked = true;
        return;
      }
      await markChecklistStep(id);
    };
    const label = document.createElement("label");
    label.htmlFor = cb.id;
    label.textContent = t(STEP_I18N[id]);
    li.appendChild(cb);
    li.appendChild(label);
    list.appendChild(li);
  });

  section.appendChild(list);

  const skipBtn = document.createElement("button");
  skipBtn.type = "button";
  skipBtn.className = "settings-skip-checklist";
  skipBtn.textContent = t("onboarding.skipForever");
  skipBtn.onclick = async () => {
    await invoke("set_onboarding_checklist_skip_forever", { skip: true });
    section.remove();
  };
  section.appendChild(skipBtn);

  root.appendChild(section);
}
