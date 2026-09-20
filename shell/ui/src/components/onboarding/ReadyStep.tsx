import { invokeTauriAsync } from "../../tauri";
import { BODY, EYEBROW, GHOST_BUTTON, KBD, SECONDARY_BUTTON, STEP_HEADING } from "./styles";
import { REGISTERED_CAPTURE_ACCELERATORS } from "./types";
import type { ReactNode } from "react";

const MODIFIER_GLYPHS: Record<string, string> = {
  alt: "⌥",
  option: "⌥",
  shift: "⇧",
  ctrl: "⌃",
  control: "⌃",
  cmd: "⌘",
  command: "⌘",
  meta: "⌘",
  super: "⌘",
};

/** `"Alt+Shift+V"` → `"⌥ ⇧ V"`; unknown tokens pass through (`"F8"` → `"F8"`). */
function formatMacHotkey(accel: string): string {
  const parts = accel
    .split("+")
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  return parts.map((part) => MODIFIER_GLYPHS[part.toLowerCase()] ?? part).join(" ");
}

const REGION_ACCELERATOR = REGISTERED_CAPTURE_ACCELERATORS[0];
const REGION_LABEL = REGION_ACCELERATOR.intent.charAt(0).toUpperCase() + REGION_ACCELERATOR.intent.slice(1);

function ShortcutRow({ label, keys }: { label: string; keys: string }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="font-ui text-[13px] leading-[1.45] text-body">{label}</span>
      <kbd className={KBD}>{keys}</kbd>
    </div>
  );
}

function ArrowButton({
  children,
  onClick,
}: {
  children: ReactNode;
  onClick: () => void;
}) {
  return (
    <button type="button" onClick={onClick} className={`group ${GHOST_BUTTON}`}>
      {children}
      <span aria-hidden className="transition-transform duration-150 group-hover:translate-x-0.5">
        →
      </span>
    </button>
  );
}

export default function ReadyStep({ pttHotkey, offline }: { pttHotkey: string; offline: boolean }) {
  const pttKeys = formatMacHotkey(pttHotkey);
  const readyLine = offline ? "Ruoxi is in your menu bar." : "Cloud answers are allowed on request.";

  return (
    <div className="flex flex-col">
      <span className={EYEBROW}>Step 6 of 6</span>
      <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
        You&apos;re set up.
      </h2>
      <p className={`${BODY} mt-2.5 max-w-[50ch]`}>{readyLine}</p>

      <div className="mt-[18px] rounded-lg border border-border bg-muted p-4">
        <div className="flex flex-col gap-2.5 [&>*+*]:mt-3">
          <ShortcutRow label="Ask by voice" keys={pttKeys} />
          <ShortcutRow label={REGION_LABEL} keys={formatMacHotkey(REGION_ACCELERATOR.keys)} />
          <ShortcutRow label="Dismiss any panel" keys="Esc" />
        </div>
      </div>

      <div className="mt-5 flex flex-wrap items-center gap-3">
        <button type="button" onClick={() => invokeTauriAsync("start_region_capture")} className={SECONDARY_BUTTON}>
          Open capture
        </button>
        <ArrowButton onClick={() => invokeTauriAsync("show_settings")}>Settings</ArrowButton>
      </div>
    </div>
  );
}
