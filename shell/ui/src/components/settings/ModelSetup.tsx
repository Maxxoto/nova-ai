import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { invokeTauriAsync, listenTauri } from "../../tauri";

function SectionCard({
  eyebrow,
  title,
  description,
  children,
}: {
  eyebrow: string;
  title: string;
  description?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="flex flex-col gap-4 rounded-xl border border-border bg-card p-5">
      <div className="flex flex-col gap-1">
        <span className="font-mono text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">
          {eyebrow}
        </span>
        <h2 className="font-ui text-[17px] font-semibold leading-[1.3] tracking-[-0.01em] text-foreground">
          {title}
        </h2>
        {description ? (
          <p className="max-w-[70ch] font-ui text-[14px] font-normal leading-[1.45] text-muted-foreground">
            {description}
          </p>
        ) : null}
      </div>
      {children}
    </section>
  );
}

function ActionButton({
  onClick,
  disabled,
  tone = "default",
  children,
}: {
  onClick: () => void;
  disabled?: boolean;
  tone?: "default" | "primary" | "danger";
  children: ReactNode;
}) {
  const tones = {
    default: "border-border bg-card text-foreground hover:bg-muted",
    primary: "border-primary bg-primary-soft text-foreground",
    danger: "border-destructive/60 bg-card text-foreground hover:bg-muted",
  } as const;
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={`rounded-pill border px-3 py-1 font-ui text-[12px] font-semibold transition-colors duration-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:opacity-40 ${tones[tone]}`}
    >
      {children}
    </button>
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
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        spellCheck={false}
        className="rounded-lg border border-border bg-background px-3 py-2 font-mono text-[13px] text-foreground outline-none placeholder:text-muted-foreground focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-ring"
      />
    </label>
  );
}

function mb(bytes: number): string {
  return `${Math.max(1, Math.round(bytes / 1_000_000))} MB`;
}

type SttModel = {
  id: string;
  name: string;
  file: string;
  size_bytes: number;
  zh: string;
  note: string;
  downloaded: boolean;
  selected: boolean;
};

type ModelProgress = { id: string; downloaded: number; total: number };

export function SttModelSection() {
  const [models, setModels] = useState<SttModel[]>([]);
  const [progress, setProgress] = useState<Record<string, number>>({});
  const [error, setError] = useState<string>();

  const refresh = () => {
    invokeTauriAsync("stt_catalog")?.then(
      (raw) => setModels((raw as SttModel[]) ?? []),
      () => setModels([]),
    );
  };

  useEffect(() => {
    refresh();
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
      track(await listenTauri<{ id: string }>("model:done", refresh));
      track(
        await listenTauri<{ id: string; message: string }>("model:error", (p) => {
          setError(p?.message ?? "download failed");
          setProgress((prev) => {
            const next = { ...prev };
            delete next[p?.id ?? ""];
            return next;
          });
        }),
      );
    })();
    return () => {
      active = false;
      offs.forEach((off) => off());
    };
  }, []);

  return (
    <SectionCard
      eyebrow="Voice input · STT"
      title="Speech-to-text runs fully on this Mac."
      description="One download per model, only with your consent. Files stay in Ruòxī's data folder — you can also import them yourself."
    >
      <div className="flex flex-col gap-3">
        {models.map((model) => {
          const pct = progress[model.id];
          return (
            <div
              key={model.id}
              className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border bg-background px-4 py-3"
            >
              <div className="flex min-w-0 flex-col gap-0.5">
                <span className="font-ui text-[14px] font-semibold text-foreground">
                  {model.name}
                  {model.selected ? (
                    <span className="ml-2 rounded-pill border border-primary bg-primary-soft px-2 py-0.5 font-ui text-[10px] font-semibold uppercase tracking-wide text-foreground">
                      selected
                    </span>
                  ) : null}
                </span>
                <span className="font-mono text-[11px] text-muted-foreground">
                  {mb(model.size_bytes)} · zh: {model.zh} · {model.note}
                </span>
              </div>
              <div className="flex items-center gap-2">
                {pct !== undefined ? (
                  <span className="font-mono text-[11px] text-muted-foreground">
                    {Math.round(pct * 100)}%
                  </span>
                ) : null}
                {model.downloaded ? (
                  <>
                    {!model.selected ? (
                      <ActionButton
                        tone="primary"
                        onClick={() => invokeTauriAsync("stt_select", { id: model.id })?.then(refresh)}
                      >
                        Select
                      </ActionButton>
                    ) : null}
                    <ActionButton
                      tone="danger"
                      onClick={() => invokeTauriAsync("stt_delete", { id: model.id })?.then(refresh)}
                    >
                      Delete
                    </ActionButton>
                  </>
                ) : (
                  <ActionButton
                    disabled={pct !== undefined}
                    onClick={() => invokeTauriAsync("stt_download", { id: model.id })}
                  >
                    {pct !== undefined ? "Downloading…" : `Download · ${mb(model.size_bytes)}`}
                  </ActionButton>
                )}
              </div>
            </div>
          );
        })}
        {models.length === 0 ? (
          <span className="font-ui text-[13px] text-muted-foreground">
            Available in the desktop app.
          </span>
        ) : null}
        {error ? (
          <span className="font-ui text-[12px] text-destructive">{error}</span>
        ) : null}
      </div>
    </SectionCard>
  );
}

type SystemVoice = { name: string; lang: string };

export function TtsVoiceSection() {
  const [engine, setEngine] = useState("system");
  const [systemVoices, setSystemVoices] = useState<SystemVoice[]>([]);
  const [kokoroVoices, setKokoroVoices] = useState<string[]>([]);
  const [kokoroNote, setKokoroNote] = useState<string>();
  const [kokoroModels, setKokoroModels] = useState<SttModel[]>([]);
  const [progress, setProgress] = useState<Record<string, number>>({});
  const [filter, setFilter] = useState("");
  const [selected, setSelected] = useState("");

  const refresh = () => {
    invokeTauriAsync("get_settings")?.then((raw) => {
      const tts = (raw as { tts?: { engine?: string; voice?: string } } | null)?.tts;
      setEngine(tts?.engine ?? "system");
      setSelected(tts?.voice ?? "");
    });
    invokeTauriAsync("tts_model_catalog")?.then(
      (raw) => setKokoroModels((raw as SttModel[]) ?? []),
      () => setKokoroModels([]),
    );
  };

  const refreshKokoroVoices = () => {
    invokeTauriAsync("tts_kokoro_voices")?.then(
      (raw) => {
        setKokoroVoices((raw as string[]) ?? []);
        setKokoroNote(undefined);
      },
      (err) => {
        setKokoroVoices([]);
        setKokoroNote(String(err ?? "Kokoro not downloaded yet"));
      },
    );
  };

  useEffect(() => {
    refresh();
    refreshKokoroVoices();
    invokeTauriAsync("tts_list_voices")?.then(
      (raw) => setSystemVoices((raw as SystemVoice[]) ?? []),
      () => setSystemVoices([]),
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
        await listenTauri<{ id: string }>("model:done", () => {
          setProgress({});
          refresh();
          refreshKokoroVoices();
        }),
      );
    })();
    return () => {
      active = false;
      offs.forEach((off) => off());
    };
  }, []);

  const chosenModel =
    kokoroModels.find((m) => m.selected) ?? kokoroModels.find((m) => m.id.startsWith("kokoro-onnx"));
  const voicesPack = kokoroModels.find((m) => m.id === "kokoro-voices");
  const kokoroReady = Boolean(chosenModel?.downloaded && voicesPack?.downloaded);

  const shownSystem = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    if (!needle) return systemVoices;
    return systemVoices.filter(
      (v) => v.name.toLowerCase().includes(needle) || v.lang.toLowerCase().includes(needle),
    );
  }, [systemVoices, filter]);
  const shownKokoro = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    if (!needle) return kokoroVoices;
    return kokoroVoices.filter((v) => v.toLowerCase().includes(needle));
  }, [kokoroVoices, filter]);

  const pickEngine = (next: string) => {
    setEngine(next);
    setSelected("");
    invokeTauriAsync("tts_save_engine", { engine: next });
  };

  const testVoice = () => {
    if (engine === "kokoro") {
      invokeTauriAsync("tts_synthesize", { text: "", voice: selected })?.catch(() => undefined);
    } else {
      invokeTauriAsync("tts_test_voice", { voice: selected, text: "" })?.catch(() => undefined);
    }
  };

  return (
    <SectionCard
      eyebrow="Voice output · TTS"
      title="Read-aloud voice."
      description="Fully local. System voices work out of the box; Kokoro (82M, neural) downloads once with consent and runs on this Mac — per RFC-0005."
    >
      <div className="flex flex-col gap-4">
        <div className="flex items-center gap-2">
          {["system", "kokoro"].map((option) => (
            <button
              key={option}
              type="button"
              onClick={() => pickEngine(option)}
              className={`rounded-pill border px-3 py-1 font-ui text-[12px] font-semibold transition-colors duration-200 ${
                engine === option
                  ? "border-primary bg-primary-soft text-foreground"
                  : "border-border bg-card text-muted-foreground hover:bg-muted"
              }`}
            >
              {option === "system" ? "System voices" : "Kokoro (neural)"}
            </button>
          ))}
        </div>

        {engine === "kokoro" ? (
          <div className="flex flex-col gap-2">
            {kokoroModels.map((model) => {
              const pct = progress[model.id];
              return (
                <div
                  key={model.id}
                  className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border bg-background px-4 py-3"
                >
                  <div className="flex min-w-0 flex-col gap-0.5">
                    <span className="font-ui text-[14px] font-semibold text-foreground">
                      {model.name}
                      {model.selected ? (
                        <span className="ml-2 rounded-pill border border-primary bg-primary-soft px-2 py-0.5 font-mono text-[10px] uppercase tracking-wide text-foreground">
                          selected
                        </span>
                      ) : null}
                    </span>
                    <span className="font-mono text-[11px] text-muted-foreground">
                      {mb(model.size_bytes)} · {model.zh} · {model.note}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    {pct !== undefined ? (
                      <span className="font-mono text-[11px] text-muted-foreground">
                        {Math.round(pct * 100)}%
                      </span>
                    ) : null}
                    {model.downloaded ? (
                      model.id !== "kokoro-voices" && !model.selected ? (
                        <ActionButton
                          tone="primary"
                          onClick={() =>
                            invokeTauriAsync("tts_model_select", { id: model.id })?.then(refresh)
                          }
                        >
                          Select
                        </ActionButton>
                      ) : null
                    ) : (
                      <ActionButton
                        disabled={pct !== undefined}
                        onClick={() => invokeTauriAsync("stt_download", { id: model.id })}
                      >
                        {pct !== undefined ? "Downloading…" : `Download · ${mb(model.size_bytes)}`}
                      </ActionButton>
                    )}
                  </div>
                </div>
              );
            })}
            {!kokoroReady ? (
              <span className="font-mono text-[11px] text-muted-foreground">
                Download a model and the voicepacks, then pick a voice.
              </span>
            ) : kokoroNote ? (
              <span className="font-mono text-[11px] text-destructive">{kokoroNote}</span>
            ) : null}
          </div>
        ) : null}

        <Field
          label="Filter voices"
          value={filter}
          onChange={setFilter}
          placeholder={engine === "kokoro" ? "e.g. zh or af_heart" : "name or locale, e.g. zh or Ava"}
        />
        <div className="flex max-h-56 flex-col gap-1 overflow-y-auto">
          {engine === "kokoro"
            ? shownKokoro.slice(0, 80).map((voice) => (
                <button
                  key={voice}
                  type="button"
                  disabled={!kokoroReady}
                  onClick={() => {
                    setSelected(voice);
                    invokeTauriAsync("tts_save_voice", { voice });
                  }}
                  className={`flex items-center justify-between rounded-lg border px-3 py-1.5 text-left transition-colors duration-200 ${
                    selected === voice
                      ? "border-primary bg-primary-soft"
                      : "border-border bg-background hover:bg-muted"
                  } disabled:opacity-40`}
                >
                  <span className="font-ui text-[13px] font-semibold text-foreground">{voice}</span>
                </button>
              ))
            : shownSystem.slice(0, 80).map((voice) => (
                <button
                  key={`${voice.name}-${voice.lang}`}
                  type="button"
                  onClick={() => {
                    setSelected(voice.name);
                    invokeTauriAsync("tts_save_voice", { voice: voice.name });
                  }}
                  className={`flex items-center justify-between rounded-lg border px-3 py-1.5 text-left transition-colors duration-200 ${
                    selected === voice.name
                      ? "border-primary bg-primary-soft"
                      : "border-border bg-background hover:bg-muted"
                  }`}
                >
                  <span className="font-ui text-[13px] font-semibold text-foreground">{voice.name}</span>
                  <span className="font-mono text-[11px] text-muted-foreground">{voice.lang}</span>
                </button>
              ))}
        </div>
        <div>
          <ActionButton disabled={!selected} onClick={testVoice}>
            Test voice
          </ActionButton>
        </div>
      </div>
    </SectionCard>
  );
}

type LlmConfig = {
  base_url: string;
  model: string;
  vision_model: string;
  api_key_set: boolean;
};

type TestResult = { ok: boolean; message?: string; model?: string; reply?: string };

export function LlmConfigSection() {
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [visionModel, setVisionModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [keySet, setKeySet] = useState(false);
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<TestResult>();

  useEffect(() => {
    invokeTauriAsync("get_settings")?.then((raw) => {
      const llm = (raw as { llm?: LlmConfig } | null)?.llm;
      if (!llm) return;
      setBaseUrl(llm.base_url ?? "");
      setModel(llm.model ?? "");
      setVisionModel(llm.vision_model ?? "");
      setKeySet(Boolean(llm.api_key_set));
    });
  }, []);

  const save = () => {
    invokeTauriAsync("llm_save_config", {
      baseUrl,
      model,
      visionModel,
      apiKey: apiKey.trim() ? apiKey : null,
    })?.then(
      (raw) => {
        const llm = raw as LlmConfig | null;
        setKeySet(Boolean(llm?.api_key_set));
        setApiKey("");
      },
      () => setResult({ ok: false, message: "save failed" }),
    );
  };

  const test = () => {
    setTesting(true);
    setResult(undefined);
    invokeTauriAsync("llm_test", {
      baseUrl: baseUrl.trim() || null,
      model: model.trim() || null,
      apiKey: apiKey.trim() || null,
    })?.then(
      (raw) => setResult((raw as TestResult) ?? { ok: false, message: "no response" }),
      () => setResult({ ok: false, message: "the brain isn't responding — try again in a moment" }),
    );
    setTesting(false);
  };

  return (
    <SectionCard
      eyebrow="Brain · LLM"
      title="Your key, your endpoint — nothing built in."
      description="Works with any OpenAI-compatible service — nothing is built in. Your key is kept in the macOS Keychain, never written to settings files."
    >
      <div className="flex flex-col gap-3">
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
          label={keySet ? "API key (stored in Keychain — type to replace)" : "API key"}
          type="password"
          value={apiKey}
          onChange={setApiKey}
          placeholder={keySet ? "••••••••" : "sk-…"}
        />
        <div className="flex flex-wrap items-center gap-2">
          <ActionButton tone="primary" onClick={save}>
            Save
          </ActionButton>
          <ActionButton disabled={testing} onClick={test}>
            {testing ? "Testing…" : "Test connection"}
          </ActionButton>
          {keySet ? (
            <ActionButton
              tone="danger"
              onClick={() =>
                invokeTauriAsync("llm_clear_api_key")?.then(() => setKeySet(false))
              }
            >
              Clear key
            </ActionButton>
          ) : null}
          {result ? (
            <span
              className={`font-ui text-[12px] ${result.ok ? "text-success" : "text-destructive"}`}
            >
              {result.ok
                ? `Connected — ${result.model ?? ""} replied`
                : `Couldn't connect — ${result.message ?? "unknown error"}`}
            </span>
          ) : null}
        </div>
        <span className="font-ui text-[12px] leading-[1.5] text-muted-foreground">
          Sends one small test request. Saved settings apply when the app restarts.
        </span>
      </div>
    </SectionCard>
  );
}
