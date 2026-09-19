declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      transformCallback?: (cb: (ev: { payload: unknown }) => void, once?: boolean) => number;
    };
  }
}

/** Unsubscribe handle returned by {@link listenTauri}; safe to call more than once. */
export type TauriUnlisten = () => void;

const EVENT_TARGET = { kind: "Any" } as const;
const NOOP_UNLISTEN: TauriUnlisten = () => undefined;

export function invokeTauri(cmd: string, args?: Record<string, unknown>): void {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) return;
  invoke(cmd, args).catch(() => undefined);
}

/** Promise-returning guard: `null` outside Tauri, otherwise the raw invoke promise. */
export function invokeTauriAsync(cmd: string, args?: Record<string, unknown>): Promise<unknown> | null {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) return null;
  return invoke(cmd, args);
}

/**
 * Subscribe to a shell event without the `@tauri-apps/api` dependency.
 * Outside Tauri — or if the listener wiring fails — it resolves to a no-op
 * unlisten and never throws, so browser demos stay silent.
 */
export async function listenTauri<T>(event: string, cb: (payload: T) => void): Promise<TauriUnlisten> {
  const internals = window.__TAURI_INTERNALS__;
  const invoke = internals?.invoke;
  const transformCallback = internals?.transformCallback;
  if (!invoke || !transformCallback) return NOOP_UNLISTEN;
  try {
    const handler = transformCallback((ev) => cb(ev.payload as T));
    const eventId = (await invoke("plugin:event|listen", {
      event,
      target: EVENT_TARGET,
      handler,
    })) as number;
    return () => {
      invoke("plugin:event|unlisten", { event, target: EVENT_TARGET, eventId }).catch(() => undefined);
    };
  } catch {
    return NOOP_UNLISTEN;
  }
}
