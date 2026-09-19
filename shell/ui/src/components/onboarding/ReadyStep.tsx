import { BODY, CAPTION, EYEBROW, KBD, STEP_HEADING } from "./styles";
import { PTT_KEY_LABEL, REGISTERED_CAPTURE_ACCELERATORS } from "./types";

function KeyCaps({ keys }: { keys: readonly string[] }) {
  return (
    <span className="flex flex-none items-center gap-1">
      {keys.map((key) => (
        <kbd key={key} className={KBD}>
          {key}
        </kbd>
      ))}
    </span>
  );
}

const REGION_KEYS = REGISTERED_CAPTURE_ACCELERATORS[0].keys.split("+");

export default function ReadyStep({ pttHotkey = PTT_KEY_LABEL }: { pttHotkey?: string }) {
  const pttKeys = pttHotkey.split("+");
  const pttNote = pttHotkey === PTT_KEY_LABEL ? "" : " (next app start)";

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3">
        <span className={EYEBROW}>Step 5 of 5</span>
        <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
          You&rsquo;re set up. Three keys to remember.
        </h2>
        <p className={BODY}>Everything is on your machine, and the tray is where you check on me.</p>
      </div>

      <div className="flex flex-col gap-1 rounded-lg border border-border bg-muted px-4 py-2">
        <div className="flex items-center justify-between gap-4 border-b border-border py-3">
          <div className="flex min-w-0 flex-col">
            <span className="font-ui text-[13px] font-semibold text-foreground">Ask by voice, anywhere</span>
            <span className={CAPTION}>hold to talk{pttNote}</span>
          </div>
          <KeyCaps keys={pttKeys} />
        </div>
        <div className="flex items-center justify-between gap-4 border-b border-border py-3">
          <div className="flex min-w-0 flex-col">
            <span className="font-ui text-[13px] font-semibold text-foreground">Box something and ask about it</span>
            <span className={CAPTION}>{REGISTERED_CAPTURE_ACCELERATORS[0].intent}</span>
          </div>
          <KeyCaps keys={REGION_KEYS} />
        </div>
        <div className="flex items-center justify-between gap-4 py-3">
          <span className="font-ui text-[13px] font-semibold text-foreground">
            The tray — brief, pause, offline
          </span>
          <span className={CAPTION}>menu bar</span>
        </div>
      </div>

      <p className={CAPTION}>
        Accelerators registered in this build:{" "}
        {REGISTERED_CAPTURE_ACCELERATORS.map((item) => item.keys).join(" · ")} · push-to-talk {pttHotkey}
        {pttNote}.
      </p>
    </div>
  );
}
