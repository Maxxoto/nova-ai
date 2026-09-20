import { useEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import ResultPanel from "../panel/ResultPanel";
import type { CaptureInfo } from "../panel/types";
import { BODY, CAPTION, EYEBROW, GHOST_BUTTON_SM, PRIMARY_BUTTON_SM, SECONDARY_BUTTON_SM, STEP_HEADING, TAG_ACCENT } from "./styles";

type Rect = { x: number; y: number; w: number; h: number };

const MIN_SELECTION = 8;
const HANDLE_POSITIONS = ["left-0 top-0", "left-full top-0", "left-0 top-full", "left-full top-full"] as const;

const SAMPLE_TITLE = "Cell Biology · p.42";
const SAMPLE_HEADING = "Oxidative phosphorylation";
const SAMPLE_PARAGRAPH =
  "The electron-transport chain passes electrons between complexes embedded in the inner mitochondrial membrane. The energy released pumps protons into the intermembrane space, and their return through ATP synthase drives the phosphorylation of ADP.";

const DEMO_ANSWER =
  "The passage explains **how the proton gradient becomes ATP**: electrons move down the chain, the energy pumps protons across the inner membrane, and their return through ATP synthase phosphorylates ADP.";

const DEMO_CAPTURE: CaptureInfo = {
  id: "cap_20250923140200",
  time: "just now",
  scope: "region",
  width: 412,
  height: 96,
};

type Phase = "idle" | "thinking" | "complete";

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
  reducedMotion,
  onDemoDone,
}: {
  reducedMotion?: boolean;
  onDemoDone?: (done: boolean) => void;
}) {
  const frameRef = useRef<HTMLDivElement | null>(null);
  const originRef = useRef<{ x: number; y: number; pointerId: number } | null>(null);
  const [rect, setRect] = useState<Rect | null>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const timerRef = useRef<number | null>(null);

  useEffect(
    () => () => {
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    },
    [],
  );

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

  const runDemo = () => {
    if (phase !== "idle") return;
    if (!isSelectable(rect)) boxWholeParagraph();
    setPhase("thinking");
    timerRef.current = window.setTimeout(() => {
      timerRef.current = null;
      setPhase("complete");
      onDemoDone?.(true);
    }, 1200);
  };

  const clearDemo = () => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    originRef.current = null;
    setRect(null);
    setPhase("idle");
    onDemoDone?.(false);
  };

  const selection = isSelectable(rect) ? rect : null;
  const status = phase === "thinking" ? "Reading the region…" : phase === "complete" ? "Answer complete." : "Ready.";

  return (
    <div className="flex flex-col">
      <span className={EYEBROW}>Step 3 of 6</span>
      <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
        Capture one thing.
      </h2>
      <p className={`${BODY} mt-2 max-w-[52ch]`}>Box the paragraph, then ask.</p>

      <div className="mt-5 grid grid-cols-1 items-start gap-4 sm:grid-cols-[1fr_2fr]">
        <div className="p-[22px]">
          <div className="max-w-[600px] overflow-hidden rounded-md border border-border bg-card shadow-e1">
            <div className="flex items-center gap-2.5 border-b border-border bg-muted px-3 py-2">
              <span aria-hidden className="flex gap-1.5">
                <i className="h-2.5 w-2.5 rounded-pill bg-destructive" />
                <i className="h-2.5 w-2.5 rounded-pill bg-live" />
                <i className="h-2.5 w-2.5 rounded-pill bg-success" />
              </span>
              <span className="font-ui text-[12px] font-semibold text-foreground">{SAMPLE_TITLE}</span>
            </div>
            <div className="px-5 py-[18px]">
              <h3 className="mb-2 font-ui text-[15px] font-semibold text-foreground">{SAMPLE_HEADING}</h3>
              <div
                ref={frameRef}
                onPointerDown={onPointerDown}
                onPointerMove={onPointerMove}
                onPointerUp={onPointerUp}
                onPointerCancel={onPointerCancel}
                className="relative cursor-crosshair touch-none select-none overflow-hidden rounded-sm border border-border bg-card px-3 py-2.5"
              >
                <p className="font-ui text-[13px] leading-[1.6] text-body">{SAMPLE_PARAGRAPH}</p>
                {rect && (
                  <span
                    aria-hidden
                    className="pointer-events-none absolute rounded-sharp border-2 border-primary bg-primary/[0.08]"
                    style={{ left: rect.x, top: rect.y, width: rect.w, height: rect.h }}
                  >
                    {HANDLE_POSITIONS.map((position) => (
                      <i
                        key={position}
                        className={`absolute h-[7px] w-[7px] -translate-x-1/2 -translate-y-1/2 border border-primary bg-card ${position}`}
                      />
                    ))}
                  </span>
                )}
              </div>
            </div>
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2.5">
            <button
              type="button"
              onClick={runDemo}
              disabled={phase === "complete"}
              className={phase === "complete" ? SECONDARY_BUTTON_SM : PRIMARY_BUTTON_SM}
            >
              {phase === "complete" ? "Captured" : "Ask"}
            </button>
            <button type="button" onClick={clearDemo} className={GHOST_BUTTON_SM}>
              Clear
            </button>
            {selection && (
              <span className={TAG_ACCENT}>
                region · {Math.round(selection.w)} × {Math.round(selection.h)} px
              </span>
            )}
          </div>

          <p className={`${CAPTION} mt-2.5`}>Preview only — this demo captured nothing on your screen.</p>
        </div>

        <div className="grid min-h-[190px] place-items-center">
          {phase === "idle" ? (
            <div className="w-full rounded-lg border border-dashed border-border-strong px-5 py-10 text-center">
              <span className={CAPTION}>Nothing captured yet.</span>
            </div>
          ) : (
            <div className="w-full min-w-0 [&>section]:w-full [&>section]:max-w-full">
              <ResultPanel
                state={phase === "thinking" ? "thinking" : "complete"}
                net="local_only"
                answer={phase === "complete" ? DEMO_ANSWER : undefined}
                capture={phase === "complete" ? DEMO_CAPTURE : undefined}
                reducedMotion={reducedMotion}
              />
            </div>
          )}
        </div>
      </div>

      <p aria-live="polite" className={`${CAPTION} mt-4`}>
        {status}
      </p>
    </div>
  );
}
