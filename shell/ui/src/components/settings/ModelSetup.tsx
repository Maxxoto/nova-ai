import { FOCUS_RING, Row, Section, Tag } from "./primitives";
import {
  gbValue,
  LlmConfigPanel,
  LlmRow,
  SttRow,
  TtsRow,
  useModelSetup,
} from "../models/ModelRows";

/**
 * Settings → Models. The rows themselves live in `../models/ModelRows` so the
 * onboarding Models step shares the exact same wiring (catalog, downloads,
 * pickers, LLM expander) and the two surfaces cannot drift.
 */
export function ModelsSection({
  offline,
  onTurnOffOffline,
}: {
  offline: boolean;
  onTurnOffOffline: () => void;
}) {
  const setup = useModelSetup();
  const ttsHelp =
    setup.engine === "kokoro"
      ? "Kokoro, running on this Mac."
      : "macOS system voice — always available.";

  return (
    <Section id="models" title="Models">
      <div className="flex flex-col">
        <SttRow setup={setup} />
        <TtsRow setup={setup} help={ttsHelp} />
        {setup.testError ? (
          <p className="pb-1 pt-0.5 font-ui text-[12px] leading-[1.45] text-destructive" role="status">
            {setup.testError}
          </p>
        ) : null}
        <Row
          label="Speaking rate"
          help="How fast answers are read."
          side={
            <>
              <input
                type="range"
                min={0.8}
                max={1.6}
                step={0.1}
                value={setup.rate}
                onChange={(event) => setup.changeRate(Number(event.target.value))}
                aria-label="Speaking rate"
                className={`h-1 w-[132px] accent-primary ${FOCUS_RING}`}
              />
              <span className="min-w-[34px] text-right font-mono text-[11px] text-muted-foreground">
                {setup.rate.toFixed(1)}×
              </span>
            </>
          }
        />
        <LlmRow setup={setup} />
        <Row
          label="On-device models"
          help={setup.storeLine}
          side={<Tag>{gbValue(setup.installedBytes)} GB</Tag>}
        />
      </div>

      {setup.llmOpen ? <LlmConfigPanel setup={setup} /> : null}

      {setup.llm?.base_url ? (
        <div
          role="status"
          className={`flex items-center gap-2.5 rounded border p-3 ${
            offline ? "border-warning" : "border-border-strong bg-muted"
          }`}
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={1.8}
            strokeLinecap="round"
            aria-hidden="true"
            className={`h-4 w-4 flex-none ${offline ? "text-warning" : "text-muted-foreground"}`}
          >
            <path d="M12 8v5m0 3h.01" />
            <circle cx="12" cy="12" r="9" />
          </svg>
          <span
            className={`flex-1 font-ui text-[13px] leading-[1.45] ${
              offline ? "text-warning" : "text-muted-foreground"
            }`}
          >
            {offline
              ? `Offline mode is on, so ${setup.endpointHost} is unavailable. Captures and memory still work.`
              : `${setup.endpointHost} receives the capture and your question. Nothing else leaves this Mac.`}
          </span>
          {offline ? (
            <button
              type="button"
              onClick={onTurnOffOffline}
              className={`flex-none rounded px-2 py-1 font-ui text-[12px] font-semibold text-warning transition-colors duration-200 hover:bg-warning/10 ${FOCUS_RING}`}
            >
              Turn off offline
            </button>
          ) : null}
        </div>
      ) : null}
    </Section>
  );
}
