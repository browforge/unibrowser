type Unlisten = () => void;

export async function listen<T>(
  event: string,
  handler: (event: { payload: T }) => void,
): Promise<Unlisten> {
  if (event === "browser-state-updated") {
    const wrapped = () => handler({ payload: undefined as T });
    window.addEventListener("ub-browser-state-updated", wrapped);
    return () => window.removeEventListener("ub-browser-state-updated", wrapped);
  }
  return () => undefined;
}
