//! Ruoxi desktop companion shell — M0 scaffold (W1).
//!
//! - Tray-resident; the frameless result panel window (W4) stays hidden until shown.
//! - Accessory activation policy on macOS (no Dock icon) — AC-01.
//! - Single instance lock.
//! - Settings persisted as JSON in the app data dir.
//! - Brain sidecar supervised per RFC-0002 §4.7 (W5 counterpart).

pub mod brain;
pub mod capture;
pub mod capture_store;
pub mod displays;
pub mod hotkeys;
pub mod llm;
pub mod models;
pub mod overlay;
pub mod panel;
pub mod permissions;
pub mod settings;
pub mod supervisor;
pub mod timeline;
pub mod tts;

#[cfg(target_os = "macos")]
pub mod ptt;

mod tray;

#[cfg(target_os = "macos")]
fn tray_state_icon(listening: bool) -> tauri::image::Image<'static> {
    let bytes: &[u8] = if listening {
        include_bytes!("../icons/tray/tray-listening-template@2x.png")
    } else {
        include_bytes!("../icons/tray/tray-idle-template@2x.png")
    };
    tauri::image::Image::from_bytes(bytes).expect("decoded tray icon")
}

use serde_json::Value;

/// Routes upstream JSON-RPC methods issued by the Python brain (Rust-backed
/// tools per RFC-0008): `capture.lookup`, `timeline.query`, …
pub trait RequestRouter: Send + Sync {
    fn route(&self, method: &str, params: &Value) -> Result<Value, String>;
}

pub fn run() {
    #[cfg(target_os = "macos")]
    let (intent_tx, intent_rx) = std::sync::mpsc::channel::<hotkeys::CaptureIntent>();
    #[cfg(target_os = "macos")]
    let tray_intents = Some(intent_tx.clone());
    #[cfg(not(target_os = "macos"))]
    let tray_intents = None;
    let brain = brain::BrainLink::new();
    let supervisor_brain = brain.clone();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // M0: there is no window to focus; a second launch is a no-op.
        }))
        .manage(brain)
        .manage(overlay::OverlayState::new());
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(capture_shortcut_plugin(intent_tx));
    builder
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::set_settings,
            settings::show_settings,
            settings::show_onboarding,
            settings::hide_onboarding,
            hotkeys::validate_hotkey,
            panel::show_panel,
            panel::hide_panel,
            brain::session_ask,
            brain::session_abort,
            models::stt_catalog,
            models::stt_download,
            models::stt_select,
            models::stt_delete,
            tts::tts_list_voices,
            tts::tts_test_voice,
            tts::tts_save_voice,
            tts::tts_save_engine,
            tts::tts_kokoro_voices,
            tts::tts_synthesize,
            models::tts_model_catalog,
            models::tts_model_select,
            llm::llm_save_config,
            llm::llm_clear_api_key,
            llm::llm_test,
            permissions::permissions_status,
            permissions::permissions_request,
            permissions::open_privacy_pane,
            capture_store::capture_store_stats,
            capture_store::capture_delete_all,
            timeline::show_timeline,
            timeline::timeline_list,
            timeline::capture_thumbnail,
            displays::list_displays,
            overlay::overlay_cancel,
            overlay::start_region_capture,
            overlay::capture_region_commit
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::install(app.handle(), tray_intents)?;
            tray::refresh_icon(app.handle());

            #[cfg(target_os = "macos")]
            {
                let (tx, rx) = std::sync::mpsc::channel::<bool>();
                // The tap keycode is read once here and the listener runs for
                // the life of the process: a rebind applies on the next app
                // start, not to an already-running event tap.
                let configured = settings::load(app.handle()).ptt_hotkey;
                let keycode = match hotkeys::ptt_keycode(&configured) {
                    Some(code) => {
                        eprintln!("ruoxi: ptt hotkey {configured:?} -> keycode {code}");
                        code
                    }
                    None => {
                        eprintln!(
                            "ruoxi: ptt hotkey {configured:?} cannot be mapped; falling back \
                             to F8 (keycode {})",
                            ptt::DEFAULT_PTT_KEYCODE
                        );
                        ptt::DEFAULT_PTT_KEYCODE
                    }
                };
                let mods = hotkeys::ptt_modifiers(&configured);
                eprintln!("ruoxi: ptt modifiers bitmask {mods:#x}");
                match ptt::spawn_listener(keycode, mods, tx) {
                    Ok(_) => {
                        eprintln!("ruoxi: ptt listener active (keycode {keycode})");
                        let handle = app.handle().clone();
                        std::thread::spawn(move || {
                            while let Ok(pressed) = rx.recv() {
                                eprintln!(
                                    "ruoxi: ptt {}",
                                    if pressed { "pressed" } else { "released" }
                                );
                                let app = handle.clone();
                                if pressed {
                                    let runner = app.clone();
                                    let _ = runner.run_on_main_thread(move || {
                                        if let Some(tray) = app.tray_by_id("main") {
                                            let _ = tray
                                                .set_icon_with_as_template(Some(tray_state_icon(true)), true);
                                            let _ = tray.set_tooltip(Some("Ruoxi — listening"));
                                        }
                                    });
                                } else {
                                    tray::refresh_icon(&app);
                                    let runner = app.clone();
                                    let _ = runner.run_on_main_thread(move || {
                                        if let Some(tray) = app.tray_by_id("main") {
                                            let _ =
                                                tray.set_tooltip(Some("Ruoxi — brain connected"));
                                        }
                                    });
                                }
                            }
                        });
                    }
                    Err(ptt::PttError::PermissionDenied) => {
                        eprintln!(
                            "ruoxi: ptt unavailable — grant Accessibility to the launching app                              (e.g. Terminal) in Privacy & Security, then restart"
                        );
                        if let Some(tray) = app.tray_by_id("main") {
                            let _ = tray.set_tooltip(Some(
                                "Ruoxi — grant Accessibility (Privacy & Security) for push-to-talk",
                            ));
                        }
                    }
                    Err(e) => eprintln!("ruoxi: ptt listener failed: {e:?}"),
                }
            }

            #[cfg(target_os = "macos")]
            {
                let handle = app.handle().clone();
                let rx = intent_rx;
                std::thread::spawn(move || capture_worker(handle, rx));
            }

            let handle = app.handle().clone();
            let brain = supervisor_brain.clone();
            tauri::async_runtime::spawn(async move {
                supervisor::run(handle, brain).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("ruoxi: failed to start nova shell");
}

#[cfg(target_os = "macos")]
fn capture_shortcut_plugin(
    intents: std::sync::mpsc::Sender<hotkeys::CaptureIntent>,
) -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_global_shortcut::ShortcutState;
    tauri_plugin_global_shortcut::Builder::new()
        .with_shortcuts(hotkeys::ALL_ACCELERATORS)
        .expect("register capture shortcuts")
        .with_handler(move |_app, shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(intent) = hotkeys::intent_from_accelerator(&shortcut.to_string()) {
                    eprintln!("ruoxi: hotkey {shortcut} -> {intent:?}");
                    let _ = intents.send(intent);
                }
            }
        })
        .build()
}

#[cfg(target_os = "macos")]
fn capture_worker(
    handle: tauri::AppHandle,
    intents: std::sync::mpsc::Receiver<hotkeys::CaptureIntent>,
) {
    use tauri::Manager;
    let data_dir = match handle.path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("ruoxi: app data dir unavailable: {e}");
            return;
        }
    };
    let store = match capture_store::CaptureStore::open(&data_dir) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("ruoxi: capture store failed to open: {e}");
            return;
        }
    };
    while let Ok(intent) = intents.recv() {
        let result = match intent {
            hotkeys::CaptureIntent::Fullscreen => capture::capture_fullscreen(),
            hotkeys::CaptureIntent::Window => capture::capture_window(std::process::id()),
            hotkeys::CaptureIntent::Region => match overlay::start(&handle) {
                Ok(()) => continue,
                Err(e) => {
                    eprintln!("ruoxi: overlay start failed, using system picker: {e}");
                    capture::capture_region_interactive()
                }
            },
        };
        let message = match result {
            Ok(cap) => match store.insert(&capture::to_new_capture(cap)) {
                Ok((record, created)) => {
                    panel::show(&handle);
                    emit_capture(&handle, &record);
                    format!(
                        "Ruoxi — saved {} ({})",
                        record.capture_id,
                        if created { "new" } else { "duplicate" }
                    )
                }
                Err(e) => {
                    eprintln!("ruoxi: capture store error: {e}");
                    format!("Ruoxi — capture store error: {e}")
                }
            },
            Err(capture::CaptureError::Cancelled) => "Ruoxi — capture cancelled".to_string(),
            Err(e) => {
                eprintln!("ruoxi: capture failed: {e}");
                format!("Ruoxi — capture failed: {e}")
            }
        };
        let app = handle.clone();
        let runner = app.clone();
        let _ = runner.run_on_main_thread(move || {
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_tooltip(Some(&message));
            }
        });
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn emit_capture(handle: &tauri::AppHandle, record: &capture_store::CaptureRecord) {
    use tauri::Emitter;
    if let Err(e) = handle.emit(
        "panel:capture",
        serde_json::json!({ "id": &record.capture_id, "at_ms": record.ts }),
    ) {
        eprintln!("ruoxi: panel:capture emit failed: {e}");
    }
}
