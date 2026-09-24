import { FOCUS_RING } from "./primitives";

export const PANEL_ANCHORS = ["tl", "tc", "tr", "ml", "mc", "mr", "bl", "bc", "br"] as const;
export type PanelAnchor = (typeof PANEL_ANCHORS)[number];

type AnchorSpec = { id: PanelAnchor; label: string; bar: string };

/**
 * Bars sit 3px in from the edge. The centre column/row uses the design's
 * 10px / 9px offsets, which centre the 10×6 bar in the 30×24 cell.
 */
const ANCHORS: readonly AnchorSpec[] = [
  { id: "tl", label: "Top left", bar: "left-[3px] top-[3px]" },
  { id: "tc", label: "Top centre", bar: "left-[10px] top-[3px]" },
  { id: "tr", label: "Top right", bar: "right-[3px] top-[3px]" },
  { id: "ml", label: "Middle left", bar: "left-[3px] top-[9px]" },
  { id: "mc", label: "Centre", bar: "left-[10px] top-[9px]" },
  { id: "mr", label: "Middle right", bar: "right-[3px] top-[9px]" },
  { id: "bl", label: "Bottom left", bar: "bottom-[3px] left-[3px]" },
  { id: "bc", label: "Bottom centre", bar: "bottom-[3px] left-[10px]" },
  { id: "br", label: "Bottom right", bar: "bottom-[3px] right-[3px]" },
];

export default function AnchorPicker({
  value,
  onChange,
}: {
  value: PanelAnchor;
  onChange: (next: PanelAnchor) => void;
}) {
  return (
    <div role="group" aria-label="Panel corner" className="grid grid-cols-[repeat(3,30px)] gap-1">
      {ANCHORS.map((anchor) => {
        const pressed = anchor.id === value;
        return (
          <button
            key={anchor.id}
            type="button"
            aria-pressed={pressed}
            aria-label={anchor.label}
            onClick={() => onChange(anchor.id)}
            className={`relative h-6 w-[30px] rounded-[5px] border p-0 transition-colors duration-[120ms] ease-in-out ${FOCUS_RING} ${
              pressed
                ? "border-primary bg-primary-soft"
                : "border-border-strong bg-card hover:border-foreground"
            }`}
          >
            <i
              aria-hidden="true"
              className={`absolute h-[6px] w-[10px] rounded-[1.5px] ${anchor.bar} ${
                pressed ? "bg-primary-active" : "bg-muted-foreground"
              }`}
            />
          </button>
        );
      })}
    </div>
  );
}
