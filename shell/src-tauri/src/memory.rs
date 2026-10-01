//! Memory store (M2, RFC-0006): episodic/semantic/procedural entries as
//! Markdown + front-matter under `<data_dir>/memory/`, with a rebuildable
//! SQLite FTS index. Files are the source of truth; the index is a cache.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::RequestRouter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    Episodic,
    Semantic,
    Procedural,
}

impl MemoryType {
    fn as_str(self) -> &'static str {
        match self {
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
        }
    }

    fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "episodic" => Some(MemoryType::Episodic),
            "semantic" => Some(MemoryType::Semantic),
            "procedural" => Some(MemoryType::Procedural),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub kind: MemoryType,
    pub created: String,
    pub source_refs: Vec<String>,
    pub tags: Vec<String>,
    pub confidence: f32,
    pub origin: String,
    /// Last time retrieval surfaced this note; resets the recency clock
    /// (Generative Agents). Derived signal — the index column is refreshed
    /// on search, files carry it when known.
    pub last_accessed: Option<String>,
    pub body: String,
}

impl MemoryEntry {
    pub fn new(kind: MemoryType, body: String) -> Self {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            id: format!("mem_{}", Ulid::from_parts((ms as u64) << 16 | 1, 0)),
            kind,
            created: iso_utc(ms),
            source_refs: Vec::new(),
            tags: Vec::new(),
            confidence: 1.0,
            origin: "auto".to_string(),
            last_accessed: None,
            body,
        }
    }

    fn to_markdown(&self) -> String {
        let mut out = String::from("---\n");
        out.push_str(&format!("id: {}\n", self.id));
        out.push_str(&format!("type: {}\n", self.kind.as_str()));
        out.push_str(&format!("created: {}\n", self.created));
        out.push_str(&format!("source_refs: [{}]\n", self.source_refs.join(", ")));
        out.push_str(&format!("tags: [{}]\n", self.tags.join(", ")));
        out.push_str(&format!("confidence: {:.2}\n", self.confidence));
        out.push_str(&format!("origin: {}\n", self.origin));
        if let Some(last) = &self.last_accessed {
            out.push_str(&format!("last_accessed: {last}\n"));
        }
        out.push_str("---\n\n");
        out.push_str(&self.body);
        out.push('\n');
        out
    }

    fn from_markdown(text: &str, path: &Path) -> Result<Self, String> {
        let rest = text
            .strip_prefix("---\n")
            .ok_or_else(|| format!("{}: missing front-matter", path.display()))?;
        let (front, body) = rest
            .split_once("\n---\n")
            .ok_or_else(|| format!("{}: unterminated front-matter", path.display()))?;
        let mut fields: HashMap<String, String> = HashMap::new();
        for line in front.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                fields.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        let list = |key: &str| -> Vec<String> {
            fields
                .get(key)
                .map(|raw| {
                    raw.trim_matches(['[', ']'])
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default()
        };
        let id = fields
            .get("id")
            .cloned()
            .ok_or_else(|| format!("{}: missing id", path.display()))?;
        let kind = fields
            .get("type")
            .and_then(|t| MemoryType::from_wire(t))
            .ok_or_else(|| format!("{}: bad type", path.display()))?;
        Ok(Self {
            id,
            kind,
            created: fields
                .get("created")
                .cloned()
                .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string()),
            source_refs: list("source_refs"),
            tags: list("tags"),
            confidence: fields
                .get("confidence")
                .and_then(|c| c.parse().ok())
                .unwrap_or(1.0),
            origin: fields.get("origin").cloned().unwrap_or_else(|| "auto".to_string()),
            last_accessed: fields.get("last_accessed").cloned(),
            body: body.trim().to_string(),
        })
    }
}

/// RFC3339 UTC from epoch milliseconds (civil-from-days, chrono-free —
/// mirrors capture_store::day_string).
/// Adds columns older stores predate. SQLite has no `ADD COLUMN IF NOT
/// EXISTS`, so presence is checked through the table info.
fn migrate_index(conn: &Connection) {
    let has = |col: &str| {
        conn.prepare("PRAGMA table_info(memory)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())?;
                Ok::<_, rusqlite::Error>(names.iter().any(|n| n == col))
            })
            .unwrap_or(false)
    };
    if !has("origin") {
        let _ = conn.execute(
            "ALTER TABLE memory ADD COLUMN origin TEXT NOT NULL DEFAULT 'auto'",
            [],
        );
    }
    if !has("last_accessed") {
        let _ = conn.execute("ALTER TABLE memory ADD COLUMN last_accessed TEXT", []);
    }
}

pub fn iso_utc(ts_ms: u64) -> String {
    let days = (ts_ms / 86_400_000) as i64;
    let date = civil_from_days(days);
    let secs = (ts_ms % 86_400_000) / 1000;
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        date,
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

fn civil_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn age_ms_from_iso(iso: &str) -> u64 {
    // Only the date part is needed for an age estimate (day granularity).
    let days_since_epoch = |y: i64, m: i64, d: i64| {
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let mp = if m > 2 { m - 3 } else { m + 9 };
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    };
    let parts: Vec<i64> = iso
        .get(0..10)
        .unwrap_or("1970-01-01")
        .split('-')
        .filter_map(|p| p.parse().ok())
        .collect();
    if parts.len() != 3 {
        return 0;
    }
    let file_days = days_since_epoch(parts[0], parts[1], parts[2]);
    let now_days = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
        / 86_400_000) as i64;
    now_days.saturating_sub(file_days) as u64 * 86_400_000
}

pub struct MemoryStore {
    conn: std::sync::Mutex<Connection>,
    root: PathBuf,
    fts: bool,
}

const INDEX_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS memory (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    created TEXT NOT NULL,
    tags TEXT NOT NULL,
    source_refs TEXT NOT NULL,
    path TEXT NOT NULL,
    body TEXT NOT NULL,
    origin TEXT NOT NULL DEFAULT 'auto',
    last_accessed TEXT
);
";

impl MemoryStore {
    pub fn open(root: &Path) -> Result<Self, String> {
        let memory_root = root.join("memory");
        for dir in ["episodic", "semantic", "procedural", "digest", "archive"] {
            std::fs::create_dir_all(memory_root.join(dir)).map_err(|e| format!("mkdir: {e}"))?;
        }
        let conn =
            Connection::open(memory_root.join("index.sqlite")).map_err(|e| e.to_string())?;
        conn.execute_batch(INDEX_SCHEMA).map_err(|e| e.to_string())?;
        migrate_index(&conn);
        let fts = conn
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(body, tags, content='memory', content_rowid='rowid');",
            )
            .is_ok();
        let store = Self {
            conn: std::sync::Mutex::new(conn),
            root: memory_root,
            fts,
        };
        store.rebuild_if_drifted().map_err(|e| e.to_string())?;
        Ok(store)
    }

    fn entry_path(&self, entry: &MemoryEntry) -> PathBuf {
        match entry.kind {
            MemoryType::Episodic => {
                let date = &entry.created[0..10];
                let mut parts = vec!["episodic".to_string()];
                parts.extend(date.split('-').map(str::to_string));
                self.root.join(parts.join("/")).join(format!("{}.md", entry.id))
            }
            _ => self
                .root
                .join(entry.kind.as_str())
                .join(format!("{}.md", entry.id)),
        }
    }

    /// Writes the entry file, inserts the index row, refreshes the digest.
    /// The index is the mirror of the file — one writer (this process), no
    /// triggers needed.
    pub fn write(&self, entry: &MemoryEntry) -> Result<(), String> {
        let path = self.entry_path(entry);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
        }
        std::fs::write(&path, entry.to_markdown()).map_err(|e| format!("write: {e}"))?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO memory (id, kind, created, tags, source_refs, path, body, origin, last_accessed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                entry.id,
                entry.kind.as_str(),
                entry.created,
                entry.tags.join(" "),
                entry.source_refs.join(" "),
                path.strip_prefix(&self.root)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
                entry.body,
                entry.origin,
                entry.last_accessed,
            ],
        )
        .map_err(|e| e.to_string())?;
        drop(conn);
        self.reindex()
    }

    /// Rebuilds the FTS mirror and the digest from the index — the shared
    /// tail of every mutation (write, archive, restore).
    fn reindex(&self) -> Result<(), String> {
        {
            let conn = self.conn.lock().map_err(|e| e.to_string())?;
            if self.fts {
                let _ = conn.execute("INSERT INTO memory_fts(memory_fts) VALUES('delete-all')", []);
                let _ = conn.execute(
                    "INSERT INTO memory_fts(rowid, body, tags)
                     SELECT rowid, body, tags FROM memory",
                    [],
                );
            }
        }
        self.refresh_digest()
    }

    fn refresh_digest(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT kind, body, tags FROM memory
                 WHERE kind != 'episodic' ORDER BY created DESC LIMIT 50",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        let mut digest = String::from("# Memory digest (generated — do not edit)\n\n");
        for kind in ["semantic", "procedural"] {
            let notes = rows.iter().filter(|(k, ..)| k == kind).collect::<Vec<_>>();
            if notes.is_empty() {
                continue;
            }
            digest.push_str(&format!("## {kind}\n\n"));
            for (_, body, tags) in notes {
                let title = body.lines().next().unwrap_or("(untitled)").trim_start_matches("# ").to_string();
                digest.push_str(&format!("- {title}"));
                if !tags.is_empty() {
                    digest.push_str(&format!("  [{tags}]"));
                }
                digest.push('\n');
            }
            digest.push('\n');
        }
        std::fs::write(self.root.join("digest").join("MEMORY.md"), digest)
            .map_err(|e| format!("digest: {e}"))
    }

    /// Files win: rescan when the index disagrees with the directory tree.
    fn rebuild_if_drifted(&self) -> Result<(), String> {
        let file_count = self.scan_files()?.len();
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let indexed: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        drop(conn);
        if file_count != indexed as usize {
            self.rebuild()?;
        }
        Ok(())
    }

    fn scan_files(&self) -> Result<Vec<(PathBuf, MemoryEntry)>, String> {
        let mut found = Vec::new();
        for kind_dir in ["episodic", "semantic", "procedural"] {
            let base = self.root.join(kind_dir);
            let mut stack = vec![base];
            while let Some(dir) = stack.pop() {
                let entries = match std::fs::read_dir(&dir) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        let text = std::fs::read_to_string(&path)
                            .map_err(|e| format!("{}: {e}", path.display()))?;
                        if let Ok(parsed) = MemoryEntry::from_markdown(&text, &path) {
                            found.push((path, parsed));
                        }
                    }
                }
            }
        }
        Ok(found)
    }

    pub fn rebuild(&self) -> Result<usize, String> {
        let files = self.scan_files()?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch("DELETE FROM memory;").map_err(|e| e.to_string())?;
        for (path, entry) in &files {
            conn.execute(
                "INSERT OR REPLACE INTO memory (id, kind, created, tags, source_refs, path, body, origin, last_accessed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    entry.id,
                    entry.kind.as_str(),
                    entry.created,
                    entry.tags.join(" "),
                    entry.source_refs.join(" "),
                    path.strip_prefix(&self.root)
                        .unwrap_or(path)
                        .display()
                        .to_string(),
                    entry.body,
                    entry.origin,
                    entry.last_accessed,
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        if self.fts {
            let _ = conn.execute("INSERT INTO memory_fts(memory_fts) VALUES('delete-all')", []);
            let _ = conn.execute(
                "INSERT INTO memory_fts(rowid, body, tags) SELECT rowid, body, tags FROM memory",
                [],
            );
        }
        drop(conn);
        self.refresh_digest()?;
        Ok(files.len())
    }

    /// FTS bm25 when available, token-LIKE otherwise; episodic recency boost;
    /// at most 3 of one type in the top-k (RFC-0006 §4.4).
    /// RFC-0011 D1 dedupe: an identical saved body returns the existing note
    /// so the UI can say "already saved" instead of writing a copy.
    pub fn find_identical(&self, body: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id FROM memory WHERE body = ?1 LIMIT 1",
            rusqlite::params![body],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other.to_string()),
        })
    }

    /// Newest-first listing for the management surface (F-16); a query
    /// filters through the same FTS path as search.
    pub fn list(&self, query: Option<&str>, k: usize) -> Result<Vec<MemoryRow>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let query = query.map(str::trim).filter(|q| !q.is_empty());
        let mut stmt;
        let rows: Vec<(String, String, String, String, String)> = match query {
            Some(q) if self.fts => {
                stmt = conn
                    .prepare(
                        "SELECT m.id, m.kind, m.created, m.source_refs, m.body
                         FROM memory m JOIN memory_fts ON m.rowid = memory_fts.rowid
                         WHERE memory_fts MATCH ?1 ORDER BY bm25(memory_fts) LIMIT ?2",
                    )
                    .map_err(|e| e.to_string())?;
                let iter = stmt
                    .query_map(rusqlite::params![fts_query(q), k as i64], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    })
                    .map_err(|e| e.to_string())?;
                iter.filter_map(|r| r.ok()).collect()
            }
            Some(q) => {
                let pattern = format!("%{}%", q.replace('%', ""));
                stmt = conn
                    .prepare(
                        "SELECT id, kind, created, source_refs, body FROM memory
                         WHERE body LIKE ?1 ORDER BY created DESC LIMIT ?2",
                    )
                    .map_err(|e| e.to_string())?;
                let iter = stmt
                    .query_map(rusqlite::params![pattern, k as i64], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    })
                    .map_err(|e| e.to_string())?;
                iter.filter_map(|r| r.ok()).collect()
            }
            None => {
                stmt = conn
                    .prepare(
                        "SELECT id, kind, created, source_refs, body FROM memory
                         ORDER BY created DESC LIMIT ?1",
                    )
                    .map_err(|e| e.to_string())?;
                let iter = stmt
                    .query_map(rusqlite::params![k as i64], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    })
                    .map_err(|e| e.to_string())?;
                iter.filter_map(|r| r.ok()).collect()
            }
        };
        Ok(rows
            .into_iter()
            .map(|(id, kind, created, refs, body)| MemoryRow {
                id,
                kind,
                created,
                source_refs: refs.split_whitespace().map(str::to_string).collect(),
                snippet: snippet_of(&body, 160),
            })
            .collect())
    }

    /// RFC-0011 D4: archive-first. The note leaves retrieval but nothing is
    /// deleted — the markdown moves to `memory/archive/` and `restore` puts
    /// it back.
    pub fn archive(&self, id: &str) -> Result<(), String> {
        let rel: String = {
            let conn = self.conn.lock().map_err(|e| e.to_string())?;
            let rel = conn
                .query_row(
                    "SELECT path FROM memory WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get::<_, String>(0),
                )
                .map_err(|_| format!("no such memory: {id}"))?;
            conn.execute("DELETE FROM memory WHERE id = ?1", rusqlite::params![id])
                .map_err(|e| e.to_string())?;
            rel
        };
        let src = self.root.join(&rel);
        let dest = self.root.join("archive").join(format!("{id}.md"));
        std::fs::rename(&src, &dest).map_err(|e| format!("archive move: {e}"))?;
        self.reindex()
    }

    /// The archived notes, newest first — for the restore list.
    pub fn archived(&self) -> Result<Vec<MemoryRow>, String> {
        let dir = self.root.join("archive");
        let mut rows = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(parsed) = MemoryEntry::from_markdown(&text, &path) else {
                    continue;
                };
                rows.push(MemoryRow {
                    id: parsed.id,
                    kind: parsed.kind.as_str().to_string(),
                    created: parsed.created,
                    source_refs: parsed.source_refs,
                    snippet: snippet_of(&parsed.body, 160),
                });
            }
        }
        rows.sort_by(|a, b| b.created.cmp(&a.created));
        Ok(rows)
    }

    /// Yesterday's diary for the reflection pass (RFC-0011 D7): episodic
    /// entries created at or after `since_iso`, newest first.
    pub fn episodic_since(
        &self,
        since_iso: &str,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, created, body, source_refs FROM memory
                 WHERE kind = 'episodic' AND created >= ?1
                 ORDER BY created DESC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![since_iso, limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        drop(stmt);
        drop(conn);
        Ok(rows
            .into_iter()
            .map(|(id, created, body, refs)| {
                serde_json::json!({
                    "id": id,
                    "created": created,
                    "body": body,
                    "source_refs": refs.split_whitespace().map(str::to_string).collect::<Vec<_>>(),
                })
            })
            .collect())
    }

    /// Marker for "reflection already ran for this day" (RFC-0011 D7).
    pub fn reflection_marker_path(&self) -> PathBuf {
        self.root.join("digest").join(".last-reflection")
    }

    /// Where reflection queues supersession proposals for the user.
    pub fn review_queue_path(&self) -> PathBuf {
        self.root.join("digest").join("REVIEW.md")
    }

    /// Puts an archived note back into its kind shard and the index.
    pub fn restore(&self, id: &str) -> Result<(), String> {
        let path = self.root.join("archive").join(format!("{id}.md"));
        let text = std::fs::read_to_string(&path).map_err(|_| format!("no archived memory: {id}"))?;
        let entry = MemoryEntry::from_markdown(&text, &path)?;
        self.write(&entry)?;
        std::fs::remove_file(&path).map_err(|e| format!("remove archived copy: {e}"))
    }

    pub fn search(
        &self,
        query: &str,
        kinds: Option<&[MemoryType]>,
        k: usize,
    ) -> Result<Vec<MemoryHit>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let type_filter = kinds
            .map(|ks| {
                let names: Vec<String> = ks.iter().map(|k| format!("'{}'", k.as_str())).collect();
                format!("AND kind IN ({})", names.join(","))
            })
            .unwrap_or_default();

        // Time-aware queries (RFC-0011 D6): a relative-date phrase becomes a
        // `created >=` filter instead of poisoning the keyword match.
        let (since_filter, query) = relative_since(query);
        let since_clause = since_filter
            .as_ref()
            .map(|since| format!("AND created >= '{since}'"))
            .unwrap_or_default();

        // id, kind, created, origin, last_accessed, body, source_refs, fts_norm
        let scored: Vec<(String, String, String, String, Option<String>, String, String, f64)> =
            if self.fts && !query.is_empty() {
                let sql = format!(
                    "SELECT m.id, m.kind, m.created, m.origin, m.last_accessed, m.body, m.source_refs, bm25(memory_fts)
                     FROM memory m JOIN memory_fts ON m.rowid = memory_fts.rowid
                     WHERE memory_fts MATCH ?1 {type_filter} {since_clause}
                     ORDER BY bm25(memory_fts) LIMIT 50"
                );
                conn.prepare(&sql)
                    .map_err(|e| e.to_string())?
                    .query_map(rusqlite::params![fts_query(&query)], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get::<_, f64>(7)?,
                        ))
                    })
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .map(|row| {
                        // FTS5's bm25 is more-negative-is-better, so the
                        // composite weight must grow with |bm25| — a naive
                        // 1/(1+|rank|) inverted quality and let weak stopword
                        // matches outrank real hits.
                        let mut row = row;
                        row.7 = row.7.abs() / (1.0 + row.7.abs());
                        row
                    })
                    .collect()
            } else {
                // No keyword left (a pure "what did I save last week?") or no
                // FTS: rank the date window by recency alone.
                let pattern = format!("%{}%", query.replace('%', ""));
                let sql = format!(
                    "SELECT id, kind, created, origin, last_accessed, body, source_refs, 0.5 FROM memory
                     WHERE (body LIKE ?1 OR ?1 = '%%') {type_filter} {since_clause}
                     ORDER BY created DESC LIMIT 50"
                );
                conn.prepare(&sql)
                    .map_err(|e| e.to_string())?
                    .query_map(rusqlite::params![pattern], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get::<_, f64>(7)?,
                        ))
                    })
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .collect()
            };

        let mut hits: Vec<MemoryHit> = scored
            .into_iter()
            .map(|(id, kind, created, origin, last_accessed, body, refs, fts)| {
                let score = composite_score(&kind, &origin, &created, last_accessed.as_deref(), fts);
                let snippet = snippet_of(&body, 140);
                MemoryHit {
                    id,
                    kind,
                    created,
                    snippet,
                    score,
                    source_refs: refs
                        .split_whitespace()
                        .map(str::to_string)
                        .collect(),
                }
            })
            .collect();
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let mut diversified = Vec::new();
        let mut per_type: HashMap<String, usize> = HashMap::new();
        for hit in hits {
            let count = per_type.get(&hit.kind).copied().unwrap_or(0);
            if count < 3 {
                per_type.insert(hit.kind.clone(), count + 1);
                diversified.push(hit);
                if diversified.len() == k {
                    break;
                }
            }
        }

        // Retrieval resets the recency clock on what it surfaced — the next
        // search sees these as fresher (Generative Agents).
        let now = iso_utc(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        );
        for hit in &diversified {
            let _ = conn.execute(
                "UPDATE memory SET last_accessed = ?1 WHERE id = ?2",
                rusqlite::params![now, hit.id],
            );
        }

        Ok(diversified)
    }

    pub fn lookup(&self, id: &str) -> Result<Option<MemoryEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, kind, created, tags, source_refs, body, origin, last_accessed FROM memory WHERE id = ?1",
            [id],
            |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    kind: MemoryType::from_wire(&row.get::<_, String>(1)?)
                        .unwrap_or(MemoryType::Episodic),
                    created: row.get(2)?,
                    tags: split_ws(&row.get::<_, String>(3)?),
                    source_refs: split_ws(&row.get::<_, String>(4)?),
                    origin: row.get::<_, String>(6).unwrap_or_else(|_| "auto".to_string()),
                    last_accessed: row.get::<_, Option<String>>(7).ok().flatten(),
                    confidence: 1.0,
                    body: row.get(5)?,
                })
            },
        )
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e.to_string()) })
    }
}

fn split_ws(raw: &str) -> Vec<String> {
    raw.split_whitespace().map(str::to_string).collect()
}

/// Composite ranking weights (RFC-0011 D6; ScalyClaw/Generative-Agents
/// lineage): keyword match dominates, recency and importance break ties and
/// lift fresh/pinned notes over stale chatter.
const W_FTS: f64 = 0.6;
const W_REC: f64 = 0.2;
const W_IMP: f64 = 0.2;
/// Half-life of a note's recency score, in days.
const RECENCY_HALF_LIFE_DAYS: f64 = 14.0;

fn composite_score(
    kind: &str,
    origin: &str,
    created: &str,
    last_accessed: Option<&str>,
    fts: f64,
) -> f64 {
    let age_days = age_ms_from_iso(last_accessed.unwrap_or(created)) as f64 / 86_400_000.0;
    let recency = 0.5_f64.powf(age_days / RECENCY_HALF_LIFE_DAYS);
    let base: f64 = match kind {
        "semantic" => 0.8,
        "procedural" => 0.6,
        _ => 0.35,
    };
    let importance = if origin == "user-save" { (base + 0.2).min(1.0) } else { base };
    W_FTS * fts + W_REC * recency + W_IMP * importance
}

/// Extracts a relative-date phrase ("last week", "yesterday", "last 3 days")
/// from a query: returns the `created >=` floor and the query with the phrase
/// removed, so the words do not pollute the keyword match.
fn relative_since(query: &str) -> (Option<String>, String) {
    let lower = query.to_lowercase();
    let days_for = |word: &str| -> Option<u64> {
        match word {
            "today" => Some(0),
            "yesterday" => Some(1),
            "this week" | "last week" | "past week" | "recently" => Some(7),
            "this month" | "last month" | "past month" => Some(31),
            _ => None,
        }
    };
    for phrase in [
        "today",
        "yesterday",
        "this week",
        "last week",
        "past week",
        "recently",
        "this month",
        "last month",
        "past month",
    ] {
        if lower.contains(phrase) {
            let cleaned = lower.replacen(phrase, "", 1);
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let since = iso_utc(ms.saturating_sub(7 * 86_400_000));
            let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
            let cleaned = cleaned.replace(" ?", "?").replace(" ,", ",");
            return (Some(since), cleaned);
        }
    }
    // "last N days/weeks/months"
    let split: Vec<&str> = lower.split_whitespace().collect();
    for window in split.windows(3) {
        if window[0] == "last" {
            if let Ok(n) = window[1].parse::<u64>() {
                let unit = window[2].trim_end_matches('s');
                let mult = match unit {
                    "day" => Some(1),
                    "week" => Some(7),
                    "month" => Some(31),
                    _ => None,
                };
                if let Some(mult) = mult {
                    let cleaned = lower.replacen(&format!("last {} {}", window[1], window[2]), "", 1);
                    let ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    let since = iso_utc(ms.saturating_sub(n * mult * 86_400_000));
                    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
            let cleaned = cleaned.replace(" ?", "?").replace(" ,", ",");
            return (Some(since), cleaned);
                }
            }
        }
    }
    let _ = days_for;
    (None, query.to_string())
}

fn snippet_of(body: &str, max: usize) -> String {
    let clean: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    clean.chars().take(max).collect()
}

/// FTS5 prefix query: every token becomes `tok*` so short queries still hit.
fn fts_query(query: &str) -> String {
    query
        .split_whitespace()
        // Single-character tokens are stopwords with near-zero signal, but a
        // prefix match on "a"* still hits almost every document.
        .filter(|t| t.len() > 1)
        .map(|t| format!("\"{}\"*", t.trim_matches('"')))
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryRow {
    pub id: String,
    pub kind: String,
    pub created: String,
    pub source_refs: Vec<String>,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryHit {
    pub id: String,
    pub kind: String,
    pub created: String,
    pub snippet: String,
    pub score: f64,
    pub source_refs: Vec<String>,
}

#[tauri::command]
pub fn memory_save_semantic(
    app: tauri::AppHandle,
    title: String,
    body: String,
    tags: Vec<String>,
    source_refs: Vec<String>,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("data dir: {e}"))?;
    let store = MemoryStore::open(&dir)?;
    let body = if body.trim_start().starts_with('#') {
        body.trim().to_string()
    } else {
        format!("# {title}\n\n{body}")
    };
    // RFC-0011 D1: an identical save returns the existing note instead of a copy.
    if let Some(existing) = store.find_identical(&body)? {
        return Ok(serde_json::json!({ "id": existing, "created": false }));
    }
    let mut entry = MemoryEntry::new(MemoryType::Semantic, body);
    entry.tags = tags;
    entry.source_refs = source_refs;
    entry.origin = "user-save".to_string();
    store.write(&entry)?;
    Ok(serde_json::json!({ "id": entry.id, "created": true }))
}

fn memory_store(app: &tauri::AppHandle) -> Result<MemoryStore, String> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("data dir: {e}"))?;
    MemoryStore::open(&dir)
}

#[tauri::command]
pub fn memory_list(
    app: tauri::AppHandle,
    query: Option<String>,
) -> Result<serde_json::Value, String> {
    let store = memory_store(&app)?;
    let rows = store.list(query.as_deref(), 200)?;
    serde_json::to_value(rows).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn memory_archive(app: tauri::AppHandle, id: String) -> Result<(), String> {
    memory_store(&app)?.archive(&id)
}

#[tauri::command]
pub fn memory_archived_list(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let rows = memory_store(&app)?.archived()?;
    serde_json::to_value(rows).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn memory_restore(app: tauri::AppHandle, id: String) -> Result<(), String> {
    memory_store(&app)?.restore(&id)
}

pub struct MemoryRouter {
    store: std::sync::Arc<MemoryStore>,
}

impl MemoryRouter {
    pub fn new(store: std::sync::Arc<MemoryStore>) -> Self {
        Self { store }
    }

    fn search(&self, params: &serde_json::Value) -> Result<serde_json::Value, String> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("memory.search requires string query")?;
        let kinds = params
            .get("types")
            .and_then(|v| v.as_array())
            .map(|list| {
                list.iter()
                    .filter_map(|t| t.as_str().and_then(MemoryType::from_wire))
                    .collect::<Vec<_>>()
            });
        let k = params.get("k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        let hits = self
            .store
            .search(query, kinds.as_deref(), k.clamp(1, 10))?;
        serde_json::to_value(hits).map_err(|e| e.to_string())
    }

    fn digest(&self) -> Result<serde_json::Value, String> {
        let path = self.store.root.join("digest").join("MEMORY.md");
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        Ok(serde_json::json!({ "digest": text }))
    }

    fn lookup(&self, params: &serde_json::Value) -> Result<serde_json::Value, String> {
        let id = params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("memory.lookup requires string id")?;
        match self.store.lookup(id)? {
            Some(entry) => serde_json::to_value(entry).map_err(|e| e.to_string()),
            None => Err(format!("memory {id} not found")),
        }
    }
}

impl RequestRouter for MemoryRouter {
    fn handles(&self, method: &str) -> bool {
        matches!(method, "memory.search" | "memory.lookup" | "memory.digest")
    }

    fn route(&self, method: &str, params: &serde_json::Value) -> Result<serde_json::Value, String> {
        match method {
            "memory.search" => self.search(params),
            "memory.lookup" => self.lookup(params),
            "memory.digest" => self.digest(),
            other => Err(format!("unknown method: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(tag: &str) -> MemoryStore {
        let dir = std::env::temp_dir().join(format!("ruoxi-mem-test-{tag}-{}", Ulid::generate()));
        MemoryStore::open(&dir).expect("open")
    }

    #[test]
    fn markdown_round_trip() {
        let mut entry = MemoryEntry::new(MemoryType::Semantic, "# Title\nBody.".to_string());
        entry.tags = vec!["study".to_string(), "bio".to_string()];
        entry.source_refs = vec!["cap_123".to_string()];
        entry.origin = "user-save".to_string();
        let parsed =
            MemoryEntry::from_markdown(&entry.to_markdown(), Path::new("x.md")).expect("parse");
        assert_eq!(entry, parsed);
    }

    #[test]
    fn episodic_lands_in_date_shard() {
        let store = temp_store("shard");
        let entry = MemoryEntry::new(MemoryType::Episodic, "asked about mitochondria".to_string());
        let path = store.entry_path(&entry);
        let date = &entry.created[0..10];
        assert!(path
            .to_string_lossy()
            .contains(&format!("episodic/{}/{}", &date[0..4], &date[5..7])));
        store.write(&entry).expect("write");
        assert!(path.exists());
    }

    #[test]
    fn identical_save_is_deduped() {
        let store = temp_store("dedupe");
        let body = "# Note\nSame text twice.".to_string();
        let mut entry = MemoryEntry::new(MemoryType::Semantic, body.clone());
        entry.origin = "user-save".to_string();
        store.write(&entry).expect("write");
        assert_eq!(store.find_identical(&body).expect("find"), Some(entry.id.clone()));
        assert_eq!(store.find_identical("different").expect("find"), None);
    }

    #[test]
    fn archive_leaves_search_and_restore_returns() {
        let store = temp_store("archive");
        let mut entry = MemoryEntry::new(
            MemoryType::Semantic,
            "# Keep me\nunique-needle-xyz lives here".to_string(),
        );
        entry.origin = "user-save".to_string();
        store.write(&entry).expect("write");
        assert!(!store.search("unique-needle-xyz", None, 10).expect("search").is_empty());

        store.archive(&entry.id).expect("archive");
        assert!(store.search("unique-needle-xyz", None, 10).expect("search").is_empty());
        let archived = store.archived().expect("archived");
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].id, entry.id);

        store.restore(&entry.id).expect("restore");
        assert!(!store.search("unique-needle-xyz", None, 10).expect("search").is_empty());
        assert!(store.archived().expect("archived").is_empty());
    }

    #[test]
    fn list_is_newest_first_and_query_filters() {
        let store = temp_store("list");
        let mut first = MemoryEntry::new(MemoryType::Semantic, "# A\nalpha-topic".to_string());
        first.origin = "user-save".to_string();
        first.created = "2026-01-01T00:00:00Z".to_string();
        store.write(&first).expect("write");
        let mut second = MemoryEntry::new(MemoryType::Semantic, "# B\nbeta-topic".to_string());
        second.origin = "user-save".to_string();
        second.created = "2026-01-02T00:00:00Z".to_string();
        store.write(&second).expect("write");

        let all = store.list(None, 10).expect("list");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, second.id);

        let filtered = store.list(Some("alpha-topic"), 10).expect("list");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, first.id);
    }

    #[test]
    fn relative_date_phrase_filters_without_poisoning_match() {
        let store = temp_store("time");
        let mut ancient = MemoryEntry::new(
            MemoryType::Semantic,
            "# Ancient\nkappa-fact from long ago".to_string(),
        );
        ancient.origin = "user-save".to_string();
        ancient.created = "2026-01-01T00:00:00Z".to_string();
        store.write(&ancient).expect("write");
        let mut fresh = MemoryEntry::new(
            MemoryType::Semantic,
            "# Fresh\nkappa-fact from today".to_string(),
        );
        fresh.origin = "user-save".to_string();
        fresh.created = iso_utc(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        );
        store.write(&fresh).expect("write");

        let windowed = store.search("kappa-fact last week", None, 10).expect("search");
        assert_eq!(windowed.len(), 1);
        assert_eq!(windowed[0].id, fresh.id);

        let unfiltered = store.search("kappa-fact", None, 10).expect("search");
        assert_eq!(unfiltered.len(), 2);
    }

    #[test]
    fn newer_note_wins_the_recency_tiebreak() {
        let store = temp_store("recency");
        let mut older = MemoryEntry::new(MemoryType::Semantic, "# A\nlambda-topic".to_string());
        older.created = "2026-01-01T00:00:00Z".to_string();
        store.write(&older).expect("write");
        let mut newer = MemoryEntry::new(MemoryType::Semantic, "# B\nlambda-topic".to_string());
        newer.created = "2026-06-01T00:00:00Z".to_string();
        store.write(&newer).expect("write");
        let hits = store.search("lambda-topic", None, 10).expect("search");
        assert_eq!(hits[0].id, newer.id);
    }

    #[test]
    fn user_save_outranks_auto_at_equal_age() {
        let store = temp_store("importance");
        let mut auto = MemoryEntry::new(MemoryType::Semantic, "# A\nmu-topic".to_string());
        auto.origin = "auto".to_string();
        auto.created = "2026-06-01T00:00:00Z".to_string();
        store.write(&auto).expect("write");
        let mut saved = MemoryEntry::new(MemoryType::Semantic, "# B\nmu-topic".to_string());
        saved.origin = "user-save".to_string();
        saved.created = "2026-06-01T00:00:00Z".to_string();
        store.write(&saved).expect("write");
        let hits = store.search("mu-topic", None, 10).expect("search");
        assert_eq!(hits[0].id, saved.id);
    }

    #[test]
    fn retrieval_bump_makes_the_survivor_sticky() {
        let store = temp_store("bump");
        for i in 0..2 {
            let mut entry = MemoryEntry::new(
                MemoryType::Semantic,
                format!("# {i}\nnu-topic identical body"),
            );
            entry.origin = "auto".to_string();
            entry.created = "2026-06-01T00:00:00Z".to_string();
            store.write(&entry).expect("write");
        }
        let first = store.search("nu-topic", None, 10).expect("search");
        let winner = first[0].id.clone();
        // The bump above reset the winner's recency clock, so a re-run of the
        // same search must keep it on top instead of flip-flopping.
        let second = store.search("nu-topic", None, 10).expect("search");
        assert_eq!(second[0].id, winner);
    }

    #[test]
    fn relative_since_parses_phrases_and_counts() {
        let (since, cleaned) = relative_since("What did I save last week?");
        assert!(since.is_some());
        assert_eq!(cleaned, "what did i save?");
        let (since, cleaned) = relative_since("notes from last 3 days");
        assert!(since.is_some());
        assert_eq!(cleaned, "notes from");
        let (since, cleaned) = relative_since("plain query");
        assert!(since.is_none());
        assert_eq!(cleaned, "plain query");
    }

    #[test]
    fn search_finds_and_diversifies() {
        let store = temp_store("search");
        for i in 0..4 {
            let mut e = MemoryEntry::new(
                MemoryType::Semantic,
                format!("# Note {i}\nmitochondria powers the cell"),
            );
            e.origin = "user-save".to_string();
            store.write(&e).expect("write");
        }
        store
            .write(&MemoryEntry::new(
                MemoryType::Procedural,
                "# Flow\nalways cite mitochondria papers".to_string(),
            ))
            .expect("write");
        let hits = store.search("mitochondria", None, 5).expect("search");
        assert!(!hits.is_empty());
        let semantic = hits.iter().filter(|h| h.kind == "semantic").count();
        assert!(semantic <= 3, "type diversification: {semantic}");
    }

    #[test]
    fn index_rebuilds_from_files_after_drift() {
        let store = temp_store("drift");
        let mut e = MemoryEntry::new(MemoryType::Semantic, "# Saved\nkrebs cycle notes".to_string());
        e.origin = "user-save".to_string();
        store.write(&e).expect("write");
        {
            let conn = store.conn.lock().unwrap();
            conn.execute("DELETE FROM memory", []).unwrap();
        }
        assert!(store.search("krebs", None, 5).unwrap().is_empty());
        store.rebuild().expect("rebuild");
        assert!(!store.search("krebs", None, 5).unwrap().is_empty());
        assert!(store.root.join("digest/MEMORY.md").exists());
    }

    #[test]
    fn iso_utc_shape() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(86_400_000), "1970-01-02T00:00:00Z");
    }

    #[test]
    fn fts_query_quotes_and_prefixes() {
        assert_eq!(fts_query("krebs cycle"), "\"krebs\"* OR \"cycle\"*");
    }
}
