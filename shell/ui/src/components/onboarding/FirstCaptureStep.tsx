import { useEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import { invokeTauriAsync } from "../../tauri";
import type { PermissionsStatus } from "../../permissions";
import { CAPTION, CHIP, GHOST_BUTTON, PRIMARY_BUTTON_SM, STEP_HEADING, BODY, EYEBROW } from "./styles";

type Rect = { x: number; y: number; w: number; h: number };

const MIN_SELECTION = 8;
const HANDLE_POSITIONS = ["left-0 top-0", "left-full top-0", "left-0 top-full", "left-full top-full"] as const;

const SAMPLE_TITLE = "Cell Biology · p.42";
const SAMPLE_HEADING = "Oxidative phosphorylation";
const SAMPLE_PARAGRAPH =
  "The electron-transport chain passes electrons between complexes embedded in the inner mitochondrial membrane. The energy released pumps protons into the intermembrane space, and their return through ATP synthase drives the phosphorylation of ADP.";
const DEMO_ANSWER =
  "This passage explains how the proton gradient becomes ATP — electrons fall down the chain, and their return through ATP synthase phosphorylates ADP.";

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function rectFrom(origin: { x: number; y: number }, point: { x: number; y: number }): Rect {
  return {
    x: Math.min(origin.x, point.x),
    y: Math.min(origin.y, point.y),
    w: Math.abs(point.x - origin.x),
    h: Math.abs(point.y - origin.y),
  };
}

function isSelectable(rect: Rect | null): rect is Rect {
  return rect !== null && rect.w >= MIN_SELECTION && rect.h >= MIN_SELECTION;
}

export default function FirstCaptureStep({
  permissionsStatus,
  onCaptureChange,
}: {
  permissionsStatus: PermissionsStatus;
  onCaptureChange?: (captured: boolean) => void;
}) {
  const frameRef = useRef<HTMLDivElement | null>(null);
  const originRef = useRef<{ x: number; y: number; pointerId: number } | null>(null);
  const [rect, setRect] = useState<Rect | null>(null);
  const [overlayLaunched, setOverlayLaunched] = useState(false);

  // The real overlay needs Tauri IPC and Screen Recording only; microphone and
  // accessibility are step-2 ritual items, not technical gates for the capture.
  const isTauri = !!window.__TAURI_INTERNALS__?.invoke;
  const realCaptureAvailable = isTauri && permissionsStatus.screen_recording;

  const pointInFrame = (clientX: number, clientY: number) => {
    const el = frameRef.current;
    if (!el) return { x: 0, y: 0 };
    const box = el.getBoundingClientRect();
    return { x: clamp(clientX - box.left, 0, box.width), y: clamp(clientY - box.top, 0, box.height) };
  };

  const onPointerDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    const point = pointInFrame(e.clientX, e.clientY);
    originRef.current = { ...point, pointerId: e.pointerId };
    setRect({ ...point, w: 0, h: 0 });
  };

  const onPointerMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    const origin = originRef.current;
    if (!origin || origin.pointerId !== e.pointerId) return;
    setRect(rectFrom(origin, pointInFrame(e.clientX, e.clientY)));
  };

  const onPointerUp = (e: ReactPointerEvent<HTMLDivElement>) => {
    const origin = originRef.current;
    if (!origin || origin.pointerId !== e.pointerId) return;
    originRef.current = null;
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    const next = rectFrom(origin, pointInFrame(e.clientX, e.clientY));
    setRect(isSelectable(next) ? next : null);
  };

  const onPointerCancel = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (originRef.current?.pointerId !== e.pointerId) return;
    originRef.current = null;
    setRect(null);
  };

  const boxWholeParagraph = () => {
    const el = frameRef.current;
    if (!el) return;
    const box = el.getBoundingClientRect();
    setRect({ x: 0, y: 0, w: box.width, h: box.height });
  };

  const launchRealCapture = () => {
    const pending = invokeTauriAsync("start_region_capture");
    if (!pending) return;
    pending.then(
      () => setOverlayLaunched(true),
      () => undefined,
    );
  };

  const selection = isSelectable(rect) ? rect : null;
  useEffect(() => {
    // The in-window demo never counts as a real capture.
    onCaptureChange?.(realCaptureAvailable ? false : selection !== null);
  }, [realCaptureAvailable, selection, onCaptureChange]);
  const status = realCaptureAvailable
    ? overlayLaunched
      ? "Ruòxī's overlay is over your screen — box the paragraph, then the panel answers."
      : "Screen Recording is on — start the guided capture and box this paragraph through the overlay."
    : selection
      ? `${Math.round(selection.w)} × ${Math.round(selection.h)} px selected — sample content, nothing was captured.`
      : "Waiting for your first capture.";

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3">
        <span className={EYEBROW}>Step 3 of 5 · the &ldquo;it works&rdquo; moment</span>
        <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
          Let&rsquo;s capture one thing together.
        </h2>
        <p className={BODY}>
          This is a real run against a real page — a paragraph from an open textbook. Box the sentence, ask, and watch
          the answer arrive without leaving this window.
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-[3fr_2fr]">
        <div>
          <div className="overflow-hidden rounded-lg border border-border bg-card shadow-e1">
            <div className="flex items-center gap-2 border-b border-border bg-muted px-3 py-2">
              <span aria-hidden className="flex gap-1.5">
                <i className="h-2.5 w-2.5 rounded-pill bg-destructive" />
                <i className="h-2.5 w-2.5 rounded-pill bg-live" />
                <i className="h-2.5 w-2.5 rounded-pill bg-success" />
              </span>
              <span className="font-ui text-[12px] font-semibold text-muted-foreground">{SAMPLE_TITLE}</span>
            </div>
            <div className="px-4 py-3.5">
              <h3 className="font-ui text-[15px] font-semibold text-foreground">{SAMPLE_HEADING}</h3>
              <div
                ref={frameRef}
                onPointerDown={onPointerDown}
                onPointerMove={onPointerMove}
                onPointerUp={onPointerUp}
                onPointerCancel={onPointerCancel}
                className="relative mt-2 cursor-crosshair touch-none select-none rounded-sm border border-border bg-card px-3 py-2.5"
              >
                <p className="font-ui text-[12.5px] leading-[1.6] text-muted-foreground">{SAMPLE_PARAGRAPH}</p>
                {rect && (
                  <span
                    aria-hidden
                    className="pointer-events-none absolute rounded-sharp border border-primary"
                    style={{ left: rect.x, top: rect.y, width: rect.w, height: rect.h }}
                  >
                    {HANDLE_POSITIONS.map((position) => (
                      <i
                        key={position}
                        className={`absolute h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 border border-primary bg-card ${position}`}
                      />
                    ))}
                  </span>
                )}
              </div>
            </div>
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={realCaptureAvailable ? launchRealCapture : boxWholeParagraph}
              className={PRIMARY_BUTTON_SM}
            >
              Box this paragraph and ask
            </button>
            <button
              type="button"
              onClick={() => setRect(null)}
              className={`${GHOST_BUTTON} h-8 px-3 text-[13px]`}
            >
              Clear
            </button>
            {selection && (
              <span className={`${CHIP} text-foreground`}>
                region · {Math.round(selection.w)} × {Math.round(selection.h)} px
              </span>
            )}
          </div>

          {!realCaptureAvailable && (
            <p className={`${CAPTION} mt-2`}>
              Drag across the paragraph to box it. Running the sample — grant Screen Recording in step 2 to try
              this against your real screen.
            </p>
          )}
        </div>

        <div className="grid min-h-[180px] place-items-center rounded-lg border border-dashed border-border bg-muted p-4">
          {selection ? (
            <div className="w-full rounded-lg border border-border bg-card p-4 shadow-e1">
              <div className="flex items-center gap-2">
                <span aria-hidden className="h-2 w-2 rounded-pill bg-success" />
                <span className="font-ui text-[12px] font-semibold text-foreground">answer</span>
                <span className={`${CHIP} ml-auto text-muted-foreground`}>sample</span>
              </div>
              <p className={`${CAPTION} mt-2`}>The panel appears next to the capture — never over it.</p>
              <p className="mt-2 font-ui text-[13px] leading-[1.5] text-foreground">{DEMO_ANSWER}</p>
            </div>
          ) : (
            <span className={`${CAPTION} text-center`}>
              The panel appears next to the capture — never over it.
            </span>
          )}
        </div>
      </div>

      <p aria-live="polite" className={CAPTION}>
        {status}
      </p>
    </div>
  );
}
