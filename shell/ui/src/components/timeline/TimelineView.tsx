import { useEffect, useId, useMemo, useRef, useState } from "react";
import { invokeTauriAsync } from "../../tauri";
import {
  SAMPLE_ROWS,
  formatAge,
  formatBytes,
  formatClock,
  formatDayLabel,
  formatDimensions,
  formatScope,
  groupByDay,
  localDayKey,
  normalizeCaptureRows,
  normalizeCaptureStats,
  sampleStats,
} from "./format";
import type { CaptureRow, CaptureStats } from "./format";

/* ── wire commands (backend contract; every call is guarded by tauri.ts) ── */
const LOAD_LIMIT = 1000;
const THUMB_LIMIT = 40;
const THUMB_MAX_PX = 480;

/** Focus-visible ring per DESIGN.md — 2px primary, offset 2. */
const FOCUS_RING =
  "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring";

const SECONDARY_BUTTON = `inline-flex h-8 items-center justify-center whitespace-nowrap rounded border border-border bg-card px-3 font-ui text-[13px] font-semibold text-foreground transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] hover:bg-muted ${FOCUS_RING}`;
const GHOST_BUTTON_SM = `inline-flex h-8 items-center justify-center rounded px-3 font-ui text-[13px] font-semibold text-muted-foreground transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] hover:bg-muted hover:text-foreground ${FOCUS_RING}`;
const DANGER_BUTTON = `inline-flex h-8 items-center justify-center rounded border border-destructive bg-card px-3 font-ui text-[13px] font-semibold text-destructive transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] hover:bg-destructive/10 disabled:cursor-default disabled:border-border disabled:text-muted-foreground disabled:opacity-70 ${FOCUS_RING}`;
const CARD_BUTTON = `${SECONDARY_BUTTON} hover:border-primary`;

type Mode = "loading" | "ready" | "demo" | "fallback" | "error";

function SearchIcon({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      aria-hidden="true"
      className={className}
    >
      <circle cx="11" cy="11" r="6" />
      <path d="m20 20-3.4-3.4" />
    </svg>
  );
}

function StatLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <span className="font-ui text-[13px] leading-[1.45] text-muted-foreground">{label}</span>
      <span className="font-mono text-[13px] font-semibold tabular-nums text-foreground">{value}</span>
    </div>
  );
}

function StatsRow({
  stats,
  rows,
  sample,
}: {
  stats: CaptureStats | null;
  rows: CaptureRow[];
  sample: boolean;
}) {
  const loaded = rows.length;
  const total = stats?.count ?? loaded;
  const truncated = stats !== null && stats.count > loaded;
  const countScope = (scope: string) => rows.reduce((n, row) => (row.scope === scope ? n + 1 : n), 0);

  return (
    <div className="grid gap-3 sm:grid-cols-2">
      <section className="flex flex-col gap-3 rounded-xl border border-border bg-card p-4">
        <h2 className="font-mono text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">
          Store
        </h2>
        <div className="flex flex-col gap-2">
          <StatLine label="Captures" value={stats ? String(total) : "…"} />
          <StatLine label="On disk" value={stats ? formatBytes(stats.bytes) : "…"} />
          <StatLine label="Oldest capture" value={stats ? formatAge(stats.oldest_ms) : "…"} />
        </div>
        <p className="font-mono text-[11px] leading-[1.45] text-muted-foreground">
          {sample
            ? "Sample numbers from the demo view — not your store."
            : "Read from your local capture store. Nothing leaves this Mac."}
        </p>
      </section>

      <section className="flex flex-col gap-3 rounded-xl border border-border bg-card p-4">
        <h2 className="font-mono text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">
          Captures by scope
        </h2>
        <div className="flex flex-col gap-2">
          <StatLine label="Region" value={String(countScope("region"))} />
          <StatLine label="Window" value={String(countScope("window"))} />
          <StatLine label="Fullscreen" value={String(countScope("fullscreen"))} />
        </div>
        <p className="font-mono text-[11px] leading-[1.45] text-muted-foreground">
          {sample
            ? "Sample counts derived from the demo rows."
            : truncated
              ? `Counts cover the newest ${loaded} of ${total} captures.`
              : `Counts across every one of your ${loaded} captures.`}
        </p>
      </section>
    </div>
  );
}

function CaptureCard({ row, thumb, sample }: { row: CaptureRow; thumb?: string; sample: boolean }) {
  const title = row.window_title?.trim() || row.app?.trim() || "Untitled capture";
  const scopeLine = `${formatScope(row.scope)}${row.app ? ` · ${row.app}` : ""}`;
  return (
    <article className="flex flex-col overflow-hidden rounded-lg border border-border bg-card transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] hover:border-primary">
      <div className="relative grid h-[112px] place-items-center border-b border-border bg-muted">
        {thumb ? (
          <img
            src={thumb}
            alt={`Thumbnail of capture ${row.capture_id}`}
            className="h-full w-full object-cover"
          />
        ) : (
          <span className="px-3 text-center font-mono text-[11px] leading-[1.5] text-muted-foreground">
            {sample ? "sample · no pixels" : "no preview"}
          </span>
        )}
        {sample ? (
          <span className="absolute left-2 top-2 rounded-pill border border-border bg-card px-2 py-0.5 font-mono text-[10px] font-medium uppercase tracking-[0.06em] text-muted-foreground">
            sample
          </span>
        ) : null}
      </div>
      <div className="flex flex-col gap-2 p-3">
        <span className="font-ui text-[13px] font-semibold leading-[1.4] text-foreground">{title}</span>
        <span className="flex items-center justify-between gap-2 font-mono text-[11px] leading-[1.4] text-muted-foreground">
          <span className="truncate">{row.capture_id}</span>
          <span className="flex-none tabular-nums">{formatClock(row.ts_ms)}</span>
        </span>
        <span className="flex items-center justify-between gap-2 font-mono text-[11px] leading-[1.4] text-muted-foreground">
          <span className="truncate">{scopeLine}</span>
          <span className="flex-none tabular-nums">{formatDimensions(row)}</span>
        </span>
      </div>
    </article>
  );
}

function EmptyCard({ title, body }: { title: string; body: string }) {
  return (
    <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-border bg-card px-5 py-10 text-center">
      <p className="font-ui text-[14px] font-semibold leading-[1.4] text-foreground">{title}</p>
      <p className="max-w-[56ch] font-ui text-[13px] leading-[1.45] text-muted-foreground">{body}</p>
    </div>
  );
}

function ConfirmDeleteDialog({
  count,
  onCancel,
  onConfirm,
}: {
  count: number;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const [typed, setTyped] = useState("");
  const inputId = useId();
  const titleId = useId();
  const canDelete = typed === "DELETE";

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="fixed inset-0 z-50 grid place-items-center p-6">
      <div aria-hidden="true" className="absolute inset-0 bg-foreground/40" onClick={onCancel} />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="relative w-[440px] max-w-full rounded-lg border border-border bg-card p-6 shadow-e3"
      >
        <h3 id={titleId} className="font-ui text-[17px] font-semibold leading-[1.3] text-foreground">
          {count === 0 ? "Delete all captures?" : `Delete all ${count} capture${count === 1 ? "" : "s"}?`}
        </h3>
        <p className="mt-2 font-ui text-[13px] leading-[1.45] text-muted-foreground">
          This removes every stored capture from this Mac. It cannot be undone, and nothing is recoverable from a
          server. Type <span className="font-mono font-medium text-foreground">DELETE</span> to confirm.
        </p>
        <div className="mt-4 flex flex-col gap-1.5">
          <label htmlFor={inputId} className="font-ui text-[13px] text-muted-foreground">
            Confirmation
          </label>
          <input
            id={inputId}
            value={typed}
            autoFocus
            spellCheck={false}
            autoComplete="off"
            placeholder="DELETE"
            onChange={(event) => setTyped(event.target.value)}
            className={`rounded border border-border bg-card px-3 py-2 font-mono text-[13px] text-foreground transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] focus:border-primary ${FOCUS_RING}`}
          />
        </div>
        <div className="mt-5 flex justify-end gap-2">
          <button type="button" onClick={onCancel} className={GHOST_BUTTON_SM}>
            Cancel
          </button>
          <button type="button" disabled={!canDelete} onClick={onConfirm} className={DANGER_BUTTON}>
            Delete all
          </button>
        </div>
      </div>
    </div>
  );
}

export default function TimelineView({
  demo = false,
  reducedMotion = false,
}: {
  demo?: boolean;
  reducedMotion?: boolean;
}) {
  const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__?.invoke;
  const [mode, setMode] = useState<Mode>(demo ? "demo" : isTauri ? "loading" : "fallback");
  const [rows, setRows] = useState<CaptureRow[]>(demo ? SAMPLE_ROWS : []);
  const [stats, setStats] = useState<CaptureStats | null>(demo ? sampleStats(SAMPLE_ROWS) : null);
  const [reloadKey, setReloadKey] = useState(0);
  const [query, setQuery] = useState("");
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [thumbs, setThumbs] = useState<Record<string, string>>({});
  const requestedRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    if (demo) {
      setMode("demo");
      setRows(SAMPLE_ROWS);
      setStats(sampleStats(SAMPLE_ROWS));
      return;
    }
    if (!isTauri) {
      setMode("fallback");
      return;
    }
    let active = true;
    setMode("loading");
    const list = invokeTauriAsync("timeline_list", { filter: { limit: LOAD_LIMIT } });
    if (!list) {
      setMode("fallback");
      return;
    }
    list.then(
      (raw) => {
        if (!active) return;
        setRows(normalizeCaptureRows(raw));
        setMode("ready");
      },
      () => {
        if (active) setMode("error");
      },
    );
    invokeTauriAsync("capture_store_stats")?.then(
      (raw) => {
        if (active) setStats(normalizeCaptureStats(raw));
      },
      () => undefined,
    );
    return () => {
      active = false;
    };
  }, [demo, isTauri, reloadKey]);

  const trimmed = query.trim().toLowerCase();
  const filtered = useMemo(() => {
    if (!trimmed) return rows;
    return rows.filter((row) => {
      const haystack = [
        row.capture_id,
        row.app ?? "",
        row.window_title ?? "",
        row.scope,
        localDayKey(row.ts_ms),
        formatDayLabel(row.ts_ms),
      ]
        .join(" ")
        .toLowerCase();
      return haystack.includes(trimmed);
    });
  }, [rows, trimmed]);

  const groups = useMemo(() => groupByDay(filtered), [filtered]);

  /* Thumbnails: only the newest THUMB_LIMIT matching cards ever fetch pixels;
     the rest keep the placeholder, and the line below says so plainly. */
  const thumbTargets = useMemo(() => filtered.slice(0, THUMB_LIMIT).map((row) => row.capture_id), [filtered]);
  const thumbSignature = thumbTargets.join(",");
  useEffect(() => {
    if (demo || !isTauri) return;
    for (const id of thumbTargets) {
      if (requestedRef.current.has(id)) continue;
      requestedRef.current.add(id);
      const pending = invokeTauriAsync("capture_thumbnail", { captureId: id, maxPx: THUMB_MAX_PX });
      if (!pending) continue;
      pending.then(
        (raw) => {
          if (typeof raw === "string" && raw.startsWith("data:image/")) {
            setThumbs((prev) => (prev[id] ? prev : { ...prev, [id]: raw }));
          }
        },
        () => undefined,
      );
    }
  }, [thumbTargets, thumbSignature, demo, isTauri]);

  const handleConfirmDelete = () => {
    setConfirmOpen(false);
    if (demo) {
      setNotice("Sample view — nothing on disk was touched.");
      return;
    }
    const pending = invokeTauriAsync("capture_delete_all");
    if (!pending) {
      setNotice("Not running in the app — there is no local store to clear.");
      return;
    }
    pending.then(
      () => {
        setNotice("Local store cleared. Nothing left the machine at any point.");
        setReloadKey((key) => key + 1);
      },
      () => setNotice("Could not clear the local store. Nothing was removed."),
    );
  };

  const rootClass = `mx-auto flex w-full max-w-[980px] flex-col gap-6 p-8${
    reducedMotion ? " reduced-motion rm-halve" : ""
  }`;

  const header = (
    <header className="flex flex-col gap-1.5">
      <h1 className="font-ui text-[22px] font-bold leading-[1.2] tracking-[-0.02em] text-foreground">
        Your captures are the source.
      </h1>
      <p className="max-w-[66ch] font-ui text-[14px] leading-[1.5] text-muted-foreground">
        Ask a question and Ruòxī answers from what you saved — with the exact region or window it came from attached
        to the answer. Everything on this screen lives on the machine; the timeline is the proof, not a sync log.
      </p>
    </header>
  );

  if (mode === "fallback" || mode === "error") {
    return (
      <div className={rootClass}>
        {header}
        <EmptyCard
          title={mode === "error" ? "Couldn’t read the local store." : "This screen reads your local capture store."}
          body={
            mode === "error"
              ? "The timeline command didn’t answer, so nothing is shown. Nothing was changed — reopen this window, or add ?demo=1 to review the surface with labelled sample rows."
              : "Open it from the Ruòxī desktop app to see your captures. In the browser there is no store to read — no sample rows appear unless you open this view with ?view=timeline&demo=1."
          }
        />
      </div>
    );
  }

  return (
    <div className={rootClass}>
      {header}

      {demo ? (
        <p className="rounded border border-dashed border-border px-3 py-2 font-mono text-[11px] leading-[1.45] text-muted-foreground">
          Sample view (?demo=1) — representative rows for review. Nothing here is read from or written to a real
          store.
        </p>
      ) : null}

      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2 rounded-pill border border-border bg-card py-1.5 pl-3 pr-1.5 shadow-e1">
          <SearchIcon className="h-[15px] w-[15px] flex-none text-muted-foreground" />
          <input
            type="text"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Filter by app, window, capture id, or day…"
            aria-label="Filter captures"
            spellCheck={false}
            autoComplete="off"
            className={`min-w-0 flex-1 border-0 bg-transparent font-mono text-[13px] text-foreground placeholder:text-muted-foreground ${FOCUS_RING}`}
          />
          {query ? (
            <button type="button" onClick={() => setQuery("")} aria-label="Clear filter" className={GHOST_BUTTON_SM}>
              clear
            </button>
          ) : null}
        </div>
        <p className="font-mono text-[11px] leading-[1.45] text-muted-foreground">
          Filters your local timeline as you type. Asking Ruòxī about your memory arrives with memory search.
        </p>
      </div>

      <StatsRow stats={stats} rows={rows} sample={demo} />

      <div className="flex flex-col gap-6" role="region" aria-label="Capture timeline">
        {mode === "loading" ? (
          <p className="font-mono text-[11px] leading-[1.45] text-muted-foreground">Reading the local store…</p>
        ) : rows.length === 0 ? (
          <EmptyCard
            title="Nothing captured yet."
            body="Save something and it lands here — the timeline is the proof, not a sync log. Until then, there is nothing to show."
          />
        ) : filtered.length === 0 ? (
          <EmptyCard title="Nothing matches that filter." body="Clear the filter to see every capture again." />
        ) : (
          <>
            {filtered.length > THUMB_LIMIT ? (
              <p className="font-mono text-[11px] leading-[1.45] text-muted-foreground">
                Showing {THUMB_LIMIT} of {filtered.length} matching captures with thumbnails — newest first.
              </p>
            ) : null}
            {groups.map((group) => (
              <section key={group.key} className="flex flex-col gap-3">
                <h2 className="font-mono text-[11px] font-medium tracking-[0.08em] text-muted-foreground">
                  {group.label}
                </h2>
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3">
                  {group.rows.map((row) => (
                    <CaptureCard key={row.capture_id} row={row} thumb={thumbs[row.capture_id]} sample={demo} />
                  ))}
                </div>
              </section>
            ))}
          </>
        )}
      </div>

      {notice ? (
        <p role="status" className="rounded border border-border bg-muted px-3 py-2 font-ui text-[13px] leading-[1.45] text-muted-foreground">
          {notice}
        </p>
      ) : null}

      <section className="flex flex-col gap-3 rounded-xl border border-border bg-card p-5 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex max-w-[60ch] flex-col gap-1">
          <h2 className="font-ui text-[17px] font-semibold leading-[1.3] tracking-[-0.01em] text-foreground">
            Delete everything
          </h2>
          <p className="font-ui text-[13px] leading-[1.45] text-muted-foreground">
            Removes every stored capture from this Mac. There is no cloud copy to fall back on — which is the point,
            and why this needs typing.
          </p>
          {demo ? (
            <p className="font-mono text-[11px] leading-[1.45] text-muted-foreground">
              Sample view — confirming only previews the dialog.
            </p>
          ) : null}
        </div>
        <button type="button" onClick={() => setConfirmOpen(true)} className={`${CARD_BUTTON} h-9 px-3.5`}>
          Delete all captures…
        </button>
      </section>

      {confirmOpen ? (
        <ConfirmDeleteDialog
          count={stats?.count ?? rows.length}
          onCancel={() => setConfirmOpen(false)}
          onConfirm={handleConfirmDelete}
        />
      ) : null}
    </div>
  );
}
