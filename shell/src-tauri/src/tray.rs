//! Tray icon + menu (M0). RFC-0002 §4.2 — accessory-resident, no windows.
//!
//! The tray is the honest state surface (docs/DESIGN.md `components.tray-menu`):
//! the offline kill-switch and capture pause live here as checkboxes, and the
//! icon mirrors the persisted state so the tray alone tells the truth.

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;

use crate::{hotkeys, settings};

/// The tray icon variant. The v4 design carries state by shape alone — ready
/// (capture frame), listening (centre dot), captures paused (slash) — and
/// confirms offline and the remaining states in words (menu checkbox, tooltip).
/// Paused/idle derive from persisted settings via [`state_variant`]; listening
/// is transient, applied by the push-to-talk listener via [`show_listening`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateVariant {
    Idle,
    Listening,
    CapturesPaused,
}

/// Pure state helper (kept free of Tauri so it is unit-testable).
fn state_variant(paused: bool) -> StateVariant {
    if paused {
        StateVariant::CapturesPaused
    } else {
        StateVariant::Idle
    }
}

/// Tray-icon PNG for a variant. macOS ships the template set (pure black +
/// alpha, tinted by the OS for light/dark menu bars — the v4 glyph carries no
/// colour); other platforms ship the colored fallbacks. The `@2x` rasters are
/// the design's 16pt glyph at 2× — crisp on Retina, downsampled on 1×.
#[cfg(target_os = "macos")]
fn icon_bytes(variant: StateVariant) -> &'static [u8] {
    match variant {
        StateVariant::Idle => include_bytes!("../icons/tray/tray-idle-template@2x.png"),
        StateVariant::Listening => {
            include_bytes!("../icons/tray/tray-listening-template@2x.png")
        }
        StateVariant::CapturesPaused => {
            include_bytes!("../icons/tray/tray-captures-paused-template@2x.png")
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn icon_bytes(variant: StateVariant) -> &'static [u8] {
    match variant {
        StateVariant::Idle => include_bytes!("../icons/tray/tray-idle.png"),
        StateVariant::Listening => include_bytes!("../icons/tray/tray-listening.png"),
        StateVariant::CapturesPaused => include_bytes!("../icons/tray/tray-captures-paused.png"),
    }
}

/// Swaps the tray icon to `variant` on the main thread — matching the
/// `run_on_main_thread` style already used in `lib.rs`. Safe to call from any
/// thread; failures are logged and the tray keeps its previous icon.
fn apply_variant(app: &tauri::AppHandle, variant: StateVariant) {
    let app = app.clone();
    let runner = app.clone();
    let _ = runner.run_on_main_thread(move || {
        let Some(tray) = app.tray_by_id("main") else {
            eprintln!("ruoxi: tray icon skipped: tray 'main' not found");
            return;
        };
        match tauri::image::Image::from_bytes(icon_bytes(variant)) {
            Ok(icon) => {
                if let Err(e) = tray.set_icon_with_as_template(Some(icon), true) {
                    eprintln!("ruoxi: tray icon update failed ({variant:?}): {e}");
                }
            }
            Err(e) => eprintln!("ruoxi: tray icon decode failed ({variant:?}): {e}"),
        }
    });
}

/// Re-derives the tray icon from persisted settings (paused wins over idle;
/// the offline kill-switch is carried in words, not in the glyph).
pub fn refresh_icon(app: &tauri::AppHandle) {
    let current = settings::load(app);
    apply_variant(app, state_variant(current.pause_captures));
}

/// Shows the listening glyph while the push-to-talk mic is open.
pub fn show_listening(app: &tauri::AppHandle) {
    apply_variant(app, StateVariant::Listening);
}

fn send_intent(
    intents: &Option<std::sync::mpsc::Sender<hotkeys::CaptureIntent>>,
    intent: hotkeys::CaptureIntent,
) {
    if let Some(tx) = intents {
        if tx.send(intent).is_err() {
            eprintln!("ruoxi: capture worker unavailable");
        }
    }
}

pub fn install(
    app: &tauri::AppHandle,
    intents: Option<std::sync::mpsc::Sender<hotkeys::CaptureIntent>>,
) -> tauri::Result<()> {
    let current = settings::load(app);

    let pause = CheckMenuItem::with_id(
        app,
        "pause_captures",
        "Pause Captures",
        true,
        current.pause_captures,
        None::<&str>,
    )?;
    let offline = CheckMenuItem::with_id(
        app,
        "offline_mode",
        "Offline Mode",
        true,
        current.offline,
        None::<&str>,
    )?;
    let show_panel =
        MenuItem::with_id(app, "show_panel", "Show Result Panel", true, None::<&str>)?;
    let capture_region =
        MenuItem::with_id(app, "capture_region", "Capture Region", true, None::<&str>)?;
    let capture_window =
        MenuItem::with_id(app, "capture_window", "Capture Window", true, None::<&str>)?;
    let capture_fullscreen = MenuItem::with_id(
        app,
        "capture_fullscreen",
        "Capture Whole Screen",
        true,
        None::<&str>,
    )?;
    let settings_item = MenuItem::with_id(app, "show_settings", "Settings…", true, None::<&str>)?;
    let timeline_item = MenuItem::with_id(app, "open_timeline", "Timeline", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Ruòxī", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &show_panel,
            &capture_region,
            &capture_window,
            &capture_fullscreen,
            &sep1,
            &timeline_item,
            &sep2,
            &pause,
            &offline,
            &sep3,
            &settings_item,
            &quit,
        ],
    )?;

    let builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .icon(
            tauri::image::Image::from_bytes(icon_bytes(StateVariant::Idle))
                .expect("idle tray icon"),
        )
        .tooltip("Ruoxi — starting");
    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);
    builder
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "show_panel" => crate::panel::show(app),
            "capture_region" => send_intent(&intents, hotkeys::CaptureIntent::Region),
            "capture_window" => send_intent(&intents, hotkeys::CaptureIntent::Window),
            "capture_fullscreen" => send_intent(&intents, hotkeys::CaptureIntent::Fullscreen),
            "show_settings" => crate::settings::show(app),
            "open_timeline" => crate::timeline::show(app),
            "pause_captures" => {
                let mut s = settings::load(app);
                s.pause_captures = !s.pause_captures;
                if let Err(e) = settings::save(app, &s) {
                    eprintln!("ruoxi: failed to persist settings: {e}");
                }
                let _ = app.emit("settings:changed", &s);
                refresh_icon(app);
            }
            "offline_mode" => {
                let mut s = settings::load(app);
                s.offline = !s.offline;
                if let Err(e) = settings::save(app, &s) {
                    eprintln!("ruoxi: failed to persist settings: {e}");
                }
                let _ = app.emit("settings:changed", &s);
                refresh_icon(app);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paused_wins_over_idle() {
        assert_eq!(state_variant(true), StateVariant::CapturesPaused);
        assert_eq!(state_variant(false), StateVariant::Idle);
    }

    #[test]
    fn each_variant_maps_to_a_distinct_nonempty_icon() {
        let variants = [
            StateVariant::Idle,
            StateVariant::Listening,
            StateVariant::CapturesPaused,
        ];
        for variant in variants {
            assert!(!icon_bytes(variant).is_empty(), "{variant:?} icon is empty");
        }
        for (i, a) in variants.iter().enumerate() {
            for b in &variants[i + 1..] {
                assert!(
                    !std::ptr::eq(icon_bytes(*a), icon_bytes(*b)),
                    "{a:?} and {b:?} share an icon"
                );
            }
        }
    }
}
