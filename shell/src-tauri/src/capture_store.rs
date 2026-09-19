//! Capture store v0 (W3). RFC-0003 §4.3/§4.7 — date-sharded image files with
//! JSON sidecars, plus a derived SQLite index; exact-sha256 dedupe; retention
//! controls (delete-all / delete-day). The capture *mechanics* (screen APIs)
//! arrive with the S3-gated workstream; this module owns everything that
//! happens after pixels exist.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ulid::Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Region,
    Window,
    Fullscreen,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Scope::Region => "region",
            Scope::Window => "window",
            Scope::Fullscreen => "fullscreen",
        }
    }
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Db(rusqlite::Error),
    Serde(serde_json::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "io: {e}"),
            StoreError::Db(e) => write!(f, "db: {e}"),
            StoreError::Serde(e) => write!(f, "serde: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Db(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Serde(e)
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct NewCapture {
    pub scope: Scope,
    pub ts_ms: u64,
    pub display_id: String,
    pub app: Option<String>,
    pub window_title: Option<String>,
    pub w_px: i64,
    pub h_px: i64,
    pub scale: f64,
    pub image: Vec<u8>,
    pub ext: String,
    pub auto: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRecord {
    pub capture_id: String,
    pub ts: u64,
    pub day: String,
    pub scope: String,
    pub display_id: String,
    pub app: Option<String>,
    pub window_title: Option<String>,
    pub path: String,
    pub w_px: i64,
    pub h_px: i64,
    pub scale: f64,
    pub sha256: String,
    pub retention: String,
    pub auto: bool,
}

#[derive(Debug, Default)]
pub struct TimelineFilter {
    pub day: Option<String>,
    pub app: Option<String>,
    pub scope: Option<Scope>,
    pub limit: u64,
    pub offset: u64,
}

pub struct CaptureStore {
    conn: Mutex<Connection>,
    root: PathBuf,
}

impl CaptureStore {
    pub fn open(root: &Path) -> Result<Self> {
        fs::create_dir_all(root.join("captures"))?;
        let conn = Connection::open(root.join("capture_index.sqlite"))?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
            root: root.to_path_buf(),
        })
    }

    /// Recreates the SQLite index from the JSON sidecars (the index is always
    /// derived data — RFC-0003 §4.3).
    pub fn rebuild_index(root: &Path) -> Result<Self> {
        let sidecars = collect_sidecars(&root.join("captures"))?;
        let store = Self::open(root)?;
        let conn = store.conn.lock().unwrap();
        conn.execute("DELETE FROM captures", [])?;
        for path in sidecars {
            let text = fs::read_to_string(&path)?;
            let record: CaptureRecord = serde_json::from_str(&text)?;
            conn.execute(
                "INSERT INTO captures (capture_id, ts, day, scope, display_id, app, window_title, path, w_px, h_px, scale, sha256, retention, auto)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    record.capture_id,
                    record.ts as i64,
                    record.day,
                    record.scope,
                    record.display_id,
                    record.app,
                    record.window_title,
                    record.path,
                    record.w_px,
                    record.h_px,
                    record.scale,
                    record.sha256,
                    record.retention,
                    record.auto as i64,
                ],
            )?;
        }
        drop(conn);
        Ok(store)
    }

    /// Stores the image + sidecar; exact-sha256 duplicates return the existing
    /// record with `created = false` (RFC-0003 §4.8).
    pub fn insert(&self, new: &NewCapture) -> Result<(CaptureRecord, bool)> {
        let sha = hash_hex(&new.image);
        let conn = self.conn.lock().unwrap();
        let existing: Option<CaptureRecord> = conn
            .query_row(
                "SELECT capture_id, ts, day, scope, display_id, app, window_title, path, w_px, h_px, scale, sha256, retention, auto
                 FROM captures WHERE sha256 = ?1",
                params![sha],
                row_to_record,
            )
            .optional()?;
        if let Some(record) = existing {
            return Ok((record, false));
        }

        let capture_id = format!("cap_{}", Ulid::generate());
        let day = day_string(new.ts_ms);
        let rel_path = format!("captures/{}/{}/{}/{}.{}", &day[0..4], &day[5..7], &day[8..10], capture_id, new.ext);
        let record = CaptureRecord {
            capture_id: capture_id.clone(),
            ts: new.ts_ms,
            day: day.clone(),
            scope: new.scope.as_str().to_string(),
            display_id: new.display_id.clone(),
            app: new.app.clone(),
            window_title: new.window_title.clone(),
            path: rel_path.clone(),
            w_px: new.w_px,
            h_px: new.h_px,
            scale: new.scale,
            sha256: sha,
            retention: "default".to_string(),
            auto: new.auto,
        };

        let abs = self.root.join(&rel_path);
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&abs, &new.image)?;
        let sidecar = abs.with_extension("json");
        fs::write(&sidecar, serde_json::to_string(&record)?)?;
        conn.execute(
            "INSERT INTO captures (capture_id, ts, day, scope, display_id, app, window_title, path, w_px, h_px, scale, sha256, retention, auto)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                record.capture_id,
                record.ts as i64,
                record.day,
                record.scope,
                record.display_id,
                record.app,
                record.window_title,
                record.path,
                record.w_px,
                record.h_px,
                record.scale,
                record.sha256,
                record.retention,
                record.auto as i64,
            ],
        )?;
        Ok((record, true))
    }

    pub fn lookup(&self, capture_id: &str) -> Result<Option<CaptureRecord>> {
        let conn = self.conn.lock().unwrap();
        let record = conn
            .query_row(
                "SELECT capture_id, ts, day, scope, display_id, app, window_title, path, w_px, h_px, scale, sha256, retention, auto
                 FROM captures WHERE capture_id = ?1",
                params![capture_id],
                row_to_record,
            )
            .optional()?;
        Ok(record)
    }

    pub fn timeline(&self, filter: &TimelineFilter) -> Result<Vec<CaptureRecord>> {
        let day = filter.day.clone();
        let app = filter.app.clone();
        let scope = filter.scope.map(|s| s.as_str().to_string());
        let limit = i64::try_from(filter.limit.max(1)).unwrap_or(50);
        let offset = i64::try_from(filter.offset).unwrap_or(0);

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT capture_id, ts, day, scope, display_id, app, window_title, path, w_px, h_px, scale, sha256, retention, auto
             FROM captures
             WHERE (?1 IS NULL OR day = ?1)
               AND (?2 IS NULL OR app = ?2)
               AND (?3 IS NULL OR scope = ?3)
             ORDER BY ts DESC
             LIMIT ?4 OFFSET ?5",
        )?;
        let rows = stmt.query_map(params![day, app, scope, limit, offset], row_to_record)?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn delete_all(&self) -> Result<usize> {
        self.delete_where(None)
    }

    pub fn delete_day(&self, day: &str) -> Result<usize> {
        self.delete_where(Some(day))
    }

    fn delete_where(&self, day: Option<&str>) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let paths: Vec<String> = match day {
            Some(day) => {
                let mut stmt = conn.prepare("SELECT path FROM captures WHERE day = ?1")?;
                let rows = stmt.query_map(params![day], |row| row.get::<_, String>(0))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()?
            }
            None => {
                let mut stmt = conn.prepare("SELECT path FROM captures")?;
                let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()?
            }
        };
        for rel in &paths {
            let _ = fs::remove_file(self.root.join(rel).with_extension("json"));
            let _ = fs::remove_file(self.root.join(rel));
        }
        let removed = match day {
            Some(day) => conn.execute("DELETE FROM captures WHERE day = ?1", params![day])?,
            None => conn.execute("DELETE FROM captures", [])?,
        };
        Ok(removed)
    }
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<CaptureRecord> {
    Ok(CaptureRecord {
        capture_id: row.get(0)?,
        ts: row.get::<_, i64>(1)? as u64,
        day: row.get(2)?,
        scope: row.get(3)?,
        display_id: row.get(4)?,
        app: row.get(5)?,
        window_title: row.get(6)?,
        path: row.get(7)?,
        w_px: row.get(8)?,
        h_px: row.get(9)?,
        scale: row.get(10)?,
        sha256: row.get(11)?,
        retention: row.get(12)?,
        auto: row.get::<_, i64>(13)? != 0,
    })
}

fn collect_sidecars(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let stack = vec![dir.to_path_buf()];
    let mut queue = stack;
    while let Some(dir) = queue.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                queue.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
                found.push(path);
            }
        }
    }
    Ok(found)
}

fn hash_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// UTC "YYYY-MM-DD" from epoch milliseconds, via the civil-from-days
/// algorithm (Howard Hinnant, chrono-free).
fn day_string(ts_ms: u64) -> String {
    let days = (ts_ms / 86_400_000) as i64;
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

use crate::RequestRouter;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS captures (
    capture_id TEXT PRIMARY KEY,
    ts INTEGER NOT NULL,
    day TEXT NOT NULL,
    scope TEXT NOT NULL,
    display_id TEXT NOT NULL,
    app TEXT,
    window_title TEXT,
    path TEXT NOT NULL,
    w_px INTEGER NOT NULL,
    h_px INTEGER NOT NULL,
    scale REAL NOT NULL,
    sha256 TEXT NOT NULL UNIQUE,
    retention TEXT NOT NULL DEFAULT 'default',
    auto INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_captures_ts ON captures (ts);
CREATE INDEX IF NOT EXISTS idx_captures_day ON captures (day);
CREATE INDEX IF NOT EXISTS idx_captures_app ON captures (app);
";

pub struct CaptureRouter {
    store: CaptureStore,
}

impl CaptureRouter {
    pub fn new(store: CaptureStore) -> Self {
        Self { store }
    }

    fn scope_from_wire(raw: &str) -> Option<Scope> {
        match raw {
            "region" => Some(Scope::Region),
            "window" => Some(Scope::Window),
            "fullscreen" => Some(Scope::Fullscreen),
            _ => None,
        }
    }

    fn lookup(&self, params: &serde_json::Value) -> std::result::Result<serde_json::Value, String> {
        let id = params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "capture.lookup requires string id".to_string())?;
        match self.store.lookup(id).map_err(|e| e.to_string())? {
            Some(record) => serde_json::to_value(record).map_err(|e| e.to_string()),
            None => Err(format!("capture {id} not found")),
        }
    }

    fn timeline(&self, params: &serde_json::Value) -> std::result::Result<serde_json::Value, String> {
        let filter = TimelineFilter {
            day: params
                .get("day")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            app: params
                .get("app")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            scope: params
                .get("scope")
                .and_then(|v| v.as_str())
                .and_then(Self::scope_from_wire),
            limit: params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50),
            offset: params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0),
        };
        let records = self.store.timeline(&filter).map_err(|e| e.to_string())?;
        serde_json::to_value(records).map_err(|e| e.to_string())
    }
}

impl RequestRouter for CaptureRouter {
    fn route(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> std::result::Result<serde_json::Value, String> {
        match method {
            "capture.lookup" => self.lookup(params),
            "timeline.query" => self.timeline(params),
            other => Err(format!("unknown method: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureStats {
    pub count: u64,
    pub bytes: u64,
    pub oldest_ms: Option<u64>,
}

/// Summarizes timeline rows + on-disk image sizes without touching the index.
/// `root` is the store root that `record.path` is relative to.
pub fn stats_from(records: &[CaptureRecord], root: &Path) -> CaptureStats {
    let mut bytes = 0u64;
    let mut oldest_ms: Option<u64> = None;
    for record in records {
        if let Ok(meta) = fs::metadata(root.join(&record.path)) {
            bytes = bytes.saturating_add(meta.len());
        }
        oldest_ms = Some(match oldest_ms {
            Some(current) => current.min(record.ts),
            None => record.ts,
        });
    }
    CaptureStats {
        count: records.len() as u64,
        bytes,
        oldest_ms,
    }
}

#[tauri::command]
pub fn capture_store_stats(
    app: tauri::AppHandle,
) -> std::result::Result<CaptureStats, String> {
    use tauri::Manager;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir unavailable: {e}"))?;
    let store = CaptureStore::open(&data_dir).map_err(|e| e.to_string())?;
    let filter = TimelineFilter {
        limit: i64::MAX as u64,
        ..Default::default()
    };
    let records = store.timeline(&filter).map_err(|e| e.to_string())?;
    Ok(stats_from(&records, &data_dir))
}

#[tauri::command]
pub fn capture_delete_all(app: tauri::AppHandle) -> std::result::Result<(), String> {
    use tauri::Manager;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir unavailable: {e}"))?;
    let store = CaptureStore::open(&data_dir).map_err(|e| e.to_string())?;
    store
        .delete_all()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_sum_sizes_and_track_oldest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::write(root.join("a.png"), vec![0u8; 10]).expect("write a");
        fs::write(root.join("b.png"), vec![0u8; 25]).expect("write b");
        let record = |path: &str, ts: u64| CaptureRecord {
            capture_id: path.to_string(),
            ts,
            day: "1970-01-01".to_string(),
            scope: "window".to_string(),
            display_id: "1".to_string(),
            app: None,
            window_title: None,
            path: path.to_string(),
            w_px: 1,
            h_px: 1,
            scale: 1.0,
            sha256: path.to_string(),
            retention: "default".to_string(),
            auto: false,
        };
        let records = vec![record("a.png", 200), record("b.png", 100)];
        let stats = stats_from(&records, root);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.bytes, 35);
        assert_eq!(stats.oldest_ms, Some(100));

        let missing = stats_from(&[record("gone.png", 5)], root);
        assert_eq!(missing.bytes, 0);
        assert_eq!(missing.oldest_ms, Some(5));
    }

    #[test]
    fn stats_empty_is_zeroed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let stats = stats_from(&[], dir.path());
        assert_eq!(stats.count, 0);
        assert_eq!(stats.bytes, 0);
        assert_eq!(stats.oldest_ms, None);
    }

    #[test]
    fn day_string_known_dates() {
        assert_eq!(day_string(0), "1970-01-01");
        assert_eq!(day_string(1_758_211_353_000), "2025-09-18");
        assert_eq!(day_string(86_399_999), "1970-01-01");
        assert_eq!(day_string(86_400_000), "1970-01-02");
    }
}
