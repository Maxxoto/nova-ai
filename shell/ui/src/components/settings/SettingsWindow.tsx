import { useCallback, useEffect, useId, useRef, useState } from "react";
import type { KeyboardEvent as ReactKeyboardEvent, ReactNode } from "react";
import { getPermissionsStatus } from "../../permissions";
import type { PermissionKind, PermissionsStatus } from "../../permissions";
import { invokeTauriAsync, listenTauri } from "../../tauri";
import { KBD } from "../onboarding/styles";
import { ModelsSection } from "./ModelSetup";
import {
  AppRow,
  CHANGE_BUTTON,
  DANGER_BUTTON,
  FOCUS_RING,
  GHOST_BUTTON_SM,
  PERMISSION_ACTION_BUTTON,
  PICK_BUTTON,
  PRIMARY_BUTTON_SM,
  Row,
  SECONDARY_BUTTON,
  Section,
  Segmented,
  Tag,
  Toggle,
} from "./primitives";

/** Mirrors the cross-window event name in `App.tsx`. */
const THEME_EVENT = "ruoxi:theme";

const THEMES = ["system", "dawn", "night"] as const;
type Theme = (typeof THEMES)[number];
const ANSWER_LENGTHS = ["short", "normal"] as const;
type AnswerLength = (typeof ANSWER_LENGTHS)[number];
const SCOPES = ["window", "fullscreen"] as const;
type CaptureScope = (typeof SCOPES)[number];

/** The four modifiers the backend hotkey parser accepts (`global-hotkey`). */
type HotkeyModifier = "Cmd" | "Ctrl" | "Alt" | "Shift";

/**
 * Persisted shell settings — mirrors `shell/src-tauri/src/settings.rs`.
 * `offline` is ON by default: privacy-first, nothing leaves the Mac.
 */
type Settings = {
  offline: boolean;
  pause_captures: boolean;
  launch_at_login: boolean;
  read_aloud: boolean;
  answer_length: AnswerLength;
  theme: Theme;
  default_scope: CaptureScope;
  fullscreen_display_id: number | null;
  ptt_hotkey: string;
  sidecar_command: string;
  sidecar_args: string[];
  ping_interval_secs: number;
};

const DEFAULT_SETTINGS: Settings = {
  offline: true,
  pause_captures: false,
  launch_at_login: false,
  read_aloud: false,
  answer_length: "short",
  theme: "system",
  default_scope: "window",
  fullscreen_display_id: null,
  ptt_hotkey: "F8",
  sidecar_command: "python3",
  sidecar_args: ["-m", "app.interfaces.sidecar"],
  ping_interval_secs: 5,
};

type CaptureStats = { count: number; bytes: number; oldest_ms: number | null };
type Display = { id: string; name: string; width: number; height: number; scale: number; primary: boolean };

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item: unknown) => typeof item === "string");
}

function pick<T extends string>(values: readonly T[], raw: unknown, fallback: T): T {
  return typeof raw === "string" && (values as readonly string[]).includes(raw) ? (raw as T) : fallback;
}

/** Modifier tokens in the stable order the accelerator string is written with. */
const HOTKEY_MODIFIER_ORDER: readonly HotkeyModifier[] = ["Cmd", "Ctrl", "Alt", "Shift"];

const HOTKEY_MODIFIER_GLYPH: Record<HotkeyModifier, string> = {
  Cmd: "⌘",
  Ctrl: "⌃",
  Alt: "⌥",
  Shift: "⇧",
};

const HOTKEY_MODIFIER_CODES: Record<string, HotkeyModifier> = {
  MetaLeft: "Cmd",
  MetaRight: "Cmd",
  ControlLeft: "Ctrl",
  ControlRight: "Ctrl",
  AltLeft: "Alt",
  AltRight: "Alt",
  ShiftLeft: "Shift",
  ShiftRight: "Shift",
};

const HOTKEY_MODIFIER_ALIASES: Record<string, HotkeyModifier> = {
  cmd: "Cmd",
  command: "Cmd",
  super: "Cmd",
  ctrl: "Ctrl",
  control: "Ctrl",
  alt: "Alt",
  option: "Alt",
  shift: "Shift",
};

/** `KeyboardEvent.code` → a key name the backend parser accepts; `null` when unmappable. */
const HOTKEY_CODE_TOKENS: Record<string, string> = {
  Space: "Space",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Backquote: "Backquote",
  Backslash: "Backslash",
  BracketLeft: "BracketLeft",
  BracketRight: "BracketRight",
  Comma: "Comma",
  Equal: "Equal",
  Minus: "Minus",
  Period: "Period",
  Quote: "Quote",
  Semicolon: "Semicolon",
  Slash: "Slash",
  Enter: "Enter",
  NumpadEnter: "NumpadEnter",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  Insert: "Insert",
  NumpadAdd: "NumpadAdd",
  NumpadSubtract: "NumpadSubtract",
  NumpadMultiply: "NumpadMultiply",
  NumpadDivide: "NumpadDivide",
  NumpadDecimal: "NumpadDecimal",
  NumpadEqual: "NumpadEqual",
  AudioVolumeUp: "VolumeUp",
  AudioVolumeDown: "VolumeDown",
  AudioVolumeMute: "VolumeMute",
  MediaPlay: "MediaPlay",
  MediaPause: "MediaPause",
  MediaPlayPause: "MediaPlayPause",
  MediaStop: "MediaStop",
  MediaTrackNext: "MediaTrackNext",
  MediaTrackPrevious: "MediaTrackPrevious",
};

const HOTKEY_KEY_GLYPHS: Record<string, string> = {
  Space: "Space",
  Up: "↑",
  Down: "↓",
  Left: "←",
  Right: "→",
  Enter: "↩",
  NumpadEnter: "↩",
  Tab: "⇥",
  Backspace: "⌫",
  Delete: "⌦",
  PageUp: "⇞",
  PageDown: "⇟",
  Home: "↖",
  End: "↘",
};

function hotkeyKeyToken(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  if (/^Numpad[0-9]$/.test(code)) return code;
  return HOTKEY_CODE_TOKENS[code] ?? null;
}

function hotkeyKeyGlyph(token: string): string {
  return HOTKEY_KEY_GLYPHS[token] ?? token;
}

function hotkeyAccelerator(modifiers: readonly HotkeyModifier[], keyToken: string): string {
  const ordered = HOTKEY_MODIFIER_ORDER.filter((modifier) => modifiers.includes(modifier));
  return [...ordered, keyToken].join("+");
}

function hotkeyKeycaps(accelerator: string): string[] {
  const trimmed = accelerator.trim();
  if (!trimmed) return [];
  return trimmed.split("+").map((raw) => {
    const token = raw.trim();
    const modifier = HOTKEY_MODIFIER_ALIASES[token.toLowerCase()];
    if (modifier) return HOTKEY_MODIFIER_GLYPH[modifier];
    return hotkeyKeyGlyph(token);
  });
}

function hotkeyErrorMessage(error: unknown): string {
  if (typeof error === "string" && error.trim().length > 0) return error;
  if (error instanceof Error && error.message.trim().length > 0) return error.message;
  return "That hotkey can't be used — try another.";
}

function normalizeSettings(raw: unknown): Settings {
  if (!raw || typeof raw !== "object") return DEFAULT_SETTINGS;
  const record = raw as Record<string, unknown>;
  return {
    /* keep fields this window doesn't know (stt · tts · llm · …) so a toggle
       never drops settings added elsewhere */
    ...(record as Partial<Settings>),
    offline: typeof record.offline === "boolean" ? record.offline : DEFAULT_SETTINGS.offline,
    pause_captures:
      typeof record.pause_captures === "boolean" ? record.pause_captures : DEFAULT_SETTINGS.pause_captures,
    launch_at_login:
      typeof record.launch_at_login === "boolean" ? record.launch_at_login : DEFAULT_SETTINGS.launch_at_login,
    read_aloud: typeof record.read_aloud === "boolean" ? record.read_aloud : DEFAULT_SETTINGS.read_aloud,
    answer_length: pick(ANSWER_LENGTHS, record.answer_length, DEFAULT_SETTINGS.answer_length),
    theme: pick(THEMES, record.theme, DEFAULT_SETTINGS.theme),
    default_scope: pick(SCOPES, record.default_scope, DEFAULT_SETTINGS.default_scope),
    fullscreen_display_id:
      typeof record.fullscreen_display_id === "number" ? record.fullscreen_display_id : null,
    ptt_hotkey:
      typeof record.ptt_hotkey === "string" && record.ptt_hotkey.trim().length > 0
        ? record.ptt_hotkey
        : DEFAULT_SETTINGS.ptt_hotkey,
    sidecar_command:
      typeof record.sidecar_command === "string" ? record.sidecar_command : DEFAULT_SETTINGS.sidecar_command,
    sidecar_args: isStringArray(record.sidecar_args) ? record.sidecar_args : DEFAULT_SETTINGS.sidecar_args,
    ping_interval_secs:
      typeof record.ping_interval_secs === "number"
        ? record.ping_interval_secs
        : DEFAULT_SETTINGS.ping_interval_secs,
  };
}

function normalizeStats(raw: unknown): CaptureStats {
  if (!raw || typeof raw !== "object") return { count: 0, bytes: 0, oldest_ms: null };
  const record = raw as Record<string, unknown>;
  return {
    count: typeof record.count === "number" ? record.count : 0,
    bytes: typeof record.bytes === "number" ? record.bytes : 0,
    oldest_ms: typeof record.oldest_ms === "number" ? record.oldest_ms : null,
  };
}

function normalizeDisplays(raw: unknown): Display[] {
  if (!Array.isArray(raw)) return [];
  const displays: Display[] = [];
  for (const item of raw) {
    if (!item || typeof item !== "object") continue;
    const record = item as Record<string, unknown>;
    displays.push({
      id: typeof record.id === "string" ? record.id : "",
      name: typeof record.name === "string" ? record.name : "",
      width: typeof record.width === "number" ? record.width : 0,
      height: typeof record.height === "number" ? record.height : 0,
      scale: typeof record.scale === "number" ? record.scale : 1,
      primary: record.primary === true,
    });
  }
  return displays;
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** exponent;
  return `${exponent === 0 ? String(value) : value.toFixed(1)} ${units[exponent]}`;
}

function storeSummary(stats: CaptureStats): string {
  if (stats.count === 0) return "No captures yet.";
  const captures = `${stats.count} capture${stats.count === 1 ? "" : "s"}`;
  return `${formatBytes(stats.bytes)} across ${captures} · encrypted at rest.`;
}

function applyThemeClass(theme: Theme) {
  const root = document.documentElement;
  if (theme === "night") root.classList.add("dark");
  else if (theme === "dawn") root.classList.remove("dark");
  else root.classList.toggle("dark", window.matchMedia("(prefers-color-scheme: dark)").matches);
  window.dispatchEvent(new CustomEvent(THEME_EVENT, { detail: theme }));
}

function ThemeSwatch({
  selected,
  label,
  caption,
  preview,
  onSelect,
}: {
  selected: boolean;
  label: string;
  caption: string;
  preview: ReactNode;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onSelect}
      className={`flex min-w-0 flex-1 basis-[30%] flex-col items-start gap-1.5 rounded border bg-card p-3 text-left transition-colors duration-200 ${FOCUS_RING} ${
        selected ? "border-primary ring-2 ring-primary" : "border-border hover:border-border-strong"
      }`}
    >
      <span className="flex h-11 w-full overflow-hidden rounded-sm border border-border">{preview}</span>
      <span className="font-ui text-[13px] font-semibold leading-[1.4] text-foreground">{label}</span>
      <span className="font-ui text-[11px] leading-[1.4] text-muted-foreground">{caption}</span>
    </button>
  );
}

const DAWN_PREVIEW = (
  <>
    <span className="flex-1 bg-background" />
    <span className="flex-1 bg-card" />
    <span className="flex-1 bg-primary" />
  </>
);

const NIGHT_PREVIEW = (
  <span className="dark flex h-full flex-1">
    <span className="flex-1 bg-background" />
    <span className="flex-1 bg-card" />
    <span className="flex-1 bg-primary" />
  </span>
);

const SYSTEM_PREVIEW = (
  <>
    <span className="flex flex-1">
      <span className="flex-1 bg-background" />
      <span className="flex-1 bg-card" />
      <span className="flex-1 bg-primary" />
    </span>
    <span className="dark flex flex-1">
      <span className="flex-1 bg-background" />
      <span className="flex-1 bg-card" />
      <span className="flex-1 bg-primary" />
    </span>
  </>
);

function CloudOffGlyph({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M4 4l16 16" />
      <path d="M7 18h9a4 4 0 0 0 2-7.5" />
      <path d="M6 9.4A3.8 3.8 0 0 0 7 18" />
    </svg>
  );
}

function CloudGlyph({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M7 18h9a4 4 0 0 0 .6-8A5.5 5.5 0 0 0 6 9.4 3.8 3.8 0 0 0 7 18Z" />
    </svg>
  );
}

function OfflineBanner({ offline }: { offline: boolean }) {
  return (
    <div className="flex items-center gap-3 rounded border border-border bg-muted p-3.5">
      <span
        aria-hidden="true"
        className={`grid h-[30px] w-[30px] flex-none place-items-center rounded-[9px] border border-border bg-card ${
          offline ? "text-muted-foreground" : "text-success"
        }`}
      >
        {offline ? <CloudOffGlyph className="h-4 w-4" /> : <CloudGlyph className="h-4 w-4" />}
      </span>
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="font-ui text-[14px] font-semibold leading-[1.4] text-foreground">
          {offline ? "Offline mode is on" : "Cloud answers are allowed on request"}
        </span>
        <p className="font-ui text-[13px] leading-[1.45] text-muted-foreground">
          {offline
            ? "Zero network calls. Captures, answers and memory stay on this Mac."
            : "Requests may reach the cloud when an answer needs it."}
        </p>
      </div>
      <span className="inline-flex flex-none items-center gap-1.5 rounded-pill border border-border bg-card px-2.5 py-1 font-ui text-[11px] font-medium text-muted-foreground">
        <span
          aria-hidden="true"
          className={`h-[7px] w-[7px] rounded-pill ${offline ? "bg-muted-foreground" : "bg-success"}`}
        />
        {offline ? "offline" : "local only"}
      </span>
    </div>
  );
}

function ConfirmDeleteDialog({
  count,
  onCancel,
  onConfirm,
}: {
  count: number;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const [typed, setTyped] = useState("");
  const inputId = useId();
  const titleId = useId();
  const canDelete = typed === "DELETE";

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="fixed inset-0 z-50 grid place-items-center p-6">
      <div aria-hidden="true" className="absolute inset-0 bg-foreground/40" onClick={onCancel} />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="relative w-[440px] max-w-full rounded-lg border border-border bg-card p-6 shadow-e3"
      >
        <h3 id={titleId} className="font-ui text-[17px] font-semibold leading-[1.3] text-foreground">
          {count === 0 ? "Delete all captures?" : `Delete all ${count} capture${count === 1 ? "" : "s"}?`}
        </h3>
        <p className="mt-2 font-ui text-[13px] leading-[1.45] text-muted-foreground">
          This removes every stored capture from this Mac. It cannot be undone. Type{" "}
          <span className="font-mono font-medium text-foreground">DELETE</span> to confirm.
        </p>
        <div className="mt-4 flex flex-col gap-1.5">
          <label htmlFor={inputId} className="font-ui text-[13px] text-muted-foreground">
            Confirmation
          </label>
          <input
            id={inputId}
            value={typed}
            autoFocus
            spellCheck={false}
            autoComplete="off"
            onChange={(event) => setTyped(event.target.value)}
            className={`rounded border border-border bg-card px-3 py-2 font-ui text-[13px] text-foreground focus:border-primary ${FOCUS_RING}`}
          />
        </div>
        <div className="mt-5 flex justify-end gap-2">
          <button type="button" onClick={onCancel} className={GHOST_BUTTON_SM}>
            Cancel
          </button>
          <button
            type="button"
            disabled={!canDelete}
            onClick={onConfirm}
            className={`inline-flex h-8 items-center justify-center rounded border border-destructive bg-card px-3 font-ui text-[13px] font-semibold text-destructive transition-colors duration-200 hover:bg-destructive/10 disabled:cursor-default disabled:opacity-50 ${FOCUS_RING}`}
          >
            Delete all
          </button>
        </div>
      </div>
    </div>
  );
}

const PERMISSION_ROWS: { kind: PermissionKind; name: string; help: string }[] = [
  {
    kind: "screen_recording",
    name: "Screen Recording",
    help: "Region, window and full-screen capture.",
  },
  {
    kind: "microphone",
    name: "Microphone",
    help: "Push-to-talk.",
  },
  {
    kind: "accessibility",
    name: "Accessibility",
    help: "Global hotkeys only. Ruòxī never clicks or types.",
  },
];

function permissionGranted(kind: PermissionKind, status: PermissionsStatus | null): boolean {
  if (!status) return false;
  if (kind === "screen_recording") return status.screen_recording;
  if (kind === "microphone") return status.microphone === "granted";
  return status.accessibility;
}

function displayCaption(display: Display): string {
  const role = display.primary ? "Primary" : "Secondary";
  const anchor = display.primary ? " · panel anchor" : "";
  return `${role} · ${display.width} × ${display.height} · scale ${display.scale}×${anchor}`;
}

type HotkeyCommitResult = { ok: true } | { ok: false; message: string; tone: "error" | "muted" };

function HotkeyKeycaps({ caps }: { caps: readonly string[] }) {
  return (
    <span className="flex flex-none items-center gap-1">
      {caps.map((cap, index) => (
        <kbd
          key={`${cap}-${String(index)}`}
          className={`${KBD} inline-flex min-w-[1.75rem] items-center justify-center`}
        >
          {cap}
        </kbd>
      ))}
    </span>
  );
}

function PushToTalkRow({
  value,
  onCommit,
}: {
  value: string;
  onCommit: (accel: string) => Promise<HotkeyCommitResult>;
}) {
  const [recording, setRecording] = useState(false);
  const [captured, setCaptured] = useState<string | null>(null);
  const [held, setHeld] = useState<HotkeyModifier[]>([]);
  const [feedback, setFeedback] = useState<{ message: string; tone: "error" | "muted" } | null>(null);
  const [saving, setSaving] = useState(false);
  const surfaceRef = useRef<HTMLDivElement | null>(null);
  const changeRef = useRef<HTMLButtonElement | null>(null);

  const closeRecorder = useCallback((refocus: boolean) => {
    setRecording(false);
    setCaptured(null);
    setHeld([]);
    setFeedback(null);
    setSaving(false);
    if (refocus) requestAnimationFrame(() => changeRef.current?.focus());
  }, []);

  const openRecorder = () => {
    setCaptured(null);
    setHeld([]);
    setFeedback(null);
    setSaving(false);
    setRecording(true);
  };

  useEffect(() => {
    if (recording) surfaceRef.current?.focus();
  }, [recording]);

  useEffect(() => {
    if (!recording) return;
    const onEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      closeRecorder(true);
    };
    window.addEventListener("keydown", onEscape, true);
    return () => window.removeEventListener("keydown", onEscape, true);
  }, [recording, closeRecorder]);

  const onSurfaceKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Tab") return;
    event.preventDefault();
    event.stopPropagation();
    const modifier = HOTKEY_MODIFIER_CODES[event.code];
    if (modifier) {
      setHeld((prev) => (prev.includes(modifier) ? prev : [...prev, modifier]));
      return;
    }
    const keyToken = hotkeyKeyToken(event.code);
    if (!keyToken) {
      setFeedback({ tone: "error", message: "That key can't be used for a hotkey — try another." });
      return;
    }
    const modifiers: HotkeyModifier[] = [];
    if (event.metaKey) modifiers.push("Cmd");
    if (event.ctrlKey) modifiers.push("Ctrl");
    if (event.altKey) modifiers.push("Alt");
    if (event.shiftKey) modifiers.push("Shift");
    setCaptured(hotkeyAccelerator(modifiers, keyToken));
    setFeedback(null);
  };

  const onSurfaceKeyUp = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    const modifier = HOTKEY_MODIFIER_CODES[event.code];
    if (modifier) setHeld((prev) => prev.filter((item) => item !== modifier));
  };

  const useThis = () => {
    if (!captured || saving) return;
    const accel = captured;
    setSaving(true);
    void onCommit(accel).then((result) => {
      if (result.ok) {
        closeRecorder(true);
        return;
      }
      setSaving(false);
      setFeedback({ tone: result.tone, message: result.message });
    });
  };

  if (recording) {
    const caps = captured ? hotkeyKeycaps(captured) : held.map((modifier) => HOTKEY_MODIFIER_GLYPH[modifier]);
    return (
      <div
        ref={surfaceRef}
        tabIndex={0}
        role="group"
        aria-label="Press the key you want to hold to talk. Modifiers optional. Escape cancels."
        onKeyDown={onSurfaceKeyDown}
        onKeyUp={onSurfaceKeyUp}
        onBlur={() => setHeld([])}
        className={`flex flex-col gap-3 rounded-lg border-2 border-primary bg-muted p-4 ${FOCUS_RING}`}
      >
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex min-w-0 flex-col gap-1">
            <span className="font-ui text-[14px] font-semibold leading-[1.4] text-foreground">
              Press the key you want to hold to talk
            </span>
            <span className="font-ui text-[12px] leading-[1.4] text-muted-foreground">
              Modifiers optional · Tab moves on · Esc cancels
            </span>
          </div>
          {caps.length > 0 ? (
            <HotkeyKeycaps caps={caps} />
          ) : (
            <span className="font-ui text-[12px] text-muted-foreground">waiting…</span>
          )}
        </div>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p
            aria-live="polite"
            className={`min-h-[1.05rem] font-ui text-[13px] leading-[1.4] ${
              feedback?.tone === "error" ? "text-destructive" : "text-muted-foreground"
            }`}
          >
            {feedback?.message ?? ""}
          </p>
          <div className="flex flex-none items-center gap-2">
            <button type="button" disabled={!captured || saving} onClick={useThis} className={PRIMARY_BUTTON_SM}>
              Use this
            </button>
            <button
              type="button"
              disabled={saving}
              onClick={() => closeRecorder(true)}
              className={GHOST_BUTTON_SM}
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    );
  }

  const caps = hotkeyKeycaps(value);
  return (
    <Row
      label="Push-to-talk hotkey"
      help="Hold to talk, release to send."
      side={
        <>
          {caps.length > 0 ? (
            <HotkeyKeycaps caps={caps} />
          ) : (
            <span className="font-mono text-[11px] text-muted-foreground">unset</span>
          )}
          <button ref={changeRef} type="button" onClick={openRecorder} className={CHANGE_BUTTON}>
            Change
          </button>
        </>
      }
    />
  );
}

export default function SettingsWindow({ reducedMotion = false }: { reducedMotion?: boolean }) {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  /** Latest settings for event handlers, so writes never race a re-render. */
  const settingsRef = useRef<Settings>(DEFAULT_SETTINGS);
  const [stats, setStats] = useState<CaptureStats | null>(null);
  const [displays, setDisplays] = useState<Display[] | null>(null);
  const [permissions, setPermissions] = useState<PermissionsStatus | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [permissionNote, setPermissionNote] = useState("");

  const apply = (next: Settings) => {
    settingsRef.current = next;
    setSettings(next);
  };

  const refreshStats = () => {
    const pending = invokeTauriAsync("capture_store_stats");
    if (!pending) {
      setStats({ count: 0, bytes: 0, oldest_ms: null });
      return;
    }
    pending.then(
      (raw) => setStats(normalizeStats(raw)),
      () => undefined,
    );
  };

  useEffect(() => {
    let off: (() => void) | null = null;
    void (async () => {
      off = await listenTauri<unknown>("settings:changed", (payload) => {
        apply(normalizeSettings(payload));
      });
    })();
    return () => {
      off?.();
    };
  }, []);

  useEffect(() => {
    let active = true;

    const pendingSettings = invokeTauriAsync("get_settings");
    if (pendingSettings) {
      pendingSettings.then(
        (raw) => {
          if (!active) return;
          const next = normalizeSettings(raw);
          applyThemeClass(next.theme);
          apply(next);
        },
        () => undefined,
      );
    }

    const pendingStats = invokeTauriAsync("capture_store_stats");
    if (pendingStats) {
      pendingStats.then(
        (raw) => {
          if (active) setStats(normalizeStats(raw));
        },
        () => {
          if (active) setStats({ count: 0, bytes: 0, oldest_ms: null });
        },
      );
    } else {
      setStats({ count: 0, bytes: 0, oldest_ms: null });
    }

    const pendingDisplays = invokeTauriAsync("list_displays");
    if (pendingDisplays) {
      pendingDisplays.then(
        (raw) => {
          if (active) setDisplays(normalizeDisplays(raw));
        },
        () => {
          if (active) setDisplays([]);
        },
      );
    } else {
      setDisplays([]);
    }

    getPermissionsStatus().then(
      (status) => {
        if (active) setPermissions(status);
      },
      () => undefined,
    );

    return () => {
      active = false;
    };
  }, []);

  /* Optimistic write: apply locally (theme also hits the root immediately),
     then persist the FULL object so untouched fields survive the round-trip. */
  const setField = (patch: Partial<Settings>) => {
    const next = { ...settingsRef.current, ...patch };
    if (patch.theme) applyThemeClass(patch.theme);
    apply(next);
    invokeTauriAsync("set_settings", { settings: next })?.catch(() => undefined);
  };

  /* Push-to-talk rebinding is not optimistic: validate first, then persist the
     FULL object and only apply locally once the write resolves. */
  const commitHotkey = async (accel: string): Promise<HotkeyCommitResult> => {
    const validation = invokeTauriAsync("validate_hotkey", { accel });
    if (!validation) {
      return {
        ok: false,
        tone: "muted",
        message: "Saving a hotkey needs the desktop app — this preview can't validate or persist it.",
      };
    }
    try {
      await validation;
    } catch (error) {
      return { ok: false, tone: "error", message: hotkeyErrorMessage(error) };
    }
    const next = { ...settingsRef.current, ptt_hotkey: accel };
    const write = invokeTauriAsync("set_settings", { settings: next });
    if (!write) return { ok: false, tone: "muted", message: "Saving a hotkey needs the desktop app." };
    try {
      await write;
    } catch {
      return {
        ok: false,
        tone: "error",
        message: "Couldn't save that hotkey — your previous shortcut is unchanged.",
      };
    }
    apply(next);
    return { ok: true };
  };

  const handleDeleteAll = () => {
    setConfirmOpen(false);
    const pending = invokeTauriAsync("capture_delete_all");
    if (!pending) {
      refreshStats();
      return;
    }
    pending.then(
      () => refreshStats(),
      () => refreshStats(),
    );
  };

  const openPane = (kind: PermissionKind) => {
    setPermissionNote("Opens System Settings — nothing changes here.");
    invokeTauriAsync("open_privacy_pane", { kind })?.catch(() => undefined);
  };

  const openTimeline = () => {
    invokeTauriAsync("show_timeline")?.catch(() => undefined);
  };

  const runOnboarding = () => {
    invokeTauriAsync("show_onboarding")?.catch(() => undefined);
  };

  const storeHelp = stats === null ? "Reading the local store…" : storeSummary(stats);

  return (
    <div
      className={`mx-auto flex w-full max-w-[820px] flex-col gap-5 p-8${reducedMotion ? " reduced-motion rm-halve" : ""}`}
    >
      <header className="flex flex-col gap-1">
        <span className="font-mono text-[11px] font-medium uppercase tracking-[0.09em] text-muted-foreground">
          Settings
        </span>
        <h1 className="font-ui text-[22px] font-bold leading-[1.2] tracking-[-0.02em] text-foreground">
          Settings.
        </h1>
        <p className="font-ui text-[14px] leading-[1.45] text-muted-foreground">
          What is stored, what is sent, and how to stop it.
        </p>
      </header>

      <OfflineBanner offline={settings.offline} />

      <Section id="privacy" title="Privacy">
        <div className="flex flex-col">
          <Row
            label="Offline mode — nothing leaves this Mac"
            help="Instant, no restart. The chip moves to offline everywhere."
            side={
              <Toggle
                label="Offline mode"
                on={settings.offline}
                onChange={(next) => setField({ offline: next })}
              />
            }
          />
          <Row
            label="Voice audio"
            help="Deleted about a minute after a session ends. There is no replay."
            side={<Tag tone="ok">deleted ≈1 min</Tag>}
          />
          <Row
            label="Local store"
            help={
              <>
                {storeHelp}
                {stats !== null && stats.count > 0 ? (
                  <>
                    {" "}
                    <button
                      type="button"
                      onClick={openTimeline}
                      className={`font-ui text-[13px] font-medium text-muted-foreground underline decoration-border-strong underline-offset-2 transition-colors duration-200 hover:text-foreground ${FOCUS_RING}`}
                    >
                      Review captures →
                    </button>
                  </>
                ) : null}
              </>
            }
            side={
              <button type="button" onClick={() => setConfirmOpen(true)} className={DANGER_BUTTON}>
                Delete all…
              </button>
            }
          />
          <Row
            label="Send diagnostics"
            help="Off. Crash reports stay on this Mac."
            side={<Tag>off</Tag>}
          />
        </div>
      </Section>

      <Section
        id="autocapture"
        title="Off for every app until you say otherwise."
        headerExtra={<Tag>Coming soon</Tag>}
        description="Each row says what would be stored."
      >
        <div className="flex flex-col gap-2">
          <AppRow
            mark="Sa"
            name="Safari"
            caption="Active tab after 30s of stillness · text only"
            side={<Tag>off</Tag>}
          />
          <AppRow
            mark="Pv"
            name="Preview"
            caption="Highlighted text on the open page"
            side={<Tag>off</Tag>}
          />
          <AppRow
            mark="VS"
            name="VS Code"
            caption="Visible editor when an error appears"
            side={<Tag>off</Tag>}
          />
          <p className="pt-1 font-ui text-[12px] leading-[1.5] text-muted-foreground">
            Capture is off everywhere. The tray glyph stays in the paused state.
          </p>
        </div>
      </Section>

      <Section id="voice" title="Voice and answers">
        <div className="flex flex-col">
          <PushToTalkRow value={settings.ptt_hotkey} onCommit={commitHotkey} />
          <p className="pt-1 font-ui text-[12px] leading-[1.5] text-muted-foreground">
            Applies on the next start.
          </p>
          <Row
            label="Read answers aloud"
            help="Esc always stops the audio."
            side={
              <>
                <Tag>Coming soon</Tag>
                <Toggle
                  label="Read answers aloud"
                  on={settings.read_aloud}
                  onChange={(next) => setField({ read_aloud: next })}
                />
              </>
            }
          />
          <Row
            label="Answer length"
            help="Short by default. The panel offers more."
            side={
              <>
                <Tag>Coming soon</Tag>
                <Segmented
                  ariaLabel="Answer length"
                  value={settings.answer_length}
                  onChange={(next) => setField({ answer_length: next })}
                  options={[
                    { value: "short", label: "Short" },
                    { value: "normal", label: "Normal" },
                  ]}
                />
              </>
            }
          />
        </div>
      </Section>

      <ModelsSection
        offline={settings.offline}
        onTurnOffOffline={() => setField({ offline: false })}
      />
      <Section id="displays" title="Displays">
        <div className="flex flex-col gap-3">
          <Row
            label="Full-screen capture"
            help="Which display the whole-screen scope uses."
            side={<Tag>{displays === null ? "…" : `${displays.length} connected`}</Tag>}
          />
          {displays !== null && displays.length === 0 ? (
            <p className="font-ui text-[12px] leading-[1.5] text-muted-foreground">No displays found.</p>
          ) : null}
          {displays !== null && displays.length > 0 ? (
            <div className="flex flex-col gap-2">
              {displays.map((display, index) => {
                const displayNumber = Number(display.id);
                const valid = Number.isInteger(displayNumber) && display.id.trim() !== "";
                const picked = valid && settings.fullscreen_display_id === displayNumber;
                return (
                  <AppRow
                    key={display.id || String(index)}
                    mark={String(index + 1)}
                    name={display.name || "Display"}
                    caption={displayCaption(display)}
                    side={
                      <button
                        type="button"
                        aria-pressed={picked}
                        disabled={!valid}
                        onClick={() => setField({ fullscreen_display_id: displayNumber })}
                        className={PICK_BUTTON}
                      >
                        Use for full screen
                      </button>
                    }
                  />
                );
              })}
            </div>
          ) : null}
        </div>
        <div className="flex flex-col">
          <Row
            label="Default capture scope"
            help="What a voice ask captures when you do not box anything."
            side={
              <Segmented
                ariaLabel="Default capture scope"
                value={settings.default_scope}
                onChange={(next) => setField({ default_scope: next })}
                options={[
                  { value: "window", label: "Active window" },
                  { value: "fullscreen", label: "Whole screen" },
                ]}
              />
            }
          />
        </div>
      </Section>

      <Section id="appearance" title="Appearance" description="Both themes use the same components.">
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap gap-3" role="group" aria-label="Theme">
            <ThemeSwatch
              selected={settings.theme === "dawn"}
              label="Dawn sky"
              caption="Cool paper canvas, dawn-blue accent"
              preview={DAWN_PREVIEW}
              onSelect={() => setField({ theme: "dawn" })}
            />
            <ThemeSwatch
              selected={settings.theme === "night"}
              label="Night before dawn"
              caption="Deep night-blue, brighter blue accent"
              preview={NIGHT_PREVIEW}
              onSelect={() => setField({ theme: "night" })}
            />
            <ThemeSwatch
              selected={settings.theme === "system"}
              label="Follow the system"
              caption="Switches with macOS appearance"
              preview={SYSTEM_PREVIEW}
              onSelect={() => setField({ theme: "system" })}
            />
          </div>
          <p className="font-ui text-[12px] leading-[1.5] text-muted-foreground">
            Ruòxī follows the system by default.
          </p>
        </div>
        <div className="flex flex-col">
          <Row
            label="Reduce motion"
            help="Follows the system setting."
            side={<Tag>{reducedMotion ? "reduce" : "system"}</Tag>}
          />
        </div>
      </Section>

      <Section id="access" title="Access" description="Revoking one shows which features stop.">
        <div className="flex flex-col">
          {PERMISSION_ROWS.map((row) => {
            const granted = permissionGranted(row.kind, permissions);
            return (
              <Row
                key={row.kind}
                label={row.name}
                help={row.help}
                side={
                  <>
                    <Tag tone={granted ? "ok" : "neutral"}>{granted ? "granted" : "not granted"}</Tag>
                    <button type="button" onClick={() => openPane(row.kind)} className={PERMISSION_ACTION_BUTTON}>
                      {granted ? "Revoke" : "Grant"}
                    </button>
                  </>
                }
              />
            );
          })}
          <div className="flex flex-wrap items-center gap-3 pt-4">
            <button type="button" onClick={runOnboarding} className={SECONDARY_BUTTON}>
              Run setup again
            </button>
            <span aria-live="polite" className="font-ui text-[12px] leading-[1.5] text-muted-foreground">
              {permissionNote}
            </span>
          </div>
        </div>
      </Section>

      <section
        id="about"
        className="scroll-mt-24 flex items-start gap-4 rounded-xl border border-border bg-card p-5"
      >
        <span
          aria-hidden="true"
          className="h-11 w-11 flex-none rounded-xl bg-gradient-to-br from-primary to-primary-active shadow-e1"
        />
        <div className="flex min-w-0 flex-1 flex-col gap-2">
          <span className="font-ui text-[17px] font-semibold leading-[1.3] tracking-[-0.01em] text-foreground">
            Ruòxī 若曦
          </span>
          <p className="font-ui text-[13px] leading-[1.45] text-muted-foreground">
            A menu-bar assistant for macOS and Windows. Open source, MIT.
          </p>
          <div className="flex flex-wrap items-center gap-3">
            <span className="font-mono text-[11px] font-medium text-foreground">M0 · v0.1</span>
            <span className="font-mono text-[11px] font-medium text-muted-foreground">Local-first</span>
          </div>
        </div>
      </section>

      {confirmOpen ? (
        <ConfirmDeleteDialog
          count={stats?.count ?? 0}
          onCancel={() => setConfirmOpen(false)}
          onConfirm={handleDeleteAll}
        />
      ) : null}
    </div>
  );
}
