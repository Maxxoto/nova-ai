/**
 * Onboarding · Step 5 of 6 — Models.
 *
 * Layout follows the design (`.set-row` trio + offline warn banner); the model
 * selection / downloads / LLM config are the REAL Settings → Models wiring,
 * shared verbatim through `../models/ModelRows` so the two surfaces cannot
 * drift. The design's mocked DeepSeek 2-option select is intentionally NOT
 * reproduced — we surface the real OpenAI-compatible config instead.
 *
 * Rendered by the orchestrator with:
 *   { offline, onTurnOffOffline }
 */

import { LlmConfigPanel, LlmRow, SttRow, TtsRow, useModelSetup } from "../models/ModelRows";
import { BODY, EYEBROW, FOCUS_RING, STEP_HEADING } from "./styles";

export default function ModelsStep({
  offline,
  onTurnOffOffline,
}: {
  offline: boolean;
  onTurnOffOffline: () => void;
}) {
  const setup = useModelSetup();

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3">
        <span className={EYEBROW}>Step 5 of 6</span>
        <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
          Models.
        </h2>
        <p className={BODY}>
          Speech, voice and answers. On-device models download once and stay on this Mac.
        </p>
      </div>

      <div className="flex flex-col border-b border-t border-border">
        <SttRow setup={setup} />
        <TtsRow setup={setup} help="Kokoro, running on this Mac. Needed to read answers aloud." />
        {setup.testError ? (
          <p className="pb-1 pt-0.5 font-ui text-[12px] leading-[1.45] text-destructive" role="status">
            {setup.testError}
          </p>
        ) : null}
        <LlmRow setup={setup} />
      </div>

      {setup.llmOpen ? <LlmConfigPanel setup={setup} /> : null}

      {offline && setup.llm?.base_url ? (
        <div
          role="status"
          className="flex items-center gap-2.5 rounded border border-warning bg-warning/10 p-3"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={1.8}
            strokeLinecap="round"
            aria-hidden="true"
            className="h-4 w-4 flex-none text-warning"
          >
            <path d="M12 8v5m0 3h.01" />
            <circle cx="12" cy="12" r="9" />
          </svg>
          <span className="flex-1 font-ui text-[13px] leading-[1.45] text-warning">
            {setup.endpointHost} is unavailable while Offline mode is on. Turn it off in the previous
            step, or change it later in Settings.
          </span>
          <button
            type="button"
            onClick={onTurnOffOffline}
            className={`flex-none rounded px-2 py-1 font-ui text-[12px] font-semibold text-warning transition-colors duration-200 hover:bg-warning/10 ${FOCUS_RING}`}
          >
            Turn off offline
          </button>
        </div>
      ) : null}
    </div>
  );
}
