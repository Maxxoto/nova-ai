import { TRAY_STATES } from "../tray-icons";
import MenuBar from "./MenuBar";
import TrayMark from "./TrayMark";

const DARK_INK = "#f2e9de"; /* {colors.dark-ink} — simulated macOS tint for template on dark bars */

export default function TemplateCompare() {
  return (
    <div className="flex flex-col gap-4">
      <div className="overflow-hidden rounded-lg border border-border bg-card shadow-e1">
        <div className="grid grid-cols-[minmax(9rem,1.2fr)_repeat(3,minmax(0,1fr))] items-center gap-x-4 border-b border-border bg-muted px-4 py-2">
          <span className="font-ui text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">state</span>
          <span className="font-ui text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">colored</span>
          <span className="font-ui text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">template (black + alpha)</span>
          <span className="font-ui text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">template, OS-tinted (dark bar)</span>
        </div>
        {TRAY_STATES.map((def) => (
          <div
            key={def.id}
            className="grid grid-cols-[minmax(9rem,1.2fr)_repeat(3,minmax(0,1fr))] items-center gap-x-4 border-b border-border px-4 py-2.5 last:border-b-0"
          >
            <span className="truncate font-companion text-[13px] font-semibold text-foreground">{def.label}</span>
            <MenuBar theme="light">
              <span className="menubar-item">
                <TrayMark svg={def.svg} size={18} anim={def.anim} />
              </span>
            </MenuBar>
            <MenuBar theme="light">
              <span className="menubar-item">
                <TrayMark svg={def.templateSvg} size={18} />
              </span>
            </MenuBar>
            <MenuBar theme="dark">
              <span className="menubar-item">
                <TrayMark svg={def.templateSvg} size={18} tint={DARK_INK} />
              </span>
            </MenuBar>
          </div>
        ))}
      </div>

      <div className="flex flex-col gap-1.5 rounded-md border border-border bg-primary-soft p-4">
        <p className="font-ui text-sm font-semibold text-primary-active">Recommendation</p>
        <p className="text-[13px] leading-[1.45] text-primary-active">
          macOS ships the <span className="font-mono text-[12px]">-template</span> variants with{" "}
          <span className="font-mono text-[12px]">icon_as_template(true)</span> — pure-black + alpha lets the OS
          tint for light/dark, hover, and active states for free, and the marks were drawn to stay legible as a
          single silhouette. Windows and Linux ship the colored PNGs (no system tinting there). Every state keeps
          its word in the tray menu and tooltip either way — color alone is never the message.
        </p>
      </div>
    </div>
  );
}
