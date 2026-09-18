//! Capture store v0 tests (W3): round-trip, dedupe, timeline filters,
//! retention deletes, and sidecar-based index rebuild.

use std::fs;
use std::path::Path;

use nova_shell::capture_store::{
    CaptureStore, NewCapture, Scope, StoreError, TimelineFilter,
};

fn temp_root() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn capture(ts_ms: u64, image: Vec<u8>, app: Option<&str>) -> NewCapture {
    NewCapture {
        scope: Scope::Region,
        ts_ms,
        display_id: "display-1".to_string(),
        app: app.map(str::to_string),
        window_title: Some("lecture-7.pdf".to_string()),
        w_px: 1280,
        h_px: 960,
        scale: 2.0,
        image,
        ext: "png".to_string(),
        auto: false,
    }
}

#[test]
fn insert_then_lookup_roundtrip() {
    let root = temp_root();
    let store = CaptureStore::open(root.path()).expect("open");

    let (record, created) = store
        .insert(&capture(1_758_211_353_000, vec![1, 2, 3, 4], Some("Preview")))
        .expect("insert");
    assert!(created);
    assert!(record.capture_id.starts_with("cap_"));
    assert_eq!(record.day, "2025-09-18");
    assert_eq!(record.scope, "region");
    assert!(record.path.starts_with("captures/2025/09/18/cap_"));
    assert!(record.path.ends_with(".png"));
    assert!(root.path().join(&record.path).is_file());
    assert!(root
        .path()
        .join(&record.path)
        .with_extension("json")
        .is_file());

    let found = store.lookup(&record.capture_id).expect("lookup");
    assert_eq!(found.as_ref().map(|r| r.sha256.clone()), Some(record.sha256));
}

#[test]
fn identical_bytes_dedupe_to_same_id() {
    let root = temp_root();
    let store = CaptureStore::open(root.path()).expect("open");

    let bytes = vec![9, 9, 9, 9];
    let (first, created_first) = store
        .insert(&capture(1_000, bytes.clone(), Some("Firefox")))
        .expect("insert 1");
    let (second, created_second) = store
        .insert(&capture(2_000, bytes, Some("Firefox")))
        .expect("insert 2");

    assert!(created_first);
    assert!(!created_second);
    assert_eq!(first.capture_id, second.capture_id);
}

#[test]
fn timeline_filters_day_app_scope() {
    let root = temp_root();
    let store = CaptureStore::open(root.path()).expect("open");

    store
        .insert(&capture(1_758_211_353_000, vec![1], Some("Preview")))
        .expect("a");
    let mut window_capture = capture(1_758_211_354_000, vec![2], Some("Firefox"));
    window_capture.scope = Scope::Window;
    store.insert(&window_capture).expect("b");
    store
        .insert(&capture(1_758_121_353_000, vec![3], Some("Preview")))
        .expect("c");

    let all = store
        .timeline(&TimelineFilter {
            limit: 10,
            ..Default::default()
        })
        .expect("all");
    assert_eq!(all.len(), 3);

    let day = store
        .timeline(&TimelineFilter {
            day: Some("2025-09-18".to_string()),
            limit: 10,
            ..Default::default()
        })
        .expect("day");
    assert_eq!(day.len(), 2);

    let app = store
        .timeline(&TimelineFilter {
            app: Some("Firefox".to_string()),
            limit: 10,
            ..Default::default()
        })
        .expect("app");
    assert_eq!(app.len(), 1);
    assert_eq!(app[0].app.as_deref(), Some("Firefox"));

    let scope = store
        .timeline(&TimelineFilter {
            scope: Some(Scope::Window),
            limit: 10,
            ..Default::default()
        })
        .expect("scope");
    assert_eq!(scope.len(), 1);
    assert_eq!(scope[0].scope, "window");

    let paged = store
        .timeline(&TimelineFilter {
            limit: 2,
            offset: 2,
            ..Default::default()
        })
        .expect("paged");
    assert_eq!(paged.len(), 1);
}

#[test]
fn delete_day_removes_rows_and_files() {
    let root = temp_root();
    let store = CaptureStore::open(root.path()).expect("open");

    let (keep, _) = store
        .insert(&capture(1_758_211_353_000, vec![1], Some("Preview")))
        .expect("keep");
    let (drop, _) = store
        .insert(&capture(1_758_121_353_000, vec![2], Some("Preview")))
        .expect("drop");
    assert_ne!(keep.day, drop.day);

    let removed = store.delete_day(&drop.day).expect("delete day");
    assert_eq!(removed, 1);
    assert!(store.lookup(&drop.capture_id).expect("gone").is_none());
    assert!(!root.path().join(&drop.path).is_file());
    assert!(store.lookup(&keep.capture_id).expect("kept").is_some());
    assert!(root.path().join(&keep.path).is_file());
}

#[test]
fn delete_all_wipes_everything() {
    let root = temp_root();
    let store = CaptureStore::open(root.path()).expect("open");

    store
        .insert(&capture(1_000, vec![1], None))
        .expect("a");
    store
        .insert(&capture(2_000, vec![2], None))
        .expect("b");

    assert_eq!(store.delete_all().expect("delete all"), 2);
    assert!(store
        .timeline(&TimelineFilter {
            limit: 10,
            ..Default::default()
        })
        .expect("empty")
        .is_empty());
}

#[test]
fn index_rebuilds_from_sidecars() {
    let root = temp_root();
    let path = root.path().to_path_buf();
    {
        let store = CaptureStore::open(&path).expect("open");
        store
            .insert(&capture(1_758_211_353_000, vec![7, 7, 7], Some("Preview")))
            .expect("insert");
    }
    fs::remove_file(path.join("capture_index.sqlite")).expect("index removed");

    let rebuilt = CaptureStore::rebuild_index(&path).expect("rebuild");
    let records = rebuilt
        .timeline(&TimelineFilter {
            limit: 10,
            ..Default::default()
        })
        .expect("timeline");
    assert_eq!(records.len(), 1);
    assert!(records[0].capture_id.starts_with("cap_"));
    assert_eq!(records[0].app.as_deref(), Some("Preview"));
}

#[test]
fn open_error_on_unwritable_root_is_typed() {
    let result = CaptureStore::open(Path::new("/proc/definitely-not-writable"));
    assert!(matches!(result, Err(StoreError::Io(_) | StoreError::Db(_))));
}
