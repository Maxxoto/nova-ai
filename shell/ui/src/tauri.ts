declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
  }
}

export function invokeTauri(cmd: string, args?: Record<string, unknown>): void {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) return;
  invoke(cmd, args).catch(() => undefined);
}
