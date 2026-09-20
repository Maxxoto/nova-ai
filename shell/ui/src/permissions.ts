import { invokeTauriAsync } from "./tauri";

export type PermissionKind = "screen_recording" | "microphone" | "accessibility";
export type MicStatus = "granted" | "denied" | "not_determined" | "unknown";

export interface PermissionsStatus {
  screen_recording: boolean;
  microphone: MicStatus;
  accessibility: boolean;
}

function defaultStatus(): PermissionsStatus {
  return { screen_recording: false, microphone: "unknown", accessibility: false };
}

async function callStatus(cmd: string, args?: Record<string, unknown>): Promise<PermissionsStatus> {
  const pending = invokeTauriAsync(cmd, args);
  if (!pending) return defaultStatus();
  try {
    return (await pending) as PermissionsStatus;
  } catch {
    return defaultStatus();
  }
}

export function getPermissionsStatus(): Promise<PermissionsStatus> {
  return callStatus("permissions_status");
}

export function requestPermission(kind: PermissionKind): Promise<PermissionsStatus> {
  return callStatus("permissions_request", { kind });
}
