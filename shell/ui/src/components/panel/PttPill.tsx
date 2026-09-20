import { useEffect, useRef } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";

type PttPillProps = {
  label: string;
  live?: boolean;
  onPressStart: () => void;
  onRelease: () => void;
  onCancel?: () => void;
};

export default function PttPill({ label, live = false, onPressStart, onRelease, onCancel }: PttPillProps) {
  const pressedRef = useRef(false);
  const detachRef = useRef<(() => void) | null>(null);
  const callbacksRef = useRef({ onPressStart, onRelease, onCancel });
  callbacksRef.current = { onPressStart, onRelease, onCancel };

  const stop = (mode: "send" | "cancel") => {
    if (!pressedRef.current) return;
    pressedRef.current = false;
    detachRef.current?.();
    detachRef.current = null;
    if (mode === "cancel") callbacksRef.current.onCancel?.();
    else callbacksRef.current.onRelease();
  };

  useEffect(
    () => () => {
      pressedRef.current = false;
      detachRef.current?.();
      detachRef.current = null;
    },
    [],
  );

  const onPointerDown = (event: ReactPointerEvent<HTMLButtonElement>) => {
    if (pressedRef.current) return;
    event.preventDefault();
    pressedRef.current = true;
    const up = () => stop("send");
    const cancel = () => stop("cancel");
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") stop("cancel");
    };
    document.addEventListener("pointerup", up);
    document.addEventListener("pointercancel", cancel);
    document.addEventListener("keydown", key);
    detachRef.current = () => {
      document.removeEventListener("pointerup", up);
      document.removeEventListener("pointercancel", cancel);
      document.removeEventListener("keydown", key);
    };
    callbacksRef.current.onPressStart();
  };

  return (
    <button
      type="button"
      aria-pressed={live}
      onPointerDown={onPointerDown}
      className={`ptt inline-flex items-center gap-[11px] rounded-pill border border-border bg-card py-2 pl-3 pr-4 shadow-e3${
        live ? " is-live" : ""
      }`}
    >
      <span className="ptt-dot h-2 w-2 rounded-full bg-border-strong" aria-hidden="true" />
      <span className="ptt-wave flex h-5 items-center gap-[3px]" aria-hidden="true">
        <i className="w-[3px] rounded-[2px] bg-border-strong" />
        <i className="w-[3px] rounded-[2px] bg-border-strong" />
        <i className="w-[3px] rounded-[2px] bg-border-strong" />
        <i className="w-[3px] rounded-[2px] bg-border-strong" />
        <i className="w-[3px] rounded-[2px] bg-border-strong" />
      </span>
      <span className="ptt-label font-ui text-[13px] font-semibold">{label}</span>
    </button>
  );
}
