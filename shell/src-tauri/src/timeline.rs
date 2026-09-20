//! Timeline window + read-only timeline backend (RFC-0003 §4.3/§4.7).
//!
//! Backs the `?view=timeline` screen: a row listing over the existing
//! [`CaptureStore`](crate::capture_store::CaptureStore) timeline API, plus
//! on-demand, aspect-preserving PNG thumbnails returned as data URLs. The
//! window is an ordinary window (decorated, resizable, not always-on-top),
//! matching [`crate::settings::show`].

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::capture_store::{CaptureRecord, CaptureStore, Scope, TimelineFilter};

pub const TIMELINE_LABEL: &str = "timeline";

/// Rows returned when the UI does not ask for a specific page size.
pub const DEFAULT_LIMIT: u32 = 500;

/// Upper bound on the thumbnail's longest edge (defensive; keeps IPC payloads
/// from ballooning if the UI asks for a full-size render).
pub const MAX_THUMB_PX: u32 = 1024;

/// Optional filters accepted from the timeline UI. Every field is optional;
/// absent/empty values mean "do not filter".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineQuery {
    pub day: Option<String>,
    pub app: Option<String>,
    pub scope: Option<String>,
    pub limit: Option<u32>,
}

/// One timeline row: the store record trimmed to the fields the UI renders,
/// plus the on-disk image size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineRow {
    pub capture_id: String,
    pub ts_ms: u64,
    pub scope: String,
    pub app: Option<String>,
    pub window_title: Option<String>,
    pub w_px: i64,
    pub h_px: i64,
    pub scale: f64,
    pub bytes: u64,
}

fn scope_from_wire(raw: &str) -> Option<Scope> {
    match raw {
        "region" => Some(Scope::Region),
        "window" => Some(Scope::Window),
        "fullscreen" => Some(Scope::Fullscreen),
        _ => None,
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.is_empty())
}

/// Converts the wire query into a store filter. Pure so the default/cap and
/// scope-mapping rules are unit-testable.
pub fn to_filter(query: Option<TimelineQuery>) -> TimelineFilter {
    let query = query.unwrap_or_default();
    TimelineFilter {
        day: non_empty(query.day),
        app: non_empty(query.app),
        scope: query.scope.as_deref().and_then(scope_from_wire),
        limit: query
            .limit
            .map(|n| u64::from(n.max(1)))
            .unwrap_or(u64::from(DEFAULT_LIMIT)),
        offset: 0,
    }
}

/// Maps a store record to a row; `bytes` is the image file size, or 0 when the
/// file is missing (e.g. deleted out from under the index).
pub fn row_from_record(record: &CaptureRecord, root: &Path) -> TimelineRow {
    let bytes = fs::metadata(root.join(&record.path))
        .map(|meta| meta.len())
        .unwrap_or(0);
    TimelineRow {
        capture_id: record.capture_id.clone(),
        ts_ms: record.ts,
        scope: record.scope.clone(),
        app: record.app.clone(),
        window_title: record.window_title.clone(),
        w_px: record.w_px,
        h_px: record.h_px,
        scale: record.scale,
        bytes,
    }
}

/// Maps + sorts newest first (the store already orders `ts DESC`; sorting here
/// keeps the contract local to the mapping).
pub fn rows_from(records: &[CaptureRecord], root: &Path) -> Vec<TimelineRow> {
    let mut rows: Vec<TimelineRow> = records
        .iter()
        .map(|record| row_from_record(record, root))
        .collect();
    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    rows
}

/// Caps a requested thumbnail edge to `[1, MAX_THUMB_PX]`.
pub fn clamp_thumb_px(max_px: u32) -> u32 {
    max_px.clamp(1, MAX_THUMB_PX)
}

/// Decodes a capture PNG, downsizes so its longest edge is at most `max_px`
/// (never upscaling), and re-encodes as PNG.
fn resized_png(bytes: &[u8], max_px: u32) -> Result<Vec<u8>, String> {
    use xcap::image::{imageops::FilterType, GenericImageView};

    let image = xcap::image::load_from_memory(bytes).map_err(|e| format!("decode image: {e}"))?;
    let (width, height) = image.dimensions();
    let longest = width.max(height);
    let image = if longest > max_px {
        let scale = f64::from(max_px) / f64::from(longest);
        let w = ((f64::from(width) * scale).round() as u32).max(1);
        let h = ((f64::from(height) * scale).round() as u32).max(1);
        image.resize_exact(w, h, FilterType::Lanczos3)
    } else {
        image
    };
    let rgba = image.to_rgba8();
    crate::capture::encode_png(rgba.as_raw(), rgba.width(), rgba.height())
        .map_err(|e| e.to_string())
}

const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Minimal RFC 4648 base64 (no new crates). Padding is always emitted.
pub fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
        let b2 = u32::from(*chunk.get(2).unwrap_or(&0));
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64_ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64_ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Opens the timeline window, focusing the existing one if it is already up.
pub fn show(app: &tauri::AppHandle) {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(win) = app.get_webview_window(TIMELINE_LABEL) {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }

    let built = WebviewWindowBuilder::new(
        app,
        TIMELINE_LABEL,
        WebviewUrl::App("index.html?view=timeline".into()),
    )
    .title("Ruòxī — Timeline")
    .inner_size(980.0, 760.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(false)
    .transparent(false)
    .center()
    .build();

    if let Err(e) = built {
        eprintln!("ruoxi: timeline window build failed: {e}");
    }
}

#[tauri::command]
pub fn show_timeline(app: tauri::AppHandle) {
    show(&app);
}

fn data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map_err(|e| format!("app data dir unavailable: {e}"))
}

#[tauri::command]
pub fn timeline_list(
    app: tauri::AppHandle,
    filter: Option<TimelineQuery>,
) -> Result<Vec<TimelineRow>, String> {
    let root = data_dir(&app)?;
    let store = CaptureStore::open(&root).map_err(|e| e.to_string())?;
    let records = store
        .timeline(&to_filter(filter))
        .map_err(|e| e.to_string())?;
    Ok(rows_from(&records, &root))
}

#[tauri::command]
pub fn capture_thumbnail(
    app: tauri::AppHandle,
    capture_id: String,
    max_px: u32,
) -> Result<String, String> {
    let root = data_dir(&app)?;
    let store = CaptureStore::open(&root).map_err(|e| e.to_string())?;
    let record = store
        .lookup(&capture_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "capture not found".to_string())?;
    let bytes = fs::read(root.join(&record.path)).map_err(|e| format!("read capture: {e}"))?;
    let png = resized_png(&bytes, clamp_thumb_px(max_px))?;
    Ok(format!("data:image/png;base64,{}", base64_encode(&png)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(capture_id: &str, ts: u64, path: &str) -> CaptureRecord {
        CaptureRecord {
            capture_id: capture_id.to_string(),
            ts,
            day: "1970-01-01".to_string(),
            scope: "window".to_string(),
            display_id: "1".to_string(),
            app: Some("Safari".to_string()),
            window_title: Some("Docs".to_string()),
            path: path.to_string(),
            w_px: 800,
            h_px: 600,
            scale: 2.0,
            sha256: capture_id.to_string(),
            retention: "default".to_string(),
            auto: false,
        }
    }

    #[test]
    fn base64_rfc4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64_encode(&[0x00, 0x01, 0x02]), "AAEC");
        assert_eq!(base64_encode(&[0xFB, 0xFF, 0xBF]), "+/+/");
    }

    #[test]
    fn default_filter_is_capped_and_empty() {
        let filter = to_filter(None);
        assert_eq!(filter.day, None);
        assert_eq!(filter.app, None);
        assert_eq!(filter.scope, None);
        assert_eq!(filter.limit, u64::from(DEFAULT_LIMIT));
        assert_eq!(filter.offset, 0);
    }

    #[test]
    fn query_converts_to_filter() {
        let filter = to_filter(Some(TimelineQuery {
            day: Some("2025-09-18".to_string()),
            app: Some("Safari".to_string()),
            scope: Some("fullscreen".to_string()),
            limit: Some(25),
        }));
        assert_eq!(filter.day.as_deref(), Some("2025-09-18"));
        assert_eq!(filter.app.as_deref(), Some("Safari"));
        assert_eq!(filter.scope, Some(Scope::Fullscreen));
        assert_eq!(filter.limit, 25);
    }

    #[test]
    fn empty_strings_and_unknown_scope_are_ignored() {
        let filter = to_filter(Some(TimelineQuery {
            day: Some(String::new()),
            app: None,
            scope: Some("bogus".to_string()),
            limit: Some(0),
        }));
        assert_eq!(filter.day, None);
        assert_eq!(filter.scope, None);
        assert_eq!(filter.limit, 1, "a zero limit is floored to 1");

        assert_eq!(
            to_filter(Some(TimelineQuery {
                scope: Some("region".to_string()),
                ..Default::default()
            }))
            .scope,
            Some(Scope::Region)
        );
    }

    #[test]
    fn rows_map_fields_sort_newest_first_and_size_missing_as_zero() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::write(root.join("a.png"), vec![0u8; 17]).expect("write a");

        let records = vec![record("old", 100, "a.png"), record("new", 200, "gone.png")];
        let rows = rows_from(&records, root);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].capture_id, "new");
        assert_eq!(rows[0].ts_ms, 200);
        assert_eq!(rows[0].bytes, 0, "missing file maps to zero bytes");
        assert_eq!(rows[1].capture_id, "old");
        assert_eq!(rows[1].bytes, 17);
        assert_eq!(rows[1].scope, "window");
        assert_eq!(rows[1].app.as_deref(), Some("Safari"));
        assert_eq!(rows[1].w_px, 800);
        assert_eq!(rows[1].h_px, 600);
        assert_eq!(rows[1].scale, 2.0);
    }

    #[test]
    fn thumb_px_is_clamped() {
        assert_eq!(clamp_thumb_px(0), 1);
        assert_eq!(clamp_thumb_px(256), 256);
        assert_eq!(clamp_thumb_px(u32::MAX), MAX_THUMB_PX);
    }

    #[test]
    fn resize_caps_longest_edge_preserving_aspect() {
        let rgba = vec![10u8; 8 * 4 * 4];
        let png = crate::capture::encode_png(&rgba, 8, 4).expect("encode");
        let out = resized_png(&png, 4).expect("resize");
        assert_eq!(crate::capture::png_dimensions(&out), Some((4, 2)));
    }

    #[test]
    fn resize_does_not_upscale() {
        let rgba = vec![0u8; 2 * 2 * 4];
        let png = crate::capture::encode_png(&rgba, 2, 2).expect("encode");
        let out = resized_png(&png, 64).expect("resize");
        assert_eq!(crate::capture::png_dimensions(&out), Some((2, 2)));
    }
}
