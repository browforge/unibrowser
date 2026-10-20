import type { TemplateRelationships } from "./relationshipConsent";

export function relationshipReason(value: string | undefined, fallback: string): string {
  const trimmed = (value ?? "").trim();
  return trimmed.length ? trimmed : fallback;
}

export function missingRequiredIds(
  relationships: TemplateRelationships,
  installedModIds: string[],
): string[] {
  const installed = new Set(installedModIds);
  return Object.keys(relationships.requires ?? {}).filter((id) => !installed.has(id));
}

export function conflictingEnabledIds(
  relationships: TemplateRelationships,
  enabledModIds: string[],
): string[] {
  const conflicts = Object.keys(relationships.incompatible ?? {});
  return conflicts.filter((id) => enabledModIds.includes(id));
}

export function isRelationshipCreateBlocked(
  relationships: TemplateRelationships,
  installedModIds: string[],
  enabledModIds: string[],
): boolean {
  if (missingRequiredIds(relationships, installedModIds).length > 0) {
    return true;
  }
  return conflictingEnabledIds(relationships, enabledModIds).length > 0;
}
