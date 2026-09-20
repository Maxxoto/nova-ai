/**
 * Shared Models surface — the real STT/TTS/LLM wiring used by BOTH
 * Settings → Models (`settings/ModelSetup.tsx`) and the onboarding Models step
 * (`onboarding/ModelsStep.tsx`), so the two surfaces can never drift.
 *
 * Data comes from the shipped Tauri commands (`stt_catalog`,
 * `tts_model_catalog`, `tts_kokoro_voices`, `tts_list_voices`, `get_settings`,
 * `tts_bundle_bytes`) and the `model:progress` / `model:done` / `model:error`
 * events. Nothing here is mocked; outside Tauri `invokeTauriAsync` returns
 * null and the rows fall back to the static "Available in the desktop app."
 * state.
 */

import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { invokeTauriAsync, listenTauri } from "../../tauri";
import {
  CHANGE_BUTTON,
  FOCUS_RING,
  GHOST_BUTTON_SM,
  PRIMARY_BUTTON_SM,
  Row,
  Tag,
} from "../settings/primitives";

export type ModelKind = "stt" | "tts_model" | "tts_voices";

export type CatalogModel = {
  id: string;
  kind: ModelKind;
  family: string;
  name: string;
  file: string;
  size_bytes: number;
  zh: string;
  note: string;
  downloaded: boolean;
  selected: boolean;
};

export type SystemVoice = { name: string; lang: string };
export type KokoroVoice = { id: string; label: string };
export type LlmConfig = { base_url: string; model: string; vision_model: string; api_key_set: boolean };
export type TestResult = { ok: boolean; message?: string; model?: string; reply?: string };
export type ModelProgress = { id: string; downloaded: number; total: number };

/** GB with up to two decimals, trailing zeros trimmed — the design's fmt(). */
export function gbValue(bytes: number): string {
  return (Math.round((bytes / 1_000_000_000) * 100) / 100).toString();
}

export function hostLabel(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return "the configured endpoint";
  }
}

export function familyLabel(family: string): string {
  if (!family) return "Models";
  return family.charAt(0).toUpperCase() + family.slice(1);
}

export type PickerOption = { value: string; label: string; disabled?: boolean };
export type PickerGroup = { label: string; options: PickerOption[] };

export function Picker({
  ariaLabel,
  value,
  groups,
  onChange,
  disabled = false,
}: {
  ariaLabel: string;
  value: string;
  groups: PickerGroup[];
  onChange: (next: string) => void;
  disabled?: boolean;
}) {
  return (
    <span className="relative inline-flex items-center">
      <select
        aria-label={ariaLabel}
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
        className={`h-7 max-w-[230px] appearance-none rounded border border-border-strong bg-card pl-2.5 pr-7 font-ui text-[12px] font-medium text-foreground transition-colors duration-200 hover:border-primary disabled:cursor-default disabled:opacity-50 ${FOCUS_RING}`}
      >
        {groups.map((group) => (
          <optgroup key={group.label} label={group.label}>
            {group.options.map((option) => (
              <option key={option.value} value={option.value} disabled={option.disabled}>
                {option.label}
              </option>
            ))}
          </optgroup>
        ))}
      </select>
      <svg
        className="pointer-events-none absolute right-2 h-3 w-3 text-muted-foreground"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="m7 10 5 5 5-5" />
      </svg>
    </span>
  );
}

export function Field({
  label,
  value,
  onChange,
  placeholder,
  type = "text",
}: {
  label: string;
  value: string;
  onChange: (next: string) => void;
  placeholder?: string;
  type?: string;
}) {
  return (
    <label className="flex flex-col gap-1">
      <span className="font-mono text-[11px] uppercase tracking-[0.06em] text-muted-foreground">
        {label}
      </span>
      <input
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        spellCheck={false}
        className="rounded-lg border border-border bg-background px-3 py-2 font-mono text-[13px] text-foreground outline-none placeholder:text-muted-foreground focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-ring"
      />
    </label>
  );
}

export function DownloadProgress({
  pct,
  label,
  onCancel,
}: {
  pct: number;
  label: string;
  onCancel: () => void;
}) {
  return (
    <span className="flex flex-none items-center gap-2">
      <span className="font-ui text-[12px] leading-none text-muted-foreground">Downloading…</span>
      <span
        role="progressbar"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={pct}
        className="h-1 w-[104px] overflow-hidden rounded-pill bg-muted"
      >
        <span
          className="block h-full bg-primary transition-[width] duration-150 ease-linear"
          style={{ width: `${pct}%` }}
        />
      </span>
      <span className="min-w-[36px] text-right font-mono text-[11px] tabular-nums text-muted-foreground">
        {pct}%
      </span>
      <button
        type="button"
        onClick={onCancel}
        className={`rounded px-1 py-0.5 font-ui text-[12px] text-muted-foreground underline decoration-border-strong underline-offset-2 transition-colors duration-200 hover:text-foreground ${FOCUS_RING}`}
      >
        Cancel
      </button>
    </span>
  );
}

/**
 * The whole Models data layer: catalogs, download progress, TTS voices/engine,
 * and the LLM config. Both the Settings section and the onboarding step mount
 * this so their wiring is byte-for-byte the same.
 */
export function useModelSetup() {
  const [sttModels, setSttModels] = useState<CatalogModel[]>([]);
  const [ttsModels, setTtsModels] = useState<CatalogModel[]>([]);
  const [progress, setProgress] = useState<Record<string, number>>({});
  const [ttsBundleBytes, setTtsBundleBytes] = useState(0);

  const [engine, setEngine] = useState("system");
  const [voice, setVoice] = useState("");
  const [rate, setRate] = useState(1);
  const [systemVoices, setSystemVoices] = useState<SystemVoice[]>([]);
  const [kokoroVoices, setKokoroVoices] = useState<KokoroVoice[]>([]);

  const [sttChoice, setSttChoice] = useState<string | null>(null);
  const [ttsRequested, setTtsRequested] = useState(false);
  const [testState, setTestState] = useState<"idle" | "testing">("idle");
  const [testError, setTestError] = useState<string>();

  const [llm, setLlm] = useState<LlmConfig | null>(null);
  const [llmOpen, setLlmOpen] = useState(false);
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [visionModel, setVisionModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [keySet, setKeySet] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestResult>();

  const sttDlRef = useRef<string | null>(null);
  const [sttDlId, setSttDlId] = useState<string | null>(null);
  const pendingTtsSelectRef = useRef<string | null>(null);
  const pendingTtsCancelRef = useRef<string[]>([]);
  const rateTimerRef = useRef<number | null>(null);

  const setSttDownload = (id: string | null) => {
    sttDlRef.current = id;
    setSttDlId(id);
  };

  const refreshStt = () => {
    invokeTauriAsync("stt_catalog")?.then(
      (raw) => setSttModels((raw as CatalogModel[]) ?? []),
      () => setSttModels([]),
    );
  };
  const refreshTtsModels = () => {
    invokeTauriAsync("tts_model_catalog")?.then(
      (raw) => setTtsModels((raw as CatalogModel[]) ?? []),
      () => setTtsModels([]),
    );
  };
  const refreshVoices = () => {
    invokeTauriAsync("tts_kokoro_voices")?.then(
      (raw) => setKokoroVoices((raw as KokoroVoice[]) ?? []),
      () => setKokoroVoices([]),
    );
    invokeTauriAsync("tts_list_voices")?.then(
      (raw) => setSystemVoices((raw as SystemVoice[]) ?? []),
      () => setSystemVoices([]),
    );
  };
  const refreshLlm = () => {
    invokeTauriAsync("get_settings")?.then(
      (raw) => {
        const settings = raw as {
          tts?: { engine?: string; voice?: string; rate?: number };
          llm?: LlmConfig;
        } | null;
        if (settings?.tts) {
          setEngine(settings.tts.engine ?? "system");
          setVoice(settings.tts.voice ?? "");
          setRate(typeof settings.tts.rate === "number" ? settings.tts.rate : 1);
        }
        const llmConfig = settings?.llm ?? null;
        setLlm(llmConfig);
        if (llmConfig) {
          setBaseUrl(llmConfig.base_url ?? "");
          setModel(llmConfig.model ?? "");
          setVisionModel(llmConfig.vision_model ?? "");
          setKeySet(Boolean(llmConfig.api_key_set));
        }
      },
      () => undefined,
    );
  };

  useEffect(() => {
    refreshStt();
    refreshTtsModels();
    refreshVoices();
    refreshLlm();
    invokeTauriAsync("tts_bundle_bytes")?.then(
      (raw) => setTtsBundleBytes(typeof raw === "number" ? raw : 0),
      () => undefined,
    );
    let active = true;
    const offs: Array<() => void> = [];
    const track = (off: () => void) => {
      if (active) offs.push(off);
      else off();
    };
    void (async () => {
      track(
        await listenTauri<ModelProgress>("model:progress", (p) => {
          if (!p || p.total <= 0) return;
          setProgress((prev) => ({ ...prev, [p.id]: p.downloaded / p.total }));
        }),
      );
      track(
        await listenTauri<{ id: string }>("model:done", (p) => {
          setProgress((prev) => {
            const next = { ...prev };
            delete next[p.id];
            return next;
          });
          refreshStt();
          refreshTtsModels();
          refreshVoices();
          if (sttDlRef.current === p.id) {
            setSttDownload(null);
            invokeTauriAsync("stt_select", { id: p.id });
          }
          if (pendingTtsSelectRef.current === p.id) {
            pendingTtsSelectRef.current = null;
            invokeTauriAsync("tts_model_select", { id: p.id });
          }
        }),
      );
      track(
        await listenTauri<{ id: string; message: string }>("model:error", (p) => {
          setProgress((prev) => {
            const next = { ...prev };
            delete next[p?.id ?? ""];
            return next;
          });
          if (sttDlRef.current === p?.id) {
            setSttDownload(null);
            setSttChoice(null);
          }
          if (pendingTtsSelectRef.current === p?.id) pendingTtsSelectRef.current = null;
          if (sttDlRef.current !== p?.id) setTtsRequested(false);
        }),
      );
    })();
    return () => {
      active = false;
      offs.forEach((off) => off());
      if (rateTimerRef.current !== null) window.clearTimeout(rateTimerRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const sttSelected = sttModels.find((m) => m.selected);
  const sttChosenId = sttChoice ?? sttSelected?.id ?? "";
  const sttChosen = sttModels.find((m) => m.id === sttChosenId);
  const sttChosenMissing = Boolean(sttChosen && !sttChosen.downloaded);
  const sttPct = sttDlId !== null ? Math.round((progress[sttDlId] ?? 0) * 100) : 0;

  const kokoroModelEntry =
    ttsModels.find((m) => m.kind === "tts_model" && m.selected) ??
    ttsModels.find((m) => m.kind === "tts_model" && m.downloaded);
  const ttsVoicesEntry = ttsModels.find((m) => m.kind === "tts_voices");
  const kokoroReady = Boolean(kokoroModelEntry?.downloaded && ttsVoicesEntry?.downloaded);
  const ttsDownloading = ttsRequested || ttsModels.some((m) => progress[m.id] !== undefined);
  const ttsActiveIds = ttsModels.filter((m) => progress[m.id] !== undefined).map((m) => m.id);
  const ttsPct =
    ttsActiveIds.length > 0
      ? Math.round(
          (ttsActiveIds.reduce((sum, id) => sum + (progress[id] ?? 0), 0) / ttsActiveIds.length) *
            100,
        )
      : 0;

  useEffect(() => {
    if (kokoroReady) setTtsRequested(false);
  }, [kokoroReady]);

  const pickSttModel = (id: string) => {
    const entry = sttModels.find((m) => m.id === id);
    if (!entry) return;
    setSttChoice(id);
    if (entry.downloaded) {
      invokeTauriAsync("stt_select", { id })?.then(refreshStt);
    }
  };

  const startSttDownload = () => {
    const id = sttChosen?.id;
    if (!id) return;
    setSttDownload(id);
    invokeTauriAsync("stt_download", { id })?.catch(() => setSttDownload(null));
  };

  const cancelSttDownload = () => {
    const id = sttDlRef.current;
    if (!id) return;
    invokeTauriAsync("download_cancel", { id });
    setSttDownload(null);
    setSttChoice(null);
    setProgress((prev) => {
      const next = { ...prev };
      delete next[id];
      return next;
    });
  };

  const pickVoice = (value: string) => {
    const separator = value.indexOf(":");
    if (separator < 0) return;
    const kind = value.slice(0, separator);
    const picked = value.slice(separator + 1);
    if (kind !== "kokoro" && kind !== "system") return;
    setEngine(kind);
    setVoice(picked);
    setTestError(undefined);
    invokeTauriAsync("tts_save_engine", { engine: kind });
    invokeTauriAsync("tts_save_voice", { voice: picked });
  };

  const downloadKokoro = () => {
    setTtsRequested(true);
    const voicePackIds = ttsModels.filter((m) => m.kind === "tts_voices").map((m) => m.id);
    invokeTauriAsync("tts_download_kokoro")?.then(
      (raw) => {
        if (typeof raw === "string") {
          pendingTtsSelectRef.current = raw;
          pendingTtsCancelRef.current = [raw, ...voicePackIds];
        } else {
          pendingTtsCancelRef.current = voicePackIds;
        }
      },
      () => setTtsRequested(false),
    );
  };

  const cancelKokoroDownload = () => {
    const ids = new Set<string>(pendingTtsCancelRef.current);
    for (const id of ttsActiveIds) ids.add(id);
    for (const id of ids) invokeTauriAsync("download_cancel", { id });
    pendingTtsSelectRef.current = null;
    pendingTtsCancelRef.current = [];
    setTtsRequested(false);
    setProgress((prev) => {
      const next = { ...prev };
      for (const id of ids) delete next[id];
      return next;
    });
  };

  const changeRate = (next: number) => {
    setRate(next);
    if (rateTimerRef.current !== null) window.clearTimeout(rateTimerRef.current);
    rateTimerRef.current = window.setTimeout(() => {
      invokeTauriAsync("tts_save_rate", { rate: next });
    }, 250);
  };

  const testVoice = () => {
    setTestState("testing");
    setTestError(undefined);
    const pending =
      engine === "kokoro"
        ? invokeTauriAsync("tts_synthesize", { text: "", voice })
        : invokeTauriAsync("tts_test_voice", { voice, text: "" });
    if (!pending) {
      setTestState("idle");
      return;
    }
    pending.then(
      () => setTestState("idle"),
      (raw: unknown) => {
        setTestState("idle");
        setTestError(
          typeof raw === "string" && raw.trim().length > 0
            ? raw
            : "The voice didn't play — see the app logs for details.",
        );
      },
    );
  };

  const saveLlm = () => {
    invokeTauriAsync("llm_save_config", {
      baseUrl,
      model,
      visionModel,
      apiKey: apiKey.trim() ? apiKey : null,
    })?.then(
      (raw) => {
        const saved = raw as LlmConfig | null;
        setKeySet(Boolean(saved?.api_key_set));
        setApiKey("");
        setLlm((prev) => (prev && saved ? { ...prev, ...saved } : saved));
      },
      () => setTestResult({ ok: false, message: "save failed" }),
    );
  };

  const testLlm = () => {
    setTesting(true);
    setTestResult(undefined);
    invokeTauriAsync("llm_test", {
      baseUrl: baseUrl.trim() || null,
      model: model.trim() || null,
      apiKey: apiKey.trim() || null,
    })?.then(
      (raw) => setTestResult((raw as TestResult) ?? { ok: false, message: "no response" }),
      () => setTestResult({ ok: false, message: "the brain isn't responding — try again in a moment" }),
    );
    setTesting(false);
  };

  const clearLlmKey = () => {
    invokeTauriAsync("llm_clear_api_key")?.then(() => setKeySet(false));
  };

  const voiceValue =
    engine === "kokoro" && voice
      ? `kokoro:${voice}`
      : engine === "system" && voice
        ? `system:${voice}`
        : "";

  const voiceOptions: PickerOption[] = [];
  if (kokoroReady && kokoroVoices.length > 0) {
    for (const item of kokoroVoices) {
      voiceOptions.push({ value: `kokoro:${item.id}`, label: item.label });
    }
  }
  for (const item of systemVoices) {
    voiceOptions.push({ value: `system:${item.name}`, label: `${item.name} (${item.lang})` });
  }
  const voiceGroups: PickerGroup[] = [
    {
      label: "Voices",
      options:
        voiceValue === "" && voiceOptions.length > 0
          ? [{ value: "", label: "Choose a voice", disabled: true }, ...voiceOptions]
          : voiceOptions,
    },
  ];

  const sttGroups: PickerGroup[] = [];
  for (const entry of sttModels) {
    const label = familyLabel(entry.family);
    const group = sttGroups.find((item) => item.label === label);
    const option: PickerOption = { value: entry.id, label: entry.name };
    if (group) group.options.push(option);
    else sttGroups.push({ label, options: [option] });
  }
  if (!sttChosen && sttGroups.length > 0) {
    sttGroups[0].options.unshift({ value: "", label: "Choose a model", disabled: true });
  }

  const installedBytes = [...sttModels, ...ttsModels]
    .filter((m) => m.downloaded)
    .reduce((sum, m) => sum + m.size_bytes, 0);
  const runningBytes =
    (sttDlId ? (sttModels.find((m) => m.id === sttDlId)?.size_bytes ?? 0) : 0) +
    (ttsDownloading ? ttsBundleBytes : 0);
  const incomingBytes = sttChosenMissing && sttChosen ? sttChosen.size_bytes : 0;
  const storeLine =
    runningBytes > 0
      ? `${gbValue(installedBytes)} GB installed · ${gbValue(runningBytes)} GB downloading.`
      : incomingBytes > 0
        ? `${gbValue(installedBytes)} GB installed · ${gbValue(incomingBytes)} GB to download.`
        : installedBytes > 0
          ? `${gbValue(installedBytes)} GB installed and kept on this Mac. Switching models downloads the new one once.`
          : "Nothing downloaded yet — files arrive only with your consent.";

  const endpointHost = llm?.base_url ? hostLabel(llm.base_url) : null;

  return {
    sttModels,
    ttsModels,
    ttsBundleBytes,
    engine,
    voice,
    rate,
    systemVoices,
    kokoroVoices,
    sttDlId,
    testState,
    testError,
    llm,
    llmOpen,
    setLlmOpen,
    baseUrl,
    setBaseUrl,
    model,
    setModel,
    visionModel,
    setVisionModel,
    apiKey,
    setApiKey,
    keySet,
    testing,
    testResult,
    sttChosenId,
    sttChosen,
    sttChosenMissing,
    sttPct,
    kokoroReady,
    ttsDownloading,
    ttsPct,
    pickSttModel,
    startSttDownload,
    cancelSttDownload,
    pickVoice,
    downloadKokoro,
    cancelKokoroDownload,
    changeRate,
    testVoice,
    saveLlm,
    testLlm,
    clearLlmKey,
    voiceValue,
    voiceOptions,
    voiceGroups,
    sttGroups,
    installedBytes,
    storeLine,
    endpointHost,
  };
}

export type ModelSetup = ReturnType<typeof useModelSetup>;

/** Speech to text — real `stt_catalog` + `stt_download` / `stt_select`. */
export function SttRow({ setup }: { setup: ModelSetup }) {
  const {
    sttModels,
    sttDlId,
    sttPct,
    sttChosen,
    sttChosenMissing,
    sttChosenId,
    sttGroups,
    cancelSttDownload,
    startSttDownload,
    pickSttModel,
  } = setup;

  return (
    <Row
      label="Speech to text"
      help="Whisper or Parakeet, running on this Mac."
      side={
        <>
          {sttModels.length > 0 ? (
            sttDlId !== null ? (
              <DownloadProgress
                pct={sttPct}
                label="Downloading speech to text model"
                onCancel={cancelSttDownload}
              />
            ) : (
              <>
                {sttChosen ? (
                  sttChosen.downloaded ? (
                    <Tag tone="ok">On-Device</Tag>
                  ) : (
                    <Tag>Not Installed · {gbValue(sttChosen.size_bytes)} GB</Tag>
                  )
                ) : null}
                {sttChosenMissing ? (
                  <button type="button" onClick={startSttDownload} className={CHANGE_BUTTON}>
                    Download
                  </button>
                ) : null}
              </>
            )
          ) : null}
          {sttModels.length > 0 ? (
            <Picker
              ariaLabel="Speech to text model"
              value={sttChosenId}
              onChange={pickSttModel}
              disabled={sttDlId !== null}
              groups={sttGroups}
            />
          ) : (
            <span className="font-ui text-[12px] text-muted-foreground">
              Available in the desktop app.
            </span>
          )}
        </>
      }
    />
  );
}

/** Text to speech — real Kokoro download bundle + voice list. */
export function TtsRow({ setup, help }: { setup: ModelSetup; help: ReactNode }) {
  const {
    ttsModels,
    ttsDownloading,
    ttsPct,
    kokoroReady,
    ttsBundleBytes,
    engine,
    voice,
    voiceOptions,
    voiceValue,
    voiceGroups,
    cancelKokoroDownload,
    downloadKokoro,
    pickVoice,
    testVoice,
    testState,
  } = setup;

  return (
    <Row
      label="Text to speech"
      help={help}
      side={
        <>
          {ttsModels.length > 0 ? (
            ttsDownloading ? (
              <DownloadProgress
                pct={ttsPct}
                label="Downloading voice model"
                onCancel={cancelKokoroDownload}
              />
            ) : (
              <>
                {!kokoroReady ? (
                  <Tag>Not Installed · {gbValue(ttsBundleBytes)} GB</Tag>
                ) : engine === "kokoro" ? (
                  <Tag tone="ok">On-Device</Tag>
                ) : (
                  <Tag>System</Tag>
                )}
                {!kokoroReady ? (
                  <button type="button" onClick={downloadKokoro} className={CHANGE_BUTTON}>
                    Download
                  </button>
                ) : null}
              </>
            )
          ) : null}
          {voiceOptions.length > 0 ? (
            <Picker
              ariaLabel="Text to speech voice"
              value={voiceValue}
              onChange={pickVoice}
              disabled={ttsDownloading}
              groups={voiceGroups}
            />
          ) : (
            <span className="font-ui text-[12px] text-muted-foreground">
              Available in the desktop app.
            </span>
          )}
          {voice ? (
            <button
              type="button"
              onClick={testVoice}
              disabled={testState === "testing"}
              className={GHOST_BUTTON_SM}
            >
              {testState === "testing" ? "Testing…" : "Test"}
            </button>
          ) : null}
        </>
      }
    />
  );
}

/**
 * Language model summary row. The design mock shows a 2-option DeepSeek
 * select; we keep the real config surface instead — host + configured model +
 * a Configure… toggle that expands {@link LlmConfigPanel}.
 */
export function LlmRow({ setup }: { setup: ModelSetup }) {
  const { llm, llmOpen, setLlmOpen, endpointHost } = setup;
  return (
    <Row
      label="Language model"
      help={
        endpointHost
          ? `${endpointHost} in the cloud. Each question sends the capture and your question.`
          : "No endpoint yet — add any OpenAI-compatible service."
      }
      side={
        <>
          <Tag tone="warn">Cloud</Tag>
          {llm?.model ? (
            <span className="max-w-[160px] truncate font-mono text-[11px] text-muted-foreground">
              {llm.model}
            </span>
          ) : null}
          <button type="button" onClick={() => setLlmOpen((open) => !open)} className={CHANGE_BUTTON}>
            {llmOpen ? "Close" : "Configure…"}
          </button>
        </>
      }
    />
  );
}

/** The real LLM config panel (Base URL / Model / Vision model / API key). */
export function LlmConfigPanel({ setup }: { setup: ModelSetup }) {
  const {
    baseUrl,
    setBaseUrl,
    model,
    setModel,
    visionModel,
    setVisionModel,
    apiKey,
    setApiKey,
    keySet,
    saveLlm,
    testLlm,
    testing,
    testResult,
    clearLlmKey,
  } = setup;

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-border bg-background p-4">
      <Field
        label="Base URL"
        value={baseUrl}
        onChange={setBaseUrl}
        placeholder="https://api.openai.com/v1"
      />
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <Field label="Model" value={model} onChange={setModel} placeholder="gpt-4o-mini" />
        <Field
          label="Vision model (optional)"
          value={visionModel}
          onChange={setVisionModel}
          placeholder="same as text model if empty"
        />
      </div>
      <Field
        label={keySet ? "API key (stored in Keychain — type to replace)" : "API key (stored in Keychain)"}
        type="password"
        value={apiKey}
        onChange={setApiKey}
        placeholder={keySet ? "••••••••" : "sk-…"}
      />
      <div className="flex flex-wrap items-center gap-2">
        <button type="button" onClick={saveLlm} className={PRIMARY_BUTTON_SM}>
          Save
        </button>
        <button type="button" onClick={testLlm} disabled={testing} className={GHOST_BUTTON_SM}>
          {testing ? "Testing…" : "Test connection"}
        </button>
        {keySet ? (
          <button type="button" onClick={clearLlmKey} className={GHOST_BUTTON_SM}>
            Clear key
          </button>
        ) : null}
        {testResult ? (
          <span
            className={`font-ui text-[12px] ${testResult.ok ? "text-success" : "text-destructive"}`}
          >
            {testResult.ok
              ? `Connected — ${testResult.model ?? ""} replied`
              : `Couldn't connect — ${testResult.message ?? "unknown error"}`}
          </span>
        ) : null}
      </div>
      <span className="font-ui text-[12px] leading-[1.5] text-muted-foreground">
        Test sends one small request. Saved settings apply on the next start.
      </span>
    </div>
  );
}
