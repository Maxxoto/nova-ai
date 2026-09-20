import type { IdleConceptDef } from "../tray-icons";
import MenuBar from "./MenuBar";
import TrayMark from "./TrayMark";

interface ConceptCardProps {
  concept: IdleConceptDef;
  recommended: boolean;
}

function SizeBars({ concept, sizes, tall }: { concept: IdleConceptDef; sizes: number[]; tall?: boolean }) {
  return (
    <>
      <MenuBar theme="light" tall={tall}>
        <span className="menubar-item">
          {sizes.map((s) => (
            <TrayMark key={s} svg={concept.svg} size={s} />
          ))}
        </span>
      </MenuBar>
      <MenuBar theme="dark" tall={tall}>
        <span className="menubar-item">
          {sizes.map((s) => (
            <TrayMark key={s} svg={concept.svg} size={s} />
          ))}
        </span>
      </MenuBar>
    </>
  );
}

export default function ConceptCard({ concept, recommended }: ConceptCardProps) {
  return (
    <article className="flex flex-col gap-4 rounded-lg border border-border bg-card p-4 shadow-e1">
      <header className="flex items-baseline justify-between gap-2">
        <h3 className="font-ui text-sm font-semibold text-foreground">{concept.name}</h3>
        {recommended ? (
          <span className="rounded-sm bg-primary-soft px-1.5 py-0.5 font-companion text-[11px] font-bold text-primary-active">
            ships
          </span>
        ) : null}
      </header>

      <p className="text-[13px] leading-[1.45] text-muted-foreground">{concept.rationale}</p>

      <div className="flex flex-col gap-2">
        <SizeBars concept={concept} sizes={[16, 18, 22]} />
        <p className="font-mono text-[11px] text-muted-foreground">16 · 18 · 22 px — menu-bar scale</p>
      </div>

      <div className="flex flex-col gap-2">
        <SizeBars concept={concept} sizes={[32, 48]} tall />
        <p className="font-mono text-[11px] text-muted-foreground">32 · 48 px — loupe (@2x export check)</p>
      </div>
    </article>
  );
}
