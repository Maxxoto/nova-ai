import { CheckGlyph, MinusGlyph } from "./Glyphs";
import { BODY, CAPTION, EYEBROW, STEP_HEADING, WHY_LINE } from "./styles";

const PROMISES = [
  {
    id: "grounded",
    off: false,
    content: (
      <>
        Answers are grounded in <strong className="font-semibold text-foreground">your</strong> saved captures, not the
        open internet.
      </>
    ),
  },
  {
    id: "voice",
    off: false,
    content: <>Voice is processed live and deleted within a minute — there is no recording to find.</>,
  },
  {
    id: "account",
    off: true,
    content: <>No account, no cloud sync, no capture that you did not ask for.</>,
  },
] as const;

export default function WelcomeStep() {
  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3">
        <span className={EYEBROW}>Step 1 of 5</span>
        <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
          Point at anything. Speak. Keep your place.
        </h2>
        <p className={BODY}>
          Ruòxī lives in your menu bar. Press a hotkey, box something on screen — or just talk — and the answer appears
          next to your work instead of in another window.
        </p>
      </div>

      <p className={`${WHY_LINE} border-l-2 border-primary pl-3`}>
        I&rsquo;ll keep this on your machine — nothing leaves unless you send it.
      </p>

      <ul className="flex flex-col gap-0.5">
        {PROMISES.map((promise) => (
          <li key={promise.id} className="flex items-start gap-2.5 py-1">
            <span className={`mt-0.5 ${promise.off ? "text-muted-foreground" : "text-success"}`}>
              {promise.off ? <MinusGlyph className="h-3.5 w-3.5" /> : <CheckGlyph className="h-3.5 w-3.5" />}
            </span>
            <span className="font-ui text-[14px] leading-[1.45] text-muted-foreground">{promise.content}</span>
          </li>
        ))}
      </ul>

      <p className={CAPTION}>
        Five steps · granting is required to continue · skipping is allowed and says what it costs
      </p>
    </div>
  );
}
