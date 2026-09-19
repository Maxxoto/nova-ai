//! Tray icon + menu (M0). RFC-0002 §4.2 — accessory-resident, no windows.
//!
//! The tray is the honest state surface (docs/DESIGN.md `components.tray-menu`):
//! the offline kill-switch and capture pause live here as checkboxes, and the
//! icon mirrors the persisted state so the tray alone tells the truth.

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;

use crate::{hotkeys, settings};

/// The tray icon variant. Offline/Paused/Idle are derived from persisted
/// settings by [`state_variant`]; `Degraded` is applied once by the supervisor
/// when the sidecar restart budget is spent.
///
/// Precedence for the settings-derived states is **offline > paused > idle**
/// (docs/DESIGN.md `components.tray-menu` + `components.capture-paused-state`):
/// the kill-switch is the strongest signal, so it wins even while captures are
/// also paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateVariant {
    Offline,
    CapturesPaused,
    Idle,
    /// Sidecar gave up (RFC-0002 §4.7; DESIGN.md AI state table: degraded →
    /// tray variant). Not settings-derived: applied via [`show_degraded`].
    Degraded,
}

/// Pure state-precedence helper (kept free of Tauri so it is unit-testable).
fn state_variant(offline: bool, paused: bool) -> StateVariant {
    if offline {
        StateVariant::Offline
    } else if paused {
        StateVariant::CapturesPaused
    } else {
        StateVariant::Idle
    }
}

/// Tray-icon PNG for a variant. macOS ships the template set (pure black + alpha,
/// tinted by the OS for light/dark menu bars — DESIGN.md tray recommendation);
/// other platforms ship the colored marks.
/// macOS ships the `@2x` template rasters: `tray-icon` lays the glyph out at
/// 18pt, so the 36px asset is the design's "16px logical glyph at 2×" — crisp on
/// Retina, downsampled on 1×. Pure black + alpha; the OS tints the bar.
#[cfg(target_os = "macos")]
fn icon_bytes(variant: StateVariant) -> &'static [u8] {
    match variant {
        StateVariant::Offline => include_bytes!("../icons/tray/tray-offline-template@2x.png"),
        StateVariant::CapturesPaused => {
            include_bytes!("../icons/tray/tray-captures-paused-template@2x.png")
        }
        StateVariant::Idle => include_bytes!("../icons/tray/tray-idle-template@2x.png"),
        StateVariant::Degraded => {
            include_bytes!("../icons/tray/tray-degraded-template@2x.png")
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn icon_bytes(variant: StateVariant) -> &'static [u8] {
    match variant {
        StateVariant::Offline => include_bytes!("../icons/tray/tray-offline.png"),
        StateVariant::CapturesPaused => include_bytes!("../icons/tray/tray-captures-paused.png"),
        StateVariant::Idle => include_bytes!("../icons/tray/tray-idle.png"),
        StateVariant::Degraded => include_bytes!("../icons/tray/tray-degraded.png"),
    }
}

/// Applies the tray icon matching the persisted offline/paused state.
///
/// Loads settings, picks the variant (offline > paused > idle), then swaps the
/// icon on the main thread — matching the `run_on_main_thread` style already
/// used in `lib.rs`. Safe to call from any thread; failures are logged and the
/// tray keeps its previous icon rather than panicking.
pub fn refresh_icon(app: &tauri::AppHandle) {
    let current = settings::load(app);
    let variant = state_variant(current.offline, current.pause_captures);
    let app = app.clone();
    let runner = app.clone();
    let _ = runner.run_on_main_thread(move || {
        let Some(tray) = app.tray_by_id("main") else {
            eprintln!("ruoxi: tray icon refresh skipped: tray 'main' not found");
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

/// Applies the degraded ("Ruòxī is resting") glyph after the sidecar restart
/// budget is spent (DESIGN.md AI state table: degraded → tray variant). The
/// supervisor is the only caller; the app reports "restart app" from here on.
pub fn show_degraded(app: &tauri::AppHandle) {
    let app = app.clone();
    let runner = app.clone();
    let _ = runner.run_on_main_thread(move || {
        let Some(tray) = app.tray_by_id("main") else {
            eprintln!("ruoxi: degraded icon skipped: tray 'main' not found");
            return;
        };
        match tauri::image::Image::from_bytes(icon_bytes(StateVariant::Degraded)) {
            Ok(icon) => {
                if let Err(e) = tray.set_icon_with_as_template(Some(icon), true) {
                    eprintln!("ruoxi: degraded icon update failed: {e}");
                }
            }
            Err(e) => eprintln!("ruoxi: degraded icon decode failed: {e}"),
        }
    });
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
        "Pause captures",
        true,
        current.pause_captures,
        None::<&str>,
    )?;
    let offline = CheckMenuItem::with_id(
        app,
        "offline_mode",
        "Offline — nothing leaves this Mac",
        true,
        current.offline,
        None::<&str>,
    )?;
    let capture_region =
        MenuItem::with_id(app, "capture_region", "Capture region", true, None::<&str>)?;
    let capture_window =
        MenuItem::with_id(app, "capture_window", "Capture window", true, None::<&str>)?;
    let capture_fullscreen = MenuItem::with_id(
        app,
        "capture_fullscreen",
        "Capture whole screen",
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
    fn variant_precedence_is_offline_then_paused_then_idle() {
        assert_eq!(state_variant(true, true), StateVariant::Offline);
        assert_eq!(state_variant(true, false), StateVariant::Offline);
        assert_eq!(state_variant(false, true), StateVariant::CapturesPaused);
        assert_eq!(state_variant(false, false), StateVariant::Idle);
    }

    #[test]
    fn each_variant_maps_to_a_distinct_nonempty_icon() {
        let variants = [
            StateVariant::Offline,
            StateVariant::CapturesPaused,
            StateVariant::Idle,
            StateVariant::Degraded,
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
