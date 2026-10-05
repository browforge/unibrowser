import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

const TOUR_STEPS: Array<{ titleKey: "onboarding.tourStep1Title"; bodyKey: "onboarding.tourStep1Body" } | {
  titleKey: "onboarding.tourStep2Title";
  bodyKey: "onboarding.tourStep2Body";
}> = [
  {
    titleKey: "onboarding.tourStep1Title",
    bodyKey: "onboarding.tourStep1Body",
  },
  {
    titleKey: "onboarding.tourStep2Title",
    bodyKey: "onboarding.tourStep2Body",
  },
];

let stepIndex = 0;

export function hideOnboardingTour() {
  const el = document.getElementById("onboarding-tour");
  if (el) {
    el.classList.add("hidden");
  }
}

export function showOnboardingTour(visible: boolean) {
  const el = document.getElementById("onboarding-tour");
  if (!el) {
    return;
  }
  if (!visible) {
    el.classList.add("hidden");
    return;
  }
  el.classList.remove("hidden");
  renderStep(el);
}

function renderStep(root: HTMLElement) {
  const step = TOUR_STEPS[stepIndex] ?? TOUR_STEPS[0];
  root.innerHTML = "";
  const card = document.createElement("div");
  card.className = "onboarding-tour-card";
  card.setAttribute("role", "region");
  card.setAttribute("aria-label", t("onboarding.tourAria"));

  const title = document.createElement("h2");
  title.textContent = t(step.titleKey);
  card.appendChild(title);

  const body = document.createElement("p");
  body.textContent = t(step.bodyKey);
  card.appendChild(body);

  const actions = document.createElement("div");
  actions.className = "onboarding-tour-actions";

  const skipBtn = document.createElement("button");
  skipBtn.type = "button";
  skipBtn.className = "onboarding-tour-skip";
  skipBtn.textContent = t("onboarding.tourSkip");
  skipBtn.onclick = async () => {
    try {
      await invoke("skip_onboarding_tour_forever");
    } catch {
      /* prefs */
    }
    hideOnboardingTour();
  };
  actions.appendChild(skipBtn);

  if (stepIndex < TOUR_STEPS.length - 1) {
    const nextBtn = document.createElement("button");
    nextBtn.type = "button";
    nextBtn.textContent = t("onboarding.tourNext");
    nextBtn.onclick = () => {
      stepIndex += 1;
      renderStep(root);
    };
    actions.appendChild(nextBtn);
  } else {
    const doneBtn = document.createElement("button");
    doneBtn.type = "button";
    doneBtn.textContent = t("onboarding.tourDone");
    doneBtn.onclick = () => {
      void skipBtn.click();
    };
    actions.appendChild(doneBtn);
  }

  card.appendChild(actions);
  root.appendChild(card);
}
