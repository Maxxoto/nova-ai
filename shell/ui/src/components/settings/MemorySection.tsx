import { useCallback, useEffect, useState } from "react";

import { invokeTauriAsync } from "../../tauri";
import { CHANGE_BUTTON, FOCUS_RING, Section } from "./primitives";

/** One row from `memory_list` / `memory_archived_list`. */
type MemoryRow = {
  id: string;
  kind: string;
  created: string;
  source_refs: string[];
  snippet: string;
};

function ageLabel(iso: string): string {
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return "";
  const days = Math.max(0, Math.floor((Date.now() - then) / 86_400_000));
  if (days === 0) return "today";
  return `${days} day${days === 1 ? "" : "s"} ago`;
}

function MemoryList({
  rows,
  action,
  onAction,
  busy,
  empty,
}: {
  rows: MemoryRow[];
  action: string;
  onAction: (id: string) => void;
  busy: string | null;
  empty: string;
}) {
  if (rows.length === 0) {
    return (
      <p className="px-1 py-2 font-ui text-[12px] leading-[1.5] text-muted-foreground">{empty}</p>
    );
  }
  return (
    <ul className="flex flex-col">
      {rows.map((row) => (
        <li
          key={row.id}
          className="flex items-start justify-between gap-3 border-b border-border py-2.5 last:border-b-0"
        >
          <div className="flex min-w-0 flex-col gap-1">
            <span className="font-ui text-[13px] leading-[1.45] text-foreground">{row.snippet}</span>
            <span className="truncate font-mono text-[11px] leading-[1.4] text-muted-foreground">
              {row.kind} · {ageLabel(row.created)}
              {row.source_refs.length > 0 ? ` · ${row.source_refs[0]}` : ""}
            </span>
          </div>
          <button
            type="button"
            disabled={busy === row.id}
            onClick={() => onAction(row.id)}
            className={CHANGE_BUTTON}
          >
            {action}
          </button>
        </li>
      ))}
    </ul>
  );
}

/**
 * Settings → Memory (RFC-0011 F-16): everything remembered, newest first,
 * with FTS search. Archiving removes a note from answers without deleting it
 * — the archived list restores it. User-saved answers live here next to the
 * auto-logged asks.
 */
export default function MemorySection() {
  const [rows, setRows] = useState<MemoryRow[] | null>(null);
  const [archived, setArchived] = useState<MemoryRow[]>([]);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState<string | null>(null);

  const refresh = useCallback((q: string) => {
    const list = invokeTauriAsync("memory_list", { query: q.trim().length > 0 ? q.trim() : null });
    if (!list) {
      setRows([]);
      return;
    }
    list.then(
      (raw) => setRows(Array.isArray(raw) ? (raw as MemoryRow[]) : []),
      () => setRows([]),
    );
    const archivedList = invokeTauriAsync("memory_archived_list");
    archivedList?.then(
      (raw) => setArchived(Array.isArray(raw) ? (raw as MemoryRow[]) : []),
      () => setArchived([]),
    );
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => refresh(query), query.trim().length > 0 ? 250 : 0);
    return () => window.clearTimeout(timer);
  }, [query, refresh]);

  const archive = async (id: string) => {
    setBusy(id);
    await invokeTauriAsync("memory_archive", { id });
    setBusy(null);
    refresh(query);
  };

  const restore = async (id: string) => {
    setBusy(id);
    await invokeTauriAsync("memory_restore", { id });
    setBusy(null);
    refresh(query);
  };

  return (
    <Section
      id="memory"
      title="Memory"
      description="What Ruoxi remembers, newest first. Archiving removes a note from answers — nothing is deleted."
    >
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2 rounded-pill border border-border bg-card py-1.5 pl-3 pr-3 shadow-e1">
          <input
            type="text"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Filter what Ruoxi remembers…"
            aria-label="Filter memories"
            spellCheck={false}
            autoComplete="off"
            className={`min-w-0 flex-1 border-0 bg-transparent font-mono text-[13px] text-foreground placeholder:text-muted-foreground ${FOCUS_RING}`}
          />
        </div>
        {rows === null ? (
          <p className="px-1 py-2 font-ui text-[12px] text-muted-foreground">Reading memory…</p>
        ) : (
          <MemoryList
            rows={rows}
            action="Archive"
            onAction={archive}
            busy={busy}
            empty={
              query.trim().length > 0
                ? "Nothing in memory matches that filter."
                : "Nothing saved yet — Save to memory from any answer."
            }
          />
        )}

        {archived.length > 0 ? (
          <>
            <p className="pt-3 font-mono text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">
              Archived
            </p>
            <MemoryList
              rows={archived}
              action="Restore"
              onAction={restore}
              busy={busy}
              empty=""
            />
          </>
        ) : null}
      </div>
    </Section>
  );
}
