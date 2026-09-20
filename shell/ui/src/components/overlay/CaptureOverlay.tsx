import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import { invokeTauriAsync } from "../../tauri";

/** Logical, viewport-relative capture rectangle (CSS px). */
export type OverlaySelection = { x: number; y: number; w: number; h: number };

/** A drag smaller than this commits nothing and snaps back to full dim. */
const MIN_SELECTION = 8;
/** Dimension chip floats this far below the selection (DESIGN.md capture-overlay). */
const CHIP_GAP = 6;
/** Viewport inset used when clamping the chip against the edges. */
const EDGE_INSET = 0;

const HINT = "Drag to box what you want explained";

const HANDLE_POSITIONS = ["left-0 top-0", "left-full top-0", "left-0 top-full", "left-full top-full"] as const;

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function rectFrom(origin: { x: number; y: number }, point: { x: number; y: number }): OverlaySelection {
  return {
    x: Math.min(origin.x, point.x),
    y: Math.min(origin.y, point.y),
    w: Math.abs(point.x - origin.x),
    h: Math.abs(point.y - origin.y),
  };
}

/**
 * Capture overlay surface (`?view=overlay`, F-03/F-04/F-05, AC-03).
 *
 * The page is transparent: the dim is painted as a spotlight `box-shadow` on the
 * selection, so the selected interior is truly clear and the desktop shows
 * through the Tauri window. Pass `selection` for the dev QA static-render mode
 * (`?view=overlay&sel=x,y,w,h`) — it disables interaction.
 */
export default function CaptureOverlay({ selection }: { selection?: OverlaySelection }) {
  const staticSelection = selection !== undefined;
  const [rect, setRect] = useState<OverlaySelection | null>(selection ?? null);
  const [chipSize, setChipSize] = useState({ w: 0, h: 0 });
  const originRef = useRef<{ x: number; y: number; pointerId: number } | null>(null);
  const chipRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      invokeTauriAsync("overlay_cancel")?.catch(() => undefined);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  /* Measure the chip so it can be clamped against the viewport edges. */
  useLayoutEffect(() => {
    const w = chipRef.current?.offsetWidth ?? 0;
    const h = chipRef.current?.offsetHeight ?? 0;
    if (w !== chipSize.w || h !== chipSize.h) setChipSize({ w, h });
  }, [rect, chipSize.w, chipSize.h]);

  const onPointerDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (staticSelection || e.button !== 0) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    originRef.current = { x: e.clientX, y: e.clientY, pointerId: e.pointerId };
    setRect({ x: e.clientX, y: e.clientY, w: 0, h: 0 });
  };

  const onPointerMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    const origin = originRef.current;
    if (!origin || origin.pointerId !== e.pointerId) return;
    const point = { x: clamp(e.clientX, 0, window.innerWidth), y: clamp(e.clientY, 0, window.innerHeight) };
    setRect(rectFrom(origin, point));
  };

  const onPointerUp = (e: ReactPointerEvent<HTMLDivElement>) => {
    const origin = originRef.current;
    if (!origin || origin.pointerId !== e.pointerId) return;
    originRef.current = null;
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    const point = { x: clamp(e.clientX, 0, window.innerWidth), y: clamp(e.clientY, 0, window.innerHeight) };
    const next = rectFrom(origin, point);
    if (next.w < MIN_SELECTION || next.h < MIN_SELECTION) {
      setRect(null);
      return;
    }
    /* Rust takes the capture after hiding the window — no UI follow-up. */
    invokeTauriAsync("capture_region_commit", {
      x: Math.round(next.x),
      y: Math.round(next.y),
      w: Math.round(next.w),
      h: Math.round(next.h),
    })?.catch(() => undefined);
  };

  const onPointerCancel = (e: ReactPointerEvent<HTMLDivElement>) => {
    const origin = originRef.current;
    if (!origin || origin.pointerId !== e.pointerId) return;
    originRef.current = null;
    setRect(null);
  };

  const chipLeft = rect ? clamp(rect.x, EDGE_INSET, Math.max(EDGE_INSET, window.innerWidth - chipSize.w)) : 0;
  const chipTop = rect
    ? clamp(rect.y + rect.h + CHIP_GAP, EDGE_INSET, Math.max(EDGE_INSET, window.innerHeight - chipSize.h))
    : 0;

  return (
    <div
      role="application"
      aria-label="Capture region overlay"
      className="fixed inset-0 cursor-crosshair select-none touch-none overflow-hidden"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerCancel}
    >
      {!rect && (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0"
          style={{ background: "hsl(var(--overlay-dim) / 0.4)" }}
        />
      )}

      {rect && (
        <>
          <div
            className="pointer-events-none absolute z-10 rounded-sharp border-2 border-primary bg-primary/[0.08]"
            style={{
              left: rect.x,
              top: rect.y,
              width: rect.w,
              height: rect.h,
              boxShadow: "0 0 0 9999px hsl(var(--overlay-dim) / 0.4)",
            }}
          >
            {HANDLE_POSITIONS.map((position) => (
              <span
                key={position}
                aria-hidden
                className={`absolute h-[7px] w-[7px] -translate-x-1/2 -translate-y-1/2 border-[1.5px] border-primary bg-card ${position}`}
              />
            ))}
          </div>

          <div
            ref={chipRef}
            className="pointer-events-none absolute z-20 whitespace-nowrap rounded-[6px] bg-primary px-2 py-[3px] font-mono text-[11px] font-medium leading-none text-primary-foreground"
            style={{ left: chipLeft, top: chipTop }}
          >
            {Math.round(rect.w)} × {Math.round(rect.h)} px
          </div>
        </>
      )}

      {/* top-11 keeps the hint clear of the menu bar and the macOS notch. */}
      <p className="pointer-events-none absolute left-1/2 top-11 z-20 inline-flex -translate-x-1/2 items-center gap-2 whitespace-nowrap rounded-pill border border-border bg-card/[0.92] px-3 py-2 font-ui text-[11px] leading-none text-muted-foreground shadow-e3">
        <span className="rounded-pill border border-primary/40 bg-primary-soft px-2 py-0.5 font-mono text-[11px] uppercase tracking-[0.04em] text-primary">
          region
        </span>
        <span>{HINT}</span>
        <kbd className="font-mono text-[11px] text-foreground">Esc</kbd>
      </p>
    </div>
  );
}
