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

/// Privacy-first default: "start with nothing leaving this Mac", so a fresh
/// (or upgraded, offline-field-less) settings file starts offline.
fn default_true() -> bool {
    true
}

fn default_answer_length() -> String {
    "short".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_default_scope() -> String {
    "window".to_string()
}

fn default_ptt_hotkey() -> String {
    "F8".to_string()
}

fn default_stt() -> SttSettings {
    SttSettings {
        model: String::new(),
    }
}

fn default_tts() -> TtsSettings {
    TtsSettings {
        engine: "system".to_string(),
        voice: String::new(),
        model: String::new(),
        rate: default_tts_rate(),
    }
}

/// Speaking rate multiplier (1.0× = the engine's normal pace).
fn default_tts_rate() -> f64 {
    1.0
}

fn default_llm() -> LlmSettings {
    LlmSettings {
        base_url: String::new(),
        model: String::new(),
        vision_model: String::new(),
        api_key_set: false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SttSettings {
    #[serde(default)]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TtsSettings {
    #[serde(default = "default_tts_engine")]
    pub engine: String,
    #[serde(default)]
    pub voice: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_tts_rate")]
    pub rate: f64,
}

fn default_tts_engine() -> String {
    "system".to_string()
}

/// Non-secret LLM config; the API key lives in the macOS Keychain and is
/// injected as an env var when the brain sidecar spawns (RFC-0007 §BrainConfig).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmSettings {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub vision_model: String,
    #[serde(default)]
    pub api_key_set: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub pause_captures: bool,
    #[serde(default = "default_true")]
    pub offline: bool,
    #[serde(default)]
    pub read_aloud: bool,
    #[serde(default = "default_answer_length")]
    pub answer_length: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_default_scope")]
    pub default_scope: String,
    /// Preferred whole-screen display (xcap monitor id); `None` keeps the
    /// original behavior — the display under the cursor.
    #[serde(default)]
    pub fullscreen_display_id: Option<u32>,
    #[serde(default = "default_ptt_hotkey")]
    pub ptt_hotkey: String,
    #[serde(default = "default_sidecar_command")]
    pub sidecar_command: String,
    #[serde(default = "default_sidecar_args")]
    pub sidecar_args: Vec<String>,
    #[serde(default = "default_ping_interval_secs")]
    pub ping_interval_secs: u64,
    #[serde(default = "default_stt")]
    pub stt: SttSettings,
    #[serde(default = "default_tts")]
    pub tts: TtsSettings,
    #[serde(default = "default_llm")]
    pub llm: LlmSettings,
}

impl Settings {
    /// Resets any unrecognized enum-ish string to its default. Callers may
    /// trust `load` output; `set_settings` round-trips whatever the UI sends.
    pub fn normalize(&mut self) {
        if !matches!(self.answer_length.as_str(), "short" | "normal") {
            self.answer_length = default_answer_length();
        }
        if !matches!(self.theme.as_str(), "system" | "dawn" | "night") {
            self.theme = default_theme();
        }
        if !matches!(self.default_scope.as_str(), "window" | "fullscreen") {
            self.default_scope = default_default_scope();
        }
        if !self.tts.rate.is_finite() || !(0.5..=2.0).contains(&self.tts.rate) {
            self.tts.rate = default_tts_rate();
        }
        if crate::hotkeys::ptt_keycode(&self.ptt_hotkey).is_none() {
            self.ptt_hotkey = default_ptt_hotkey();
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            pause_captures: false,
            offline: true,
            read_aloud: false,
            answer_length: default_answer_length(),
            theme: default_theme(),
            default_scope: default_default_scope(),
            fullscreen_display_id: None,
            ptt_hotkey: default_ptt_hotkey(),
            sidecar_command: default_sidecar_command(),
            sidecar_args: default_sidecar_args(),
            ping_interval_secs: default_ping_interval_secs(),
            stt: default_stt(),
            tts: default_tts(),
            llm: default_llm(),
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
    let mut settings = match settings_path(app) {
        Ok(path) => match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Settings::default(),
        },
        Err(_) => Settings::default(),
    };
    settings.normalize();
    settings
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
    save(&app, &settings)?;
    use tauri::Emitter;
    let _ = app.emit("settings:changed", &settings);
    Ok(())
}

pub const SETTINGS_LABEL: &str = "settings";

/// Opens the Settings window, focusing the existing one if it is already up.
/// An ordinary window (decorated, resizable, not always-on-top) — unlike the
/// panel it may take focus (docs/DESIGN.md: settings windows are ordinary
/// windows).
pub fn show(app: &tauri::AppHandle) {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(win) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }

    let built = WebviewWindowBuilder::new(
        app,
        SETTINGS_LABEL,
        WebviewUrl::App("index.html?view=settings".into()),
    )
    .title("Ruòxī — Settings")
    .inner_size(900.0, 700.0)
    .min_inner_size(640.0, 480.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(false)
    .transparent(false)
    .center()
    .build();

    if let Err(e) = built {
        eprintln!("ruoxi: settings window build failed: {e}");
    }
}

#[tauri::command]
pub fn show_settings(app: tauri::AppHandle) {
    show(&app);
}

#[tauri::command]
pub fn hide_onboarding(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(win) = app.get_webview_window(ONBOARDING_LABEL) {
        let _ = win.hide();
    }
}

pub const ONBOARDING_LABEL: &str = "onboarding";

/// Opens the onboarding window, focusing the existing one if it is already up.
/// Same ordinary-window treatment as [`show`].
pub fn show_onboarding_window(app: &tauri::AppHandle) {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(win) = app.get_webview_window(ONBOARDING_LABEL) {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }

    let built = WebviewWindowBuilder::new(
        app,
        ONBOARDING_LABEL,
        WebviewUrl::App("index.html?view=onboarding".into()),
    )
    .title("Ruòxī — Setup")
    .inner_size(720.0, 780.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(false)
    .transparent(false)
    .center()
    .build();

    if let Err(e) = built {
        eprintln!("ruoxi: onboarding window build failed: {e}");
    }
}

#[tauri::command]
pub fn show_onboarding(app: tauri::AppHandle) {
    show_onboarding_window(&app);
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
        assert!(settings.offline);
        assert!(!settings.read_aloud);
        assert_eq!(settings.answer_length, "short");
        assert_eq!(settings.theme, "system");
        assert_eq!(settings.default_scope, "window");
        assert_eq!(settings.fullscreen_display_id, None);
        assert_eq!(settings.ptt_hotkey, "F8");
        assert_eq!(settings.tts.rate, 1.0);
    }

    #[test]
    fn settings_roundtrip_preserves_values() {
        let settings = Settings {
            launch_at_login: true,
            pause_captures: true,
            offline: false,
            read_aloud: true,
            answer_length: "normal".to_string(),
            theme: "night".to_string(),
            default_scope: "fullscreen".to_string(),
            fullscreen_display_id: Some(7),
            ptt_hotkey: "Cmd+Shift+Space".to_string(),
            sidecar_command: "uv".to_string(),
            sidecar_args: vec!["run".to_string()],
            ping_interval_secs: 2,
            stt: SttSettings {
                model: "whisper-base-q5".to_string(),
            },
            tts: TtsSettings {
                engine: "kokoro".to_string(),
                voice: "Tingting".to_string(),
                model: "kokoro-onnx-int8".to_string(),
                rate: 1.2,
            },
            llm: LlmSettings {
                base_url: "https://api.example.com/v1".to_string(),
                model: "gpt-test".to_string(),
                vision_model: "vision-test".to_string(),
                api_key_set: true,
            },
        };
        let text = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(back.sidecar_command, "uv");
        assert_eq!(back.ping_interval_secs, 2);
        assert!(back.launch_at_login && back.pause_captures);
        assert!(!back.offline);
        assert!(back.read_aloud);
        assert_eq!(back.answer_length, "normal");
        assert_eq!(back.theme, "night");
        assert_eq!(back.default_scope, "fullscreen");
        assert_eq!(back.fullscreen_display_id, Some(7));
        assert_eq!(back.ptt_hotkey, "Cmd+Shift+Space");
        assert_eq!(back.stt.model, "whisper-base-q5");
        assert_eq!(back.tts.voice, "Tingting");
        assert_eq!(back.tts.engine, "kokoro");
        assert_eq!(back.tts.model, "kokoro-onnx-int8");
        assert_eq!(back.tts.rate, 1.2);
        assert_eq!(back.llm.base_url, "https://api.example.com/v1");
        assert_eq!(back.llm.model, "gpt-test");
        assert!(back.llm.api_key_set);
    }

    #[test]
    fn normalize_resets_invalid_enums_and_keeps_valid() {
        let mut invalid = Settings {
            answer_length: "verbose".to_string(),
            theme: "noir".to_string(),
            default_scope: "display".to_string(),
            tts: TtsSettings {
                rate: 9.0,
                ..Settings::default().tts
            },
            ..Settings::default()
        };
        invalid.normalize();
        assert_eq!(invalid.answer_length, "short");
        assert_eq!(invalid.theme, "system");
        assert_eq!(invalid.default_scope, "window");
        assert_eq!(invalid.tts.rate, 1.0);

        let mut valid = Settings {
            answer_length: "normal".to_string(),
            theme: "dawn".to_string(),
            default_scope: "fullscreen".to_string(),
            tts: TtsSettings {
                rate: 1.4,
                ..Settings::default().tts
            },
            ..Settings::default()
        };
        valid.normalize();
        assert_eq!(valid.answer_length, "normal");
        assert_eq!(valid.theme, "dawn");
        assert_eq!(valid.default_scope, "fullscreen");
        assert_eq!(valid.tts.rate, 1.4);
    }

    #[test]
    fn normalize_falls_back_for_unmappable_ptt_hotkey() {
        let mut unmappable = Settings {
            ptt_hotkey: "F13".to_string(),
            ..Settings::default()
        };
        unmappable.normalize();
        assert_eq!(unmappable.ptt_hotkey, "F8");

        let mut blank = Settings {
            ptt_hotkey: String::new(),
            ..Settings::default()
        };
        blank.normalize();
        assert_eq!(blank.ptt_hotkey, "F8");

        let mut valid = Settings {
            ptt_hotkey: "Cmd+Shift+Space".to_string(),
            ..Settings::default()
        };
        valid.normalize();
        assert_eq!(valid.ptt_hotkey, "Cmd+Shift+Space");
    }
}
