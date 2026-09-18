import type { TrayAnim } from "../tray-icons";

interface TrayMarkProps {
  /** Raw SVG markup from src/tray-icons (colored or -template source). */
  svg: string;
  /** Rendered box in CSS px — the PNG matrix is generated from the same sources. */
  size: number;
  /** Motion-law loop to run when the state is genuinely live. */
  anim?: TrayAnim;
  /** Replaces the template's pure-black fills (simulated macOS tint). */
  tint?: string;
  className?: string;
}

export default function TrayMark({ svg, size, anim, tint, className }: TrayMarkProps) {
  const markup = tint ? svg.replace(/#000000/gi, tint) : svg;
  return (
    <span
      className={`tray-mark${className ? ` ${className}` : ""}`}
      data-anim={anim}
      style={{ width: size, height: size }}
      aria-hidden="true"
      dangerouslySetInnerHTML={{ __html: markup }}
    />
  );
}
