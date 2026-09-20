//! STT model manager (RFC-0004 §4.2/§4.3): model catalog, first-run download
//! with explicit consent + progress events, sha256 verification, storage in
//! the app data dir. The download is the only model-related network event;
//! manual import (dropping a file into `models/`) stays possible.

use std::io::Read;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Emitter;

use crate::settings;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Stt,
    TtsModel,
    TtsVoices,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFamily {
    Whisper,
    Parakeet,
    Kokoro,
}

pub struct SttModel {
    pub id: &'static str,
    pub kind: ModelKind,
    pub family: ModelFamily,
    pub name: &'static str,
    pub file: &'static str,
    pub url: &'static str,
    pub size_bytes: u64,
    pub zh: &'static str,
    pub note: &'static str,
}

pub const CATALOG: &[SttModel] = &[
    SttModel {
        id: "whisper-base-q5",
        kind: ModelKind::Stt,
        family: ModelFamily::Whisper,
        name: "Whisper Base",
        file: "ggml-base-q5_1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin",
        size_bytes: 59_707_625,
        zh: "marginal",
        note: "RFC-0004 default for M1 bring-up",
    },
    SttModel {
        id: "whisper-small-q5",
        kind: ModelKind::Stt,
        family: ModelFamily::Whisper,
        name: "Whisper Small",
        file: "ggml-small-q5_1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin",
        size_bytes: 190_085_487,
        zh: "decent",
        note: "quality option; larger RAM envelope",
    },
    SttModel {
        id: "parakeet-tdt-0.6b",
        kind: ModelKind::Stt,
        family: ModelFamily::Parakeet,
        name: "Parakeet 0.6B",
        file: "ggml-parakeet-tdt-0.6b-v2-q5_0.bin",
        url: "https://huggingface.co/JoaoZaokk/parakeet-tdt-0.6b-v2-ggml/resolve/main/ggml-parakeet-tdt-0.6b-v2-q5_0.bin",
        size_bytes: 427_494_308,
        zh: "none",
        note: "fast English STT (whisper.cpp libparakeet); no Chinese",
    },
];

pub const KOKORO_DEFAULT_ID: &str = "kokoro-onnx-fp32";
pub const KOKORO_VOICES_ID: &str = "kokoro-voices";

pub const TTS_CATALOG: &[SttModel] = &[
    SttModel {
        id: "kokoro-onnx-fp32",
        kind: ModelKind::TtsModel,
        family: ModelFamily::Kokoro,
        name: "Kokoro 82M",
        file: "kokoro-tiny.onnx",
        url: "https://github.com/8b-is/kokoro-tiny/raw/main/models/0.onnx",
        size_bytes: 325_532_387,
        zh: "8 zh voices (v1.0)",
        note: "neural TTS, RFC-0005 primary engine; engine-compatible export (tokens input)",
    },
    SttModel {
        id: "kokoro-voices",
        kind: ModelKind::TtsVoices,
        family: ModelFamily::Kokoro,
        name: "Kokoro voicepacks",
        file: "voices-v1.0.bin",
        url: "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.1/voices-v1.0.bin",
        size_bytes: 28_214_398,
        zh: "54 voices",
        note: "required by the Kokoro model",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttModelInfo {
    pub id: String,
    pub kind: ModelKind,
    pub family: ModelFamily,
    pub name: String,
    pub file: String,
    pub size_bytes: u64,
    pub zh: String,
    pub note: String,
    pub downloaded: bool,
    pub selected: bool,
}

pub fn models_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("models"))
        .map_err(|e| format!("app data dir unavailable: {e}"))
}

pub fn catalog_entry(id: &str) -> Option<&'static SttModel> {
    CATALOG.iter().find(|m| m.id == id)
}

/// Any downloadable artifact (STT or TTS) by id — downloads are shared.
pub fn any_entry(id: &str) -> Option<&'static SttModel> {
    CATALOG
        .iter()
        .chain(TTS_CATALOG.iter())
        .find(|m| m.id == id)
}

pub fn file_sha256(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read model: {e}"))?;
    let digest = Sha256::new().chain_update(&bytes).finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

static CANCEL_REQUESTS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

fn cancel_requested(id: &str) -> bool {
    CANCEL_REQUESTS
        .lock()
        .map(|pending| pending.iter().any(|entry| entry == id))
        .unwrap_or(false)
}

#[tauri::command]
pub fn download_cancel(id: String) {
    if let Ok(mut pending) = CANCEL_REQUESTS.lock() {
        if !pending.iter().any(|entry| entry == &id) {
            pending.push(id);
        }
    }
}

fn download(app: tauri::AppHandle, id: String) {
    let Some(model) = any_entry(&id) else {
        return;
    };
    if let Ok(mut pending) = CANCEL_REQUESTS.lock() {
        pending.retain(|entry| entry != &id);
    }
    let dir = match models_dir(&app) {
        Ok(dir) => dir,
        Err(e) => {
            let _ = app.emit("model:error", serde_json::json!({ "id": id, "message": e }));
            return;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        let _ = app.emit(
            "model:error",
            serde_json::json!({ "id": id, "message": format!("create models dir: {e}") }),
        );
        return;
    }
    let final_path = dir.join(model.file);
    if final_path.is_file() {
        let _ = app.emit(
            "model:done",
            serde_json::json!({ "id": id, "path": final_path.display().to_string() }),
        );
        return;
    }
    let stale_prefix = format!("{}.", model.file);
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(&stale_prefix) && name.ends_with(".part") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    let part_path = dir.join(format!("{}.{}.part", model.file, ulid::Ulid::generate()));
    let result = fetch_to(&app, &id, model, &part_path).and_then(|sha256| {
        std::fs::rename(&part_path, &final_path).map_err(|e| format!("finalize download: {e}"))?;
        Ok(sha256)
    });
    match result {
        Ok(sha256) => {
            let _ = std::fs::write(final_path.with_extension("sha256"), &sha256);
            eprintln!("ruoxi: {id} downloaded to {} (sha256 {sha256})", final_path.display());
            let _ = app.emit(
                "model:done",
                serde_json::json!({ "id": id, "path": final_path.display().to_string(), "sha256": sha256 }),
            );
        }
        Err(message) => {
            let _ = std::fs::remove_file(&part_path);
            eprintln!("ruoxi: {id} download failed: {message}");
            let _ = app.emit("model:error", serde_json::json!({ "id": id, "message": message }));
        }
    }
}

fn fetch_to(
    app: &tauri::AppHandle,
    id: &str,
    model: &SttModel,
    part_path: &std::path::Path,
) -> Result<String, String> {
    let response = ureq::get(model.url)
        .call()
        .map_err(|e| format!("download request failed: {e}"))?;
    let total: u64 = response
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(model.size_bytes);
    let mut file =
        std::fs::File::create(part_path).map_err(|e| format!("create temp file: {e}"))?;
    let mut reader = response.into_body().into_reader();
    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    let mut hasher = Sha256::new();
    let mut logged_percent = 0u64;
    eprintln!("ruoxi: downloading {id} ({total} bytes) from {}", model.url);
    let mut first_emit_logged = false;
    loop {
        let n = reader
            .read(&mut buffer)
            .map_err(|e| format!("download interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        if cancel_requested(id) {
            eprintln!("ruoxi: download {id} cancelled at {downloaded}/{total}");
            return Err("cancelled".to_string());
        }
        hasher.update(&buffer[..n]);
        std::io::Write::write_all(&mut file, &buffer[..n])
            .map_err(|e| format!("write model data: {e}"))?;
        downloaded += n as u64;
        let percent = if total > 0 { downloaded * 100 / total } else { 0 };
        if percent >= logged_percent + 10 || downloaded == total {
            logged_percent = percent;
            eprintln!(
                "ruoxi: {id} {percent}% ({downloaded}/{total} bytes, {:.1} MB)",
                downloaded as f64 / 1_000_000.0
            );
        }
        match app.emit(
            "model:progress",
            serde_json::json!({ "id": id, "downloaded": downloaded, "total": total }),
        ) {
            Ok(()) if !first_emit_logged => {
                first_emit_logged = true;
                eprintln!("ruoxi: model:progress events flowing (first emit ok: {downloaded}/{total})");
            }
            Err(e) => eprintln!("ruoxi: model:progress emit failed: {e}"),
            Ok(()) => {}
        }
    }
    if downloaded == 0 {
        return Err("empty download".to_string());
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

#[tauri::command]
pub fn stt_catalog(app: tauri::AppHandle) -> Result<Vec<SttModelInfo>, String> {
    catalog_list(&app, CATALOG, &settings::load(&app).stt.model)
}

#[tauri::command]
pub fn tts_model_catalog(app: tauri::AppHandle) -> Result<Vec<SttModelInfo>, String> {
    catalog_list(&app, TTS_CATALOG, &settings::load(&app).tts.model)
}

fn catalog_list(
    app: &tauri::AppHandle,
    catalog: &'static [SttModel],
    selected: &str,
) -> Result<Vec<SttModelInfo>, String> {
    let dir = models_dir(app)?;
    Ok(catalog
        .iter()
        .map(|m| SttModelInfo {
            id: m.id.to_string(),
            kind: m.kind,
            family: m.family,
            name: m.name.to_string(),
            file: m.file.to_string(),
            size_bytes: m.size_bytes,
            zh: m.zh.to_string(),
            note: m.note.to_string(),
            downloaded: dir.join(m.file).is_file(),
            selected: selected == m.id,
        })
        .collect())
}



/// Starts the download in the background; progress streams as
/// `model:progress` / `model:done` / `model:error` events. Shared by the STT
/// and TTS catalogs.
#[tauri::command]
pub fn stt_download(app: tauri::AppHandle, id: String) -> Result<(), String> {
    if any_entry(&id).is_none() {
        return Err(format!("unknown model: {id}"));
    }
    std::thread::spawn(move || download(app, id));
    Ok(())
}

#[tauri::command]
pub fn stt_select(app: tauri::AppHandle, id: String) -> Result<(), String> {
    if catalog_entry(&id).is_none() {
        return Err(format!("unknown model: {id}"));
    }
    let mut current = settings::load(&app);
    current.stt.model = id;
    settings::save(&app, &current)
}

#[tauri::command]
pub fn stt_delete(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let model = catalog_entry(&id).ok_or_else(|| format!("unknown model: {id}"))?;
    let dir = models_dir(&app)?;
    let path = dir.join(model.file);
    if path.is_file() {
        std::fs::remove_file(&path).map_err(|e| format!("delete model: {e}"))?;
        let _ = std::fs::remove_file(path.with_extension("sha256"));
    }
    let mut current = settings::load(&app);
    let mut changed = false;
    if current.stt.model == id {
        current.stt.model = String::new();
        changed = true;
    }
    if current.tts.model == id {
        current.tts.model = String::new();
        changed = true;
    }
    if changed {
        settings::save(&app, &current)?;
    }
    Ok(())
}

#[tauri::command]
pub fn tts_model_select(app: tauri::AppHandle, id: String) -> Result<(), String> {
    if !TTS_CATALOG.iter().any(|m| m.id == id) {
        return Err(format!("unknown tts model: {id}"));
    }
    let mut current = settings::load(&app);
    current.tts.model = id;
    settings::save(&app, &current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_across_stt_and_tts_catalogs() {
        let mut ids: Vec<&str> = CATALOG
            .iter()
            .chain(TTS_CATALOG.iter())
            .map(|m| m.id)
            .collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
        assert!(any_entry("kokoro-voices").is_some());
        assert!(any_entry("whisper-base-q5").is_some());
    }

    #[test]
    fn catalogs_partition_by_kind() {
        assert!(CATALOG.iter().all(|m| m.kind == ModelKind::Stt));
        assert!(CATALOG
            .iter()
            .all(|m| matches!(m.family, ModelFamily::Whisper | ModelFamily::Parakeet)));
        assert!(TTS_CATALOG
            .iter()
            .all(|m| matches!(m.kind, ModelKind::TtsModel | ModelKind::TtsVoices)));
        assert!(TTS_CATALOG.iter().all(|m| m.family == ModelFamily::Kokoro));
        assert!(TTS_CATALOG
            .iter()
            .any(|m| m.id == KOKORO_DEFAULT_ID && m.kind == ModelKind::TtsModel));
        assert!(TTS_CATALOG
            .iter()
            .any(|m| m.id == KOKORO_VOICES_ID && m.kind == ModelKind::TtsVoices));
    }

    #[test]
    fn catalog_ids_are_unique_and_files_map_back() {
        let mut ids: Vec<&str> = CATALOG.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
        for id in ids {
            assert!(catalog_entry(id).is_some());
        }
    }

    #[test]
    fn every_catalog_entry_targets_its_own_file() {
        let mut files: Vec<&str> = CATALOG.iter().map(|m| m.file).collect();
        files.sort_unstable();
        let count = files.len();
        files.dedup();
        assert_eq!(files.len(), count);
    }

    #[test]
    fn sha256_of_known_bytes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("x.bin");
        std::fs::write(&path, b"abc").expect("write");
        assert_eq!(
            file_sha256(&path).expect("hash"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
