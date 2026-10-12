export function isWebPlatform(): boolean {
  return import.meta.env.VITE_UB_PLATFORM === "web";
}
