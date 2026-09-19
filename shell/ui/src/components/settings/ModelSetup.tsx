import { useEffect, useRef, useState } from "react";
import { invokeTauriAsync, listenTauri } from "../../tauri";
import {
  CHANGE_BUTTON,
  FOCUS_RING,
  GHOST_BUTTON_SM,
  PRIMARY_BUTTON_SM,
  Row,
  Section,
  Tag,
} from "./primitives";

type ModelKind = "stt" | "tts_model" | "tts_voices";

type CatalogModel = {
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

type SystemVoice = { name: string; lang: string };
type KokoroVoice = { id: string; label: string };
type LlmConfig = { base_url: string; model: string; vision_model: string; api_key_set: boolean };
type TestResult = { ok: boolean; message?: string; model?: string; reply?: string };
type ModelProgress = { id: string; downloaded: number; total: number };

function mb(bytes: number): string {
  return `${Math.max(1, Math.round(bytes / 1_000_000))} MB`;
}

function sizeLabel(bytes: number): string {
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
  return mb(bytes);
}

function hostLabel(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return "the configured endpoint";
  }
}

function familyLabel(family: string): string {
  if (!family) return "Models";
  return family.charAt(0).toUpperCase() + family.slice(1);
}

type PickerOption = { value: string; label: string; disabled?: boolean };
type PickerGroup = { label: string; options: PickerOption[] };

function Picker({
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

function Field({
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

export function ModelsSection({
  offline,
  onTurnOffOffline,
}: {
  offline: boolean;
  onTurnOffOffline: () => void;
}) {
  const [sttModels, setSttModels] = useState<CatalogModel[]>([]);
  const [ttsModels, setTtsModels] = useState<CatalogModel[]>([]);
  const [progress, setProgress] = useState<Record<string, number>>({});

  const [engine, setEngine] = useState("system");
  const [voice, setVoice] = useState("");
  const [rate, setRate] = useState(1);
  const [systemVoices, setSystemVoices] = useState<SystemVoice[]>([]);
  const [kokoroVoices, setKokoroVoices] = useState<KokoroVoice[]>([]);

  const [llm, setLlm] = useState<LlmConfig | null>(null);
  const [llmOpen, setLlmOpen] = useState(false);
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [visionModel, setVisionModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [keySet, setKeySet] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestResult>();

  const pendingSttSelectRef = useRef<string | null>(null);
  const pendingTtsSelectRef = useRef<string | null>(null);
  const rateTimerRef = useRef<number | null>(null);
  const [ttsRequested, setTtsRequested] = useState(false);

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
          if (pendingSttSelectRef.current === p.id) {
            pendingSttSelectRef.current = null;
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
          setTtsRequested(false);
          if (pendingSttSelectRef.current === p?.id) pendingSttSelectRef.current = null;
          if (pendingTtsSelectRef.current === p?.id) pendingTtsSelectRef.current = null;
        }),
      );
    })();
    return () => {
      active = false;
      offs.forEach((off) => off());
      if (rateTimerRef.current !== null) window.clearTimeout(rateTimerRef.current);
    };
  }, []);

  const sttSelected = sttModels.find((m) => m.selected);
  const sttDownloading = sttModels.find((m) => progress[m.id] !== undefined);
  const kokoroModelEntry =
    ttsModels.find((m) => m.kind === "tts_model" && m.selected) ??
    ttsModels.find((m) => m.kind === "tts_model" && m.downloaded);
  const ttsVoicesEntry = ttsModels.find((m) => m.kind === "tts_voices");
  const kokoroReady = Boolean(kokoroModelEntry?.downloaded && ttsVoicesEntry?.downloaded);
  const ttsDownloading = ttsRequested || ttsModels.some((m) => progress[m.id] !== undefined);

  useEffect(() => {
    if (kokoroReady) setTtsRequested(false);
  }, [kokoroReady]);

  const pickSttModel = (id: string) => {
    const entry = sttModels.find((m) => m.id === id);
    if (!entry) return;
    if (entry.downloaded) {
      invokeTauriAsync("stt_select", { id })?.then(refreshStt);
      return;
    }
    pendingSttSelectRef.current = id;
    invokeTauriAsync("stt_download", { id });
  };

  const pickVoice = (value: string) => {
    const separator = value.indexOf(":");
    if (separator < 0) return;
    const kind = value.slice(0, separator);
    const picked = value.slice(separator + 1);
    if (kind !== "kokoro" && kind !== "system") return;
    setEngine(kind);
    setVoice(picked);
    invokeTauriAsync("tts_save_engine", { engine: kind });
    invokeTauriAsync("tts_save_voice", { voice: picked });
  };

  const downloadKokoro = () => {
    setTtsRequested(true);
    invokeTauriAsync("tts_download_kokoro")?.then(
      (raw) => {
        if (typeof raw === "string") pendingTtsSelectRef.current = raw;
      },
      () => setTtsRequested(false),
    );
  };

  const changeRate = (next: number) => {
    setRate(next);
    if (rateTimerRef.current !== null) window.clearTimeout(rateTimerRef.current);
    rateTimerRef.current = window.setTimeout(() => {
      invokeTauriAsync("tts_save_rate", { rate: next });
    }, 250);
  };

  const testVoice = () => {
    if (engine === "kokoro") {
      invokeTauriAsync("tts_synthesize", { text: "", voice: voice })?.catch(() => undefined);
    } else {
      invokeTauriAsync("tts_test_voice", { voice: voice, text: "" })?.catch(() => undefined);
    }
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
    const option: PickerOption = {
      value: entry.id,
      label: entry.downloaded ? entry.name : `${entry.name} — download ${mb(entry.size_bytes)}`,
    };
    if (group) group.options.push(option);
    else sttGroups.push({ label, options: [option] });
  }
  if (!sttSelected && sttGroups.length > 0) {
    sttGroups[0].options.unshift({ value: "", label: "Choose a model", disabled: true });
  }

  const sttOnDevice = sttModels.find((m) => m.downloaded && m.selected);
  const kokoroBytes =
    (kokoroModelEntry?.downloaded ? kokoroModelEntry.size_bytes : 0) +
    (ttsVoicesEntry?.downloaded ? ttsVoicesEntry.size_bytes : 0);
  const onDeviceEntries: { label: string; bytes: number }[] = [];
  if (sttOnDevice) onDeviceEntries.push({ label: sttOnDevice.name, bytes: sttOnDevice.size_bytes });
  if (kokoroBytes > 0) onDeviceEntries.push({ label: "Kokoro", bytes: kokoroBytes });
  const onDeviceTotal = onDeviceEntries.reduce((sum, entry) => sum + entry.bytes, 0);

  const endpointHost = llm?.base_url ? hostLabel(llm.base_url) : null;

  return (
    <Section id="models" title="Models">
      <div className="flex flex-col">
        <Row
          label="Speech to text"
          help="Whisper or Parakeet, running on this Mac."
          side={
            <>
              <Tag>on-device</Tag>
              {sttModels.length > 0 ? (
                <Picker
                  ariaLabel="Speech to text model"
                  value={sttSelected?.id ?? ""}
                  onChange={pickSttModel}
                  groups={sttGroups}
                />
              ) : (
                <span className="font-ui text-[12px] text-muted-foreground">
                  Available in the desktop app.
                </span>
              )}
              {sttDownloading ? (
                <span className="font-mono text-[11px] text-muted-foreground">
                  {Math.round((progress[sttDownloading.id] ?? 0) * 100)}%
                </span>
              ) : null}
            </>
          }
        />
        <Row
          label="Text to speech"
          help={engine === "kokoro" ? "Kokoro, running on this Mac." : "macOS system voice — always available."}
          side={
            <>
              <Tag>{engine === "kokoro" ? "on-device" : "system"}</Tag>
              {voiceOptions.length > 0 ? (
                <Picker
                  ariaLabel="Text to speech voice"
                  value={voiceValue}
                  onChange={pickVoice}
                  groups={voiceGroups}
                />
              ) : (
                <span className="font-ui text-[12px] text-muted-foreground">
                  Available in the desktop app.
                </span>
              )}
              {!kokoroReady && ttsModels.length > 0 ? (
                <button type="button" onClick={downloadKokoro} disabled={ttsDownloading} className={CHANGE_BUTTON}>
                  {ttsDownloading ? "Downloading…" : "Get Kokoro"}
                </button>
              ) : null}
              {voice ? (
                <button type="button" onClick={testVoice} className={GHOST_BUTTON_SM}>
                  Test
                </button>
              ) : null}
            </>
          }
        />
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
                value={rate}
                onChange={(event) => changeRate(Number(event.target.value))}
                aria-label="Speaking rate"
                className={`h-1 w-[132px] accent-primary ${FOCUS_RING}`}
              />
              <span className="min-w-[34px] text-right font-mono text-[11px] text-muted-foreground">
                {rate.toFixed(1)}×
              </span>
            </>
          }
        />
        <Row
          label="Language model"
          help={
            endpointHost
              ? `${endpointHost} in the cloud. Each question sends the capture and your question.`
              : "No endpoint yet — add any OpenAI-compatible service."
          }
          side={
            <>
              <Tag tone="warn">cloud</Tag>
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
        <Row
          label="On-device models"
          help={
            onDeviceEntries.length > 0
              ? `${onDeviceEntries.map((entry) => `${entry.label} ${sizeLabel(entry.bytes)}`).join(" · ")}. Switching models downloads the new one once.`
              : "Nothing downloaded yet — files arrive only with your consent."
          }
          side={<Tag>{onDeviceTotal > 0 ? sizeLabel(onDeviceTotal) : "0 MB"}</Tag>}
        />
      </div>

      {llmOpen ? (
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
      ) : null}

      {llm?.base_url ? (
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
              ? `Offline mode is on, so ${endpointHost} is unavailable. Captures and memory still work.`
              : `${endpointHost} receives the capture and your question. Nothing else leaves this Mac.`}
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
