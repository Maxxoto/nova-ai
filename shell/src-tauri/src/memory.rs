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
            body: body.trim().to_string(),
        })
    }
}

/// RFC3339 UTC from epoch milliseconds (civil-from-days, chrono-free —
/// mirrors capture_store::day_string).
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
    body TEXT NOT NULL
);
";

impl MemoryStore {
    pub fn open(root: &Path) -> Result<Self, String> {
        let memory_root = root.join("memory");
        for dir in ["episodic", "semantic", "procedural", "digest"] {
            std::fs::create_dir_all(memory_root.join(dir)).map_err(|e| format!("mkdir: {e}"))?;
        }
        let conn =
            Connection::open(memory_root.join("index.sqlite")).map_err(|e| e.to_string())?;
        conn.execute_batch(INDEX_SCHEMA).map_err(|e| e.to_string())?;
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
            "INSERT OR REPLACE INTO memory (id, kind, created, tags, source_refs, path, body)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
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
            ],
        )
        .map_err(|e| e.to_string())?;
        if self.fts {
            let _ = conn.execute("INSERT INTO memory_fts(memory_fts) VALUES('delete-all')", []);
            let _ = conn.execute(
                "INSERT INTO memory_fts(rowid, body, tags)
                 SELECT rowid, body, tags FROM memory",
                [],
            );
        }
        drop(conn);
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
                "INSERT OR REPLACE INTO memory (id, kind, created, tags, source_refs, path, body)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
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
        let scored: Vec<(String, String, String, String, f64)> = if self.fts && !query.is_empty()
        {
            let sql = format!(
                "SELECT m.id, m.kind, m.body, m.source_refs, bm25(memory_fts)
                 FROM memory m JOIN memory_fts ON m.rowid = memory_fts.rowid
                 WHERE memory_fts MATCH ?1 {type_filter}
                 ORDER BY bm25(memory_fts) LIMIT 50"
            );
            conn.prepare(&sql)
                .map_err(|e| e.to_string())?
                .query_map(rusqlite::params![fts_query(query)], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f64>(4)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .map(|(id, kind, body, refs, rank)| {
                    let norm = 1.0 / (1.0 + rank.abs());
                    (id, kind, body, refs, norm)
                })
                .collect()
        } else {
            let pattern = format!("%{}%", query.replace('%', ""));
            let sql = format!(
                "SELECT id, kind, body, source_refs, 0.5 FROM memory
                 WHERE body LIKE ?1 {type_filter} LIMIT 50"
            );
            conn.prepare(&sql)
                .map_err(|e| e.to_string())?
                .query_map(rusqlite::params![pattern], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f64>(4)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect()
        };
        let mut hits: Vec<MemoryHit> = scored
            .into_iter()
            .map(|(id, kind, body, refs, mut score)| {
                if kind == "episodic" {
                    let age_days =
                        age_ms_from_iso(&body_age_lookup(&conn, &id).unwrap_or_default())
                            as f64
                            / 86_400_000.0;
                    score *= 1.0 + 0.3 * (1.0 - (age_days / 14.0).clamp(0.0, 1.0));
                }
                let snippet = snippet_of(&body, 140);
                MemoryHit {
                    id,
                    kind,
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
        Ok(diversified)
    }

    pub fn lookup(&self, id: &str) -> Result<Option<MemoryEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, kind, created, tags, source_refs, body FROM memory WHERE id = ?1",
            [id],
            |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    kind: MemoryType::from_wire(&row.get::<_, String>(1)?)
                        .unwrap_or(MemoryType::Episodic),
                    created: row.get(2)?,
                    tags: split_ws(&row.get::<_, String>(3)?),
                    source_refs: split_ws(&row.get::<_, String>(4)?),
                    confidence: 1.0,
                    origin: String::new(),
                    body: row.get(5)?,
                })
            },
        )
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e.to_string()) })
    }
}

fn body_age_lookup(conn: &Connection, id: &str) -> Option<String> {
    conn.query_row("SELECT created FROM memory WHERE id = ?1", [id], |r| {
        r.get::<_, String>(0)
    })
    .ok()
}

fn split_ws(raw: &str) -> Vec<String> {
    raw.split_whitespace().map(str::to_string).collect()
}

fn snippet_of(body: &str, max: usize) -> String {
    let clean: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    clean.chars().take(max).collect()
}

/// FTS5 prefix query: every token becomes `tok*` so short queries still hit.
fn fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t.trim_matches('"')))
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryHit {
    pub id: String,
    pub kind: String,
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
) -> Result<String, String> {
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
    let mut entry = MemoryEntry::new(MemoryType::Semantic, body);
    entry.tags = tags;
    entry.source_refs = source_refs;
    entry.origin = "user-save".to_string();
    store.write(&entry)?;
    Ok(entry.id)
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
        matches!(method, "memory.search" | "memory.lookup")
    }

    fn route(&self, method: &str, params: &serde_json::Value) -> Result<serde_json::Value, String> {
        match method {
            "memory.search" => self.search(params),
            "memory.lookup" => self.lookup(params),
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
        assert_eq!(fts_query("krebs cycle"), "\"krebs\"* \"cycle\"*");
    }
}
