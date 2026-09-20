//! Ruoxi desktop companion shell — M0 scaffold (W1).
//!
//! - Tray-resident; the frameless result panel window (W4) stays hidden until shown.
//! - Accessory activation policy on macOS (no Dock icon) — AC-01.
//! - Single instance lock.
//! - Settings persisted as JSON in the app data dir.
//! - Brain sidecar supervised per RFC-0002 §4.7 (W5 counterpart).

pub mod ask;
pub mod brain;
pub mod capture;
pub mod capture_store;
pub mod memory;
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
pub mod voice;

#[cfg(target_os = "macos")]
pub mod ptt;

mod tray;

use serde_json::Value;

/// Routes upstream JSON-RPC methods issued by the Python brain (Rust-backed
/// tools per RFC-0008): `capture.lookup`, `timeline.query`, …
pub trait RequestRouter: Send + Sync {
    /// Whether this router owns `method`; chains use it to dispatch without
    /// mistaking a domain error for an unknown method.
    fn handles(&self, method: &str) -> bool {
        let _ = method;
        true
    }
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
            settings::resize_onboarding,
            hotkeys::validate_hotkey,
            panel::show_panel,
            panel::hide_panel,
            ask::voice_ask_start,
            ask::voice_ask_stop,
            ask::voice_ask_cancel,
            memory::memory_save_semantic,
            brain::session_ask,
            brain::session_abort,
            models::stt_catalog,
            models::stt_download,
            models::download_cancel,
            models::stt_select,
            models::stt_delete,
            tts::tts_list_voices,
            tts::tts_test_voice,
            tts::tts_save_voice,
            tts::tts_save_engine,
            tts::tts_save_rate,
            tts::tts_kokoro_voices,
            tts::tts_download_kokoro,
            tts::tts_bundle_bytes,
            tts::tts_synthesize,
            tts::tts_speak_text,
            tts::tts_stop,
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
                let voice_handle = app.handle().clone();
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
                                    if panel::is_visible() {
                                        if let Err(e) = ask::begin_ask(&app) {
                                            eprintln!("ruoxi: ask recording: {e}");
                                        }
                                    } else if let Err(e) = ask::start_recording() {
                                        eprintln!("ruoxi: microphone: {e}");
                                    }
                                    tray::show_listening(&app);
                                    let runner = app.clone();
                                    let _ = runner.run_on_main_thread(move || {
                                        if let Some(tray) = app.tray_by_id("main") {
                                            let _ = tray.set_tooltip(Some("Ruoxi — listening"));
                                        }
                                    });
                                } else {
                                    tray::refresh_icon(&app);
                                    let runner = app.clone();
                                    let tooltip_app = app.clone();
                                    let _ = runner.run_on_main_thread(move || {
                                        if let Some(tray) = tooltip_app.tray_by_id("main") {
                                            let _ =
                                                tray.set_tooltip(Some("Ruoxi — brain connected"));
                                        }
                                    });
                                    if ask::is_active() {
                                        if let Err(e) = ask::finish_ask(&app) {
                                            eprintln!("ruoxi: voice ask: {e}");
                                        }
                                    } else if let Some(session) = ask::take_recording() {
                                        let samples = session.stop();
                                        eprintln!(
                                            "ruoxi: captured {} samples ({:.1}s)",
                                            samples.len(),
                                            samples.len() as f64 / voice::TARGET_RATE as f64
                                        );
                                        if !samples.is_empty() {
                                            if let Ok(dir) = std::env::var("RUOXI_S5_RECORD") {
                                                match voice::write_corpus_wav(&dir, &samples) {
                                                    Ok(path) => {
                                                        eprintln!(
                                                            "ruoxi: s5 corpus wrote {}",
                                                            path.display()
                                                        );
                                                    }
                                                    Err(e) => eprintln!(
                                                        "ruoxi: s5 corpus write failed: {e}"
                                                    ),
                                                }
                                            }
                                            let voice_handle = voice_handle.clone();
                                            std::thread::spawn(move || {
                                                run_ptt_transcription(voice_handle, samples);
                                            });
                                        }
                                    }
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

            #[cfg(target_os = "macos")]
            {
                panel::apply_macos_panel_style(app.handle());
                if let Err(e) = panel::spawn_esc_dismiss(app.handle().clone()) {
                    eprintln!("ruoxi: esc dismiss unavailable: {e}");
                }
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
            hotkeys::CaptureIntent::Fullscreen => {
                capture::capture_fullscreen(settings::load(&handle).fullscreen_display_id)
            }
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
    ask::set_current_capture(record);
    if let Err(e) = handle.emit("panel:capture", ask::capture_event(record)) {
        eprintln!("ruoxi: panel:capture emit failed: {e}");
    }
}

#[cfg(target_os = "macos")]
fn run_ptt_transcription(handle: tauri::AppHandle, samples: Vec<f32>) {
    use tauri::Emitter;
    let _ = handle.emit("panel:transcribing", serde_json::json!({}));
    let started = std::time::Instant::now();
    let audio_secs = samples.len() as f64 / voice::TARGET_RATE as f64;
    match voice::transcribe(&handle, &samples) {
        Ok(transcript) if !transcript.is_empty() => {
            eprintln!(
                "ruoxi: stt latency: {}ms for {:.1}s audio",
                started.elapsed().as_millis(),
                audio_secs
            );
            eprintln!("ruoxi: transcript: {transcript}");
            use tauri::Manager;
            let brain = handle.state::<brain::BrainLink>();
            let _ = brain.send(
                "session.ask",
                serde_json::json!({ "transcript": transcript, "capture_ids": [] }),
            );
            panel::show(&handle);
        }
        Ok(_) => eprintln!("ruoxi: no speech detected"),
        Err(e) => eprintln!("ruoxi: transcription: {e}"),
    }
}
