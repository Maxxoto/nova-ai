//! Result panel window (W4) — frameless, always-on-top, hidden until shown.
//! Spec: RFC-0002 §4.4 + docs/DESIGN.md `panel-shell`.
//!
//! Keyboard dismissal (AC-06) requires the panel to hold focus while it is
//! visible; true non-activation waits on the S4 spike verdict (documented
//! fallback per plans/m0-build-plan.md W4).

use tauri::{AppHandle, Manager};

pub const PANEL_LABEL: &str = "panel";

pub fn show(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(PANEL_LABEL) {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

pub fn hide(app: &AppHandle) {
    crate::tts::stop_speaking();
    if let Some(win) = app.get_webview_window(PANEL_LABEL) {
        let _ = win.hide();
    }
}

#[tauri::command]
pub fn show_panel(app: AppHandle) {
    show(&app);
}

#[tauri::command]
pub fn hide_panel(app: AppHandle) {
    hide(&app);
}
