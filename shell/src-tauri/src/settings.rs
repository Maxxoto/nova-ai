//! Settings persistence (M0): JSON file in the app data dir.
//! RFC-0002 §4.2 — tray lifecycle configuration surface.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

fn default_sidecar_command() -> String {
    "python3".to_string()
}

fn default_sidecar_args() -> Vec<String> {
    vec!["-m".to_string(), "app.interfaces.sidecar".to_string()]
}

fn default_ping_interval_secs() -> u64 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub pause_captures: bool,
    #[serde(default = "default_sidecar_command")]
    pub sidecar_command: String,
    #[serde(default = "default_sidecar_args")]
    pub sidecar_args: Vec<String>,
    #[serde(default = "default_ping_interval_secs")]
    pub ping_interval_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            pause_captures: false,
            sidecar_command: default_sidecar_command(),
            sidecar_args: default_sidecar_args(),
            ping_interval_secs: default_ping_interval_secs(),
        }
    }
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir unavailable: {e}"))?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &tauri::AppHandle) -> Settings {
    match settings_path(app) {
        Ok(path) => match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Settings::default(),
        },
        Err(_) => Settings::default(),
    }
}

pub fn save(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create data dir: {e}"))?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| format!("write settings: {e}"))
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    Ok(load(&app))
}

#[tauri::command]
pub fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    save(&app, &settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_json_fills_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.sidecar_command, "python3");
        assert_eq!(
            settings.sidecar_args,
            vec!["-m".to_string(), "app.interfaces.sidecar".to_string()]
        );
        assert_eq!(settings.ping_interval_secs, 5);
        assert!(!settings.launch_at_login);
        assert!(!settings.pause_captures);
    }

    #[test]
    fn settings_roundtrip_preserves_values() {
        let settings = Settings {
            launch_at_login: true,
            pause_captures: true,
            sidecar_command: "uv".to_string(),
            sidecar_args: vec!["run".to_string()],
            ping_interval_secs: 2,
        };
        let text = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(back.sidecar_command, "uv");
        assert_eq!(back.ping_interval_secs, 2);
        assert!(back.launch_at_login && back.pause_captures);
    }
}
