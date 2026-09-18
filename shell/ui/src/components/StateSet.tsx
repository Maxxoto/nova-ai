import { TRAY_STATES, type TrayStateDef } from "../tray-icons";
import MenuBar from "./MenuBar";
import StateCard from "./StateCard";
import TrayMark from "./TrayMark";

function ThemePanel({
  theme,
  reducedMotion,
}: {
  theme: "light" | "dark";
  reducedMotion: boolean;
}) {
  return (
    <div className={theme === "dark" ? "dark" : ""}>
      <div className="flex flex-col gap-3 rounded-lg border border-border bg-background p-4">
        <div className="flex items-center justify-between">
          <h3 className="font-ui text-sm font-semibold text-foreground">
            {theme === "light" ? "Dawn · light" : "Night before dawn · dark"}
          </h3>
          <MenuBar theme={theme}>
            <span className="menubar-item">
              <TrayMark svg={TRAY_STATES[0].svg} size={18} anim={TRAY_STATES[0].anim} />
            </span>
          </MenuBar>
        </div>
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
          {TRAY_STATES.map((def: TrayStateDef) => (
            <StateCard key={def.id} def={def} theme={theme} reducedMotion={reducedMotion} />
          ))}
        </div>
      </div>
    </div>
  );
}

export default function StateSet({ reducedMotion }: { reducedMotion: boolean }) {
  return (
    <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
      <ThemePanel theme="light" reducedMotion={reducedMotion} />
      <ThemePanel theme="dark" reducedMotion={reducedMotion} />
    </div>
  );
}
