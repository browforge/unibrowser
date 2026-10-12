import { webInvoke } from "../webInvoke";

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return webInvoke<T>(cmd, args);
}

export function convertFileSrc(path: string): string {
  return path;
}
