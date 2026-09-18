//! Ruoxi desktop companion shell — M0 scaffold (W1).
//!
//! - Tray-resident, no main window (panel arrives in W4).
//! - Accessory activation policy on macOS (no Dock icon) — AC-01.
//! - Single instance lock.
//! - Settings persisted as JSON in the app data dir.
//! - Brain sidecar supervised per RFC-0002 §4.7 (W5 counterpart).

pub mod capture_store;
pub mod settings;
pub mod supervisor;

mod tray;

use serde_json::Value;

/// Routes upstream JSON-RPC methods issued by the Python brain (Rust-backed
/// tools per RFC-0008): `capture.lookup`, `timeline.query`, …
pub trait RequestRouter: Send + Sync {
    fn route(&self, method: &str, params: &Value) -> Result<Value, String>;
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // M0: there is no window to focus; a second launch is a no-op.
        }))
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::set_settings
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::install(app.handle())?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                supervisor::run(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("ruoxi: failed to start nova shell");
}
