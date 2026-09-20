import { CheckGlyph, MinusGlyph } from "./Glyphs";
import { BODY, EYEBROW, STEP_HEADING, WHY_LINE } from "./styles";

const PROMISES = [
  { id: "grounded", off: false, text: "Answers come from what you saved, not the open web." },
  { id: "voice", off: false, text: "Voice is deleted about a minute after you release the key." },
  { id: "account", off: true, text: "No account, no cloud sync, no capture you did not ask for." },
] as const;

export default function WelcomeStep() {
  return (
    <div className="flex flex-col">
      <span className={EYEBROW}>Step 1 of 6</span>
      <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
        Point at anything. Speak. Keep your place.
      </h2>
      <p className={`${BODY} mt-2.5 max-w-[52ch]`}>
        Ruoxi lives in your menu bar. Press a shortcut, box something on screen, or hold to talk — the answer appears
        next to your work.
      </p>

      <p className={`${WHY_LINE} mt-5`}>Everything stays on this Mac.</p>

      <ul className="mt-[18px]">
        {PROMISES.map((promise) => (
          <li key={promise.id} className="flex gap-[9px] py-[5px] text-[13px] leading-[1.45] text-body">
            <span className={`mt-0.5 flex-none ${promise.off ? "text-muted-foreground" : "text-success"}`}>
              {promise.off ? <MinusGlyph /> : <CheckGlyph />}
            </span>
            <span>{promise.text}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
