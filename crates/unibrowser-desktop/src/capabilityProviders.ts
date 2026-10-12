export type OpenWithOption = {
  mod_name: string;
  mod_title: string;
  active: boolean;
};

export function formatCapabilityChip(capabilityId: string): string {
  const tail = capabilityId.replace(/^unibrowser\.capability\./, "");
  return tail.replace(/_/g, " ");
}

export function pickActiveOpenWith(options: OpenWithOption[]): OpenWithOption | null {
  return options.find((o) => o.active) ?? options[0] ?? null;
}

export function sortOpenWithOptions(options: OpenWithOption[]): OpenWithOption[] {
  return [...options].sort((a, b) => a.mod_title.localeCompare(b.mod_title));
}
