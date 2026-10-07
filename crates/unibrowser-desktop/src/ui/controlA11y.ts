import type { ResolvedControlDto } from "./commandControls";

export function controlA11yState(control: ResolvedControlDto) {
  const state = control.capability.state;
  const actionable = state === "supported" || state === "emulated";
  const mixed =
    control.selection === "mixed" || control.selection === "mixed_value";
  const reason =
    state === "unsupported" || state === "locked"
      ? control.capability.reason
      : state === "emulated"
        ? control.capability.note
        : null;
  const stateLabel =
    state === "emulated" ? "emulated" : actionable ? null : "unavailable";
  return {
    actionable,
    mixed,
    reason,
    ariaDisabled: actionable ? null : "true",
    ariaChecked: mixed
      ? "mixed"
      : control.control_type === "toggle"
        ? control.selection === "on"
          ? "true"
          : "false"
        : null,
    announcement: [
      control.label,
      stateLabel,
      reason,
      mixed ? "mixed selection" : null,
    ]
      .filter(Boolean)
      .join(", "),
  };
}
