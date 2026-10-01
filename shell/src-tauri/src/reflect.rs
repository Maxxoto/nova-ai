//! Nightly reflection (RFC-0011 D7): once a day, yesterday's episodic diary
//! goes to the brain's LLM, which distils durable facts into semantic notes
//! (origin `consolidation`) and queues contradiction proposals in
//! `digest/REVIEW.md` for the user — never auto-applying a supersession.
//! Runs off the interactive path; skipped while offline or unconfigured.

use serde_json::{json, Value};

use crate::brain::BrainLink;
use crate::memory::{MemoryEntry, MemoryStore, MemoryType};
use crate::settings;

/// Entries per reflect RPC — keeps each LLM call inside the ask timeout.
const BATCH: usize = 8;

pub fn spawn(app: tauri::AppHandle, link: BrainLink) {
    std::thread::spawn(move || loop {
        // Let the brain come online first; the first pass waits it out.
        std::thread::sleep(std::time::Duration::from_secs(20));
        if let Err(e) = run_once(&app, &link) {
            eprintln!("ruoxi: reflection skipped: {e}");
        }
        std::thread::sleep(std::time::Duration::from_secs(6 * 3600));
    });
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn today() -> String {
    crate::memory::iso_utc(now_ms())[0..10].to_string()
}

fn run_once(app: &tauri::AppHandle, link: &BrainLink) -> Result<(), String> {
    if settings::load(app).offline {
        return Ok(());
    }
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("data dir: {e}"))?;
    let store = MemoryStore::open(&dir)?;
    let marker = store.reflection_marker_path();
    let last = std::fs::read_to_string(&marker)
        .unwrap_or_default()
        .trim()
        .to_string();
    let today = today();
    if last == today {
        return Ok(());
    }
    // Reflect everything since the last pass (or the last week on a fresh
    // marker), so a missed day is folded in rather than lost.
    let since = if last.len() == 10 {
        format!("{last}T00:00:00Z")
    } else {
        crate::memory::iso_utc(now_ms().saturating_sub(7 * 86_400_000))
    };
    let entries = store.episodic_since(&since, 64)?;
    if entries.is_empty() {
        std::fs::write(&marker, &today).map_err(|e| e.to_string())?;
        return Ok(());
    }
    if !link.is_online() {
        return Ok(()); // retry on the next cycle; the marker is untouched
    }
    let mut written = 0usize;
    for batch in entries.chunks(BATCH) {
        let result = link
            .rpc("session.reflect", json!({ "entries": batch }))
            .map_err(|e| format!("reflect rpc: {e}"))?;
        written += apply_result(&store, &result)?;
    }
    std::fs::write(&marker, &today).map_err(|e| e.to_string())?;
    eprintln!("ruoxi: reflection wrote {written} semantic note(s)");
    Ok(())
}

/// Writes distilled facts as consolidation-origin semantic notes and appends
/// review proposals. Returns the number of facts written.
fn apply_result(store: &MemoryStore, result: &Value) -> Result<usize, String> {
    let mut written = 0usize;
    if let Some(facts) = result.get("facts").and_then(Value::as_array) {
        for fact in facts {
            let (Some(title), Some(body)) = (
                fact.get("title").and_then(Value::as_str),
                fact.get("body").and_then(Value::as_str),
            ) else {
                continue;
            };
            if title.trim().is_empty() || body.trim().is_empty() {
                continue;
            }
            let tags = fact
                .get("tags")
                .and_then(Value::as_array)
                .map(|tags| {
                    tags.iter()
                        .filter_map(|t| t.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let mut entry =
                MemoryEntry::new(MemoryType::Semantic, format!("# {title}\n\n{body}"));
            entry.origin = "consolidation".to_string();
            entry.tags = tags;
            store.write(&entry)?;
            written += 1;
        }
    }
    if let Some(review) = result.get("review").and_then(Value::as_array) {
        let mut lines = String::new();
        for item in review {
            let (Some(id), Some(note)) = (
                item.get("id").and_then(Value::as_str),
                item.get("note").and_then(Value::as_str),
            ) else {
                continue;
            };
            lines.push_str(&format!("- [{}] {note}\n", id));
        }
        if !lines.is_empty() {
            let path = store.review_queue_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut existing = std::fs::read_to_string(&path).unwrap_or_default();
            existing.push_str(&format!("\n## {}\n\n", today()));
            existing.push_str(&lines);
            std::fs::write(&path, existing).map_err(|e| e.to_string())?;
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ulid::Ulid;

    fn temp_store(tag: &str) -> MemoryStore {
        let dir = std::env::temp_dir().join(format!("ruoxi-reflect-{tag}-{}", Ulid::generate()));
        MemoryStore::open(&dir).expect("open")
    }

    #[test]
    fn facts_land_as_consolidation_and_review_queues() {
        let store = temp_store("apply");
        let result = serde_json::json!({
            "facts": [
                {"title": "Prefers short answers", "body": "Answer length: short.", "tags": ["preference"]},
                {"title": "", "body": "titleless — dropped", "tags": []}
            ],
            "review": [
                {"id": "mem_old", "note": "Superseded by the newer office note"}
            ]
        });
        let written = apply_result(&store, &result).expect("apply");
        assert_eq!(written, 1);
        let hits = store.search("short answers", None, 10).expect("search");
        assert!(hits.iter().any(|h| h.kind == "semantic"));
        let review = std::fs::read_to_string(store.review_queue_path()).expect("review file");
        assert!(review.contains("mem_old"));
        assert!(review.contains("Superseded by the newer office note"));
    }

    #[test]
    fn episodic_since_windows_by_date() {
        let store = temp_store("window");
        let mut fresh = MemoryEntry::new(
            MemoryType::Episodic,
            "asked about the reflection window".to_string(),
        );
        fresh.created = "2026-09-30T10:00:00Z".to_string();
        store.write(&fresh).expect("write");
        let mut old = MemoryEntry::new(MemoryType::Episodic, "ancient ask".to_string());
        old.created = "2026-01-01T10:00:00Z".to_string();
        store.write(&old).expect("write");

        let rows = store
            .episodic_since("2026-09-30T00:00:00Z", 10)
            .expect("since");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["id"], json!(fresh.id));
    }
}
