import type { ReactNode } from "react";

interface SectionProps {
  eyebrow: string;
  title: string;
  lede?: string;
  children: ReactNode;
}

export default function Section({ eyebrow, title, lede, children }: SectionProps) {
  return (
    <section className="flex flex-col gap-5">
      <header className="flex flex-col gap-1.5">
        <p className="font-mono text-[11px] font-medium uppercase tracking-[0.14em] text-muted-foreground">
          {eyebrow}
        </p>
        <h2 className="font-ui text-[17px] font-semibold leading-[1.3] tracking-[-0.01em] text-foreground">
          {title}
        </h2>
        {lede ? <p className="max-w-[68ch] text-sm leading-[1.45] text-muted-foreground">{lede}</p> : null}
      </header>
      {children}
    </section>
  );
}
