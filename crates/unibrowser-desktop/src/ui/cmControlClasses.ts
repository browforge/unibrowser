export type CapabilityState =
  | "supported"
  | "emulated"
  | "unsupported"
  | "hidden_by_user"
  | "locked";

export function capabilityActionableForState(state: CapabilityState): boolean {
  return state === "supported" || state === "emulated";
}

/** CSS class names for a command-manager control button (parity with renderResolvedControl). */
export function uiCommandChromeClassList(opts: {
  controlType: string;
  capabilityState: CapabilityState;
  actionable: boolean;
  emulated: boolean;
  locked: boolean;
  mixed: boolean;
  modControl: boolean;
}): string[] {
  const unsupported = opts.capabilityState === "unsupported";
  const supportedChrome =
    opts.capabilityState === "supported" || opts.capabilityState === "emulated";
  return [
    "ui-command",
    opts.controlType === "toggle" ? "toggle" : "",
    opts.modControl ? "cm-mod-control" : "",
    !opts.actionable && !unsupported ? "disabled" : "",
    unsupported ? "unsupported" : "",
    supportedChrome ? "cm-cap-supported" : "",
    opts.emulated ? "emulated" : "",
    opts.locked ? "locked" : "",
    opts.mixed ? "mixed" : "",
  ].filter(Boolean);
}
