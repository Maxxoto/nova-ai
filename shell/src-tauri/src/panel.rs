//! Result panel window (W4) — frameless, always-on-top, hidden until shown.
//! Spec: RFC-0002 §4.4 + docs/DESIGN.md `panel-shell`.
//!
//! S4 verdict route: the panel shows as a non-activating window (keystrokes
//! keep flowing to the user's app) and a listen-only event tap delivers Esc
//! pass-through while the panel is visible (AC-01/AC-06).

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

pub const PANEL_LABEL: &str = "panel";

pub static PANEL_VISIBLE: AtomicBool = AtomicBool::new(false);

const NS_STYLE_NONACTIVATING: usize = 1 << 7;
const NS_COLL_CAN_JOIN_ALL_SPACES: usize = 1 << 0;
const NS_COLL_IGNORES_CYCLE: usize = 1 << 2;
const NS_COLL_FULLSCREEN_AUXILIARY: usize = 1 << 8;

const ESC_KEYCODE: i64 = 53; // kVK_Escape

/// One-time macOS window tuning: non-activating, visible on every space
/// incl. fullscreen, excluded from the window cycle.
pub fn apply_macos_panel_style(app: &AppHandle) {
    let Some(win) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };
    let Ok(raw) = win.ns_window() else {
        return;
    };
    let ns_win = raw as *mut objc2::runtime::AnyObject;
    unsafe {
        let mask: usize = objc2::msg_send![&*ns_win, styleMask];
        let (): () = objc2::msg_send![&*ns_win, setStyleMask: mask | NS_STYLE_NONACTIVATING];
        let behavior: usize = objc2::msg_send![&*ns_win, collectionBehavior];
        let (): () = objc2::msg_send![
            &*ns_win,
            setCollectionBehavior: behavior
                | NS_COLL_CAN_JOIN_ALL_SPACES
                | NS_COLL_IGNORES_CYCLE
                | NS_COLL_FULLSCREEN_AUXILIARY
        ];
    }
}

pub fn show(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(PANEL_LABEL) {
        let _ = win.show();
        PANEL_VISIBLE.store(true, Ordering::Relaxed);
        eprintln!("ruoxi: panel shown (non-activating)");
    }
}

pub fn hide(app: &AppHandle) {
    crate::tts::stop_speaking();
    if let Some(win) = app.get_webview_window(PANEL_LABEL) {
        let _ = win.hide();
        PANEL_VISIBLE.store(false, Ordering::Relaxed);
        eprintln!("ruoxi: panel hidden");
    }
}

/// Listen-only Esc tap: hides the panel when visible, never consumes the
/// key — the underlying app sees its own Esc semantics (S1 tap pattern).
pub fn spawn_esc_dismiss(app: AppHandle) -> Result<(), String> {
    use core_foundation::base::TCFType;
    use core_foundation::runloop::CFRunLoop;
    use core_foundation::string::CFString;
    use core_graphics::event::{
        CallbackResult, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
        CGEventType, EventField,
    };

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    std::thread::spawn(move || {
        let tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown],
            move |_proxy, _ty, event| {
                use core_graphics::event::CGEventFlags as F;
                let modifier_mask = F::CGEventFlagCommand
                    | F::CGEventFlagShift
                    | F::CGEventFlagAlternate
                    | F::CGEventFlagControl;
                let bare = (event.get_flags() & modifier_mask) == F::CGEventFlagNull;
                let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                if keycode == ESC_KEYCODE && bare && PANEL_VISIBLE.load(Ordering::Relaxed) {
                    hide(&app);
                }
                CallbackResult::Keep
            },
        );
        let tap = match tap {
            Ok(t) => t,
            Err(_) => {
                let _ = ready_tx.send(Err("esc tap create failed".to_string()));
                return;
            }
        };
        let source = match tap.mach_port().create_runloop_source(0) {
            Ok(s) => s,
            Err(_) => {
                let _ = ready_tx.send(Err("esc runloop source failed".to_string()));
                return;
            }
        };
        let mode = CFString::new("kCFRunLoopDefaultMode");
        let rl = CFRunLoop::get_current();
        rl.add_source(&source, mode.as_concrete_TypeRef());
        tap.enable();
        let _ = ready_tx.send(Ok(()));
        CFRunLoop::run_current();
    });
    ready_rx.recv().map_err(|_| "esc tap thread died".to_string())?
}

#[tauri::command]
pub fn show_panel(app: AppHandle) {
    show(&app);
}

#[tauri::command]
pub fn hide_panel(app: AppHandle) {
    hide(&app);
}
