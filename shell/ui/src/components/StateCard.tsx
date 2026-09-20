import type { TrayStateDef } from "../tray-icons";
import TrayMark from "./TrayMark";

interface StateCardProps {
  def: TrayStateDef;
  theme: "light" | "dark";
  reducedMotion: boolean;
}

export default function StateCard({ def, theme, reducedMotion }: StateCardProps) {
  const hex = theme === "light" ? def.hexLight : def.hexDark;
  return (
    <div className="flex items-center gap-3 rounded-md border border-border bg-card p-3">
      <TrayMark svg={def.svg} size={24} anim={def.anim} />
      <div className="flex min-w-0 flex-col gap-0.5">
        <span className="truncate font-companion text-[13px] font-semibold text-foreground">{def.label}</span>
        <span className="font-mono text-[11px] text-muted-foreground">
          {def.colorToken} · {hex}
        </span>
        {def.anim ? (
          <span className="font-mono text-[11px] text-muted-foreground">
            {reducedMotion ? `frozen · ${def.loopName}` : `live · ${def.loopName}`}
          </span>
        ) : null}
      </div>
    </div>
  );
}
