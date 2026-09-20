/**
 * Pure helpers for the memory/timeline surface (DESIGN.md F-08, RFC-0009 §4.7).
 * No rendering here — see TimelineView.tsx. Everything is token-free data:
 * wire normalization, day grouping, and the explicitly-labelled sample rows
 * behind `?view=timeline&demo=1`.
 */

export type CaptureRow = {
  capture_id: string;
  ts_ms: number;
  scope: string;
  app: string | null;
  window_title: string | null;
  w_px: number;
  h_px: number;
  scale: number;
  bytes: number;
};

export type CaptureStats = { count: number; bytes: number; oldest_ms: number | null };

export type DayGroup = { key: string; label: string; rows: CaptureRow[] };

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"] as const;
const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;

function pad2(n: number): string {
  return n < 10 ? `0${n}` : String(n);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function readNumber(record: Record<string, unknown>, key: string): number | null {
  const value = record[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

/**
 * Normalizes one `timeline_list` row. Accepts `ts_ms` (the stated contract) and
 * falls back to the store's `ts` field so the view survives either shape; rows
 * without a stable id or timestamp are dropped rather than invented.
 */
export function normalizeCaptureRow(raw: unknown): CaptureRow | null {
  if (!isRecord(raw)) return null;
  const captureId = raw.capture_id;
  if (typeof captureId !== "string" || captureId.length === 0) return null;
  const ts = readNumber(raw, "ts_ms") ?? readNumber(raw, "ts");
  if (ts === null) return null;
  return {
    capture_id: captureId,
    ts_ms: ts,
    scope: typeof raw.scope === "string" && raw.scope.length > 0 ? raw.scope : "unknown",
    app: typeof raw.app === "string" ? raw.app : null,
    window_title: typeof raw.window_title === "string" ? raw.window_title : null,
    w_px: readNumber(raw, "w_px") ?? 0,
    h_px: readNumber(raw, "h_px") ?? 0,
    scale: readNumber(raw, "scale") ?? 0,
    bytes: readNumber(raw, "bytes") ?? 0,
  };
}

export function normalizeCaptureRows(raw: unknown): CaptureRow[] {
  if (!Array.isArray(raw)) return [];
  const rows: CaptureRow[] = [];
  for (const item of raw) {
    const row = normalizeCaptureRow(item);
    if (row) rows.push(row);
  }
  return rows;
}

export function normalizeCaptureStats(raw: unknown): CaptureStats {
  if (!isRecord(raw)) return { count: 0, bytes: 0, oldest_ms: null };
  return {
    count: readNumber(raw, "count") ?? 0,
    bytes: readNumber(raw, "bytes") ?? 0,
    oldest_ms: readNumber(raw, "oldest_ms"),
  };
}

/** Local calendar date, so day groups follow the user's clock, not UTC. */
export function localDayKey(tsMs: number): string {
  const d = new Date(tsMs);
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`;
}

export function formatDayLabel(tsMs: number): string {
  const d = new Date(tsMs);
  const base = `${WEEKDAYS[d.getDay()]} · ${MONTHS[d.getMonth()]} ${d.getDate()}`;
  return d.getFullYear() === new Date().getFullYear() ? base : `${base}, ${d.getFullYear()}`;
}

export function formatClock(tsMs: number): string {
  const d = new Date(tsMs);
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"] as const;
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** exponent;
  return `${exponent === 0 ? String(value) : value.toFixed(1)} ${units[exponent]}`;
}

export function formatAge(oldestMs: number | null): string {
  if (oldestMs === null) return "unknown";
  const days = Math.max(0, Math.floor((Date.now() - oldestMs) / 86_400_000));
  if (days === 0) return "today";
  return `${days} day${days === 1 ? "" : "s"} ago`;
}

export function formatScope(scope: string): string {
  if (scope.length === 0 || scope === "unknown") return "unknown";
  return scope.charAt(0).toUpperCase() + scope.slice(1);
}

export function formatDimensions(row: CaptureRow): string {
  if (row.w_px <= 0 || row.h_px <= 0) return "—";
  return `${row.w_px} × ${row.h_px}`;
}

/** Groups a newest-first row list into newest-first day buckets. */
export function groupByDay(rows: CaptureRow[]): DayGroup[] {
  const groups: DayGroup[] = [];
  for (const row of rows) {
    const key = localDayKey(row.ts_ms);
    const last = groups[groups.length - 1];
    if (last && last.key === key) last.rows.push(row);
    else groups.push({ key, label: formatDayLabel(row.ts_ms), rows: [row] });
  }
  return groups;
}

const SAMPLE_ANCHOR = Date.now();

function sampleTime(daysAgo: number, hour: number, minute: number): number {
  const d = new Date(SAMPLE_ANCHOR - daysAgo * 86_400_000);
  d.setHours(hour, minute, 0, 0);
  return d.getTime();
}

/**
 * Representative rows for the dev-only QA affordance (`?view=timeline&demo=1`).
 * Never merged with real data: the view only reaches for these when the flag is
 * present, and every card is labelled `sample`.
 */
export const SAMPLE_ROWS: CaptureRow[] = [
  {
    capture_id: "cap_demo_01J8ZF5Q2W9R3T4A",
    ts_ms: sampleTime(0, 14, 2),
    scope: "region",
    app: "Preview",
    window_title: "Oxidative phosphorylation paragraph",
    w_px: 412,
    h_px: 96,
    scale: 2,
    bytes: 262_144,
  },
  {
    capture_id: "cap_demo_01J8ZF5Q2W9R3T5B",
    ts_ms: sampleTime(0, 13, 48),
    scope: "window",
    app: "Safari",
    window_title: "Cell Biology · Chapter 4",
    w_px: 1440,
    h_px: 900,
    scale: 2,
    bytes: 1_887_436,
  },
  {
    capture_id: "cap_demo_01J8ZF5Q2W9R3T6C",
    ts_ms: sampleTime(0, 9, 14),
    scope: "fullscreen",
    app: "Terminal",
    window_title: "cargo build — errors",
    w_px: 2880,
    h_px: 1800,
    scale: 2,
    bytes: 2_621_440,
  },
  {
    capture_id: "cap_demo_01J8ZF5Q2W9R3T7D",
    ts_ms: sampleTime(1, 17, 26),
    scope: "region",
    app: "Preview",
    window_title: "Rust borrow checker diagram",
    w_px: 920,
    h_px: 540,
    scale: 1,
    bytes: 486_400,
  },
  {
    capture_id: "cap_demo_01J8ZF5Q2W9R3T8E",
    ts_ms: sampleTime(1, 16, 3),
    scope: "window",
    app: "Safari",
    window_title: "Rust Book · Lifetime Elision",
    w_px: 1440,
    h_px: 900,
    scale: 2,
    bytes: 1_744_896,
  },
  {
    capture_id: "cap_demo_01J8ZF5Q2W9R3T9F",
    ts_ms: sampleTime(1, 8, 41),
    scope: "fullscreen",
    app: "Finder",
    window_title: "Nova workspace",
    w_px: 2560,
    h_px: 1440,
    scale: 2,
    bytes: 2_097_152,
  },
  {
    capture_id: "cap_demo_01J8ZF5Q2W9R3TAG",
    ts_ms: sampleTime(2, 11, 5),
    scope: "region",
    app: "Notes",
    window_title: "Interview notes — sync design",
    w_px: 680,
    h_px: 220,
    scale: 2,
    bytes: 131_072,
  },
];

export function sampleStats(rows: CaptureRow[]): CaptureStats {
  let bytes = 0;
  let oldest: number | null = null;
  for (const row of rows) {
    bytes += row.bytes;
    oldest = oldest === null ? row.ts_ms : Math.min(oldest, row.ts_ms);
  }
  return { count: rows.length, bytes, oldest_ms: oldest };
}
