export interface EncodeProgressState {
  active: boolean;
  percent: number;
  stage: string;
}

let state: EncodeProgressState = { active: false, percent: 0, stage: "" };

export function setEncodeProgress(percent: number | null, stage = "") {
  if (percent === null) {
    state = { active: false, percent: 0, stage: "" };
  } else {
    state = { active: true, percent, stage };
  }
}

export function getEncodeProgress(): EncodeProgressState {
  return state;
}
