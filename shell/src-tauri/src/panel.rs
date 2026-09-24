//! Result panel window (W4) — frameless, always-on-top, hidden until shown.
//! Spec: RFC-0002 §4.4 + docs/DESIGN.md `panel-shell`.
//!
//! S4 verdict route: the panel shows as a non-activating window (keystrokes
//! keep flowing to the user's app) and a listen-only event tap delivers Esc
//! pass-through while the panel is visible (AC-01/AC-06).

use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    /// Re-arms a tap after macOS disables it (timeout / user input).
    fn CGEventTapEnable(tap: *mut std::os::raw::c_void, enable: bool);
}

pub const PANEL_LABEL: &str = "panel";

pub static PANEL_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Bumped on every show. A hide applies only if no show has happened since it
/// was requested, so a late hide (the webview dismisses 120 ms after Esc, or a
/// stray tap) cannot turn off a panel that was just shown.
static VISIBILITY_EPOCH: AtomicU64 = AtomicU64::new(0);

/// Whether the result panel is currently up — the PTT consumer routes a press
/// into the capture-scoped ask only while it is.
pub fn is_visible() -> bool {
    PANEL_VISIBLE.load(Ordering::Relaxed)
}

// --- Placement (mini mode + panel placement) -------------------------------
//
// Pure geometry only — no ObjC — so the placement contract is unit-testable.
// Matches the design's `panel-slot`/`mini-slot` CSS (capture-and-ask.html
// 793–812): the 9 `is-<anchor>` cells share a 24px inset, and "near" hugs the
// capture with a 12px gap.

/// (x, y, w, h) in AppKit bottom-left screen coords.
pub type Rect = (f64, f64, f64, f64);

pub const PANEL_W: f64 = 424.0;
pub const PANEL_H: f64 = 480.0;
pub const MINI_W: f64 = 44.0;
pub const MINI_H: f64 = 44.0;
pub const ANCHOR_INSET: f64 = 24.0;
pub const CAPTURE_GAP: f64 = 12.0;

/// The one per-mode size table (mini mark vs full panel) so the window build,
/// the mode switch and the hide-restore can never drift.
pub fn mode_size(mode: &str) -> Option<(f64, f64)> {
    match mode {
        "mini" => Some((MINI_W, MINI_H)),
        "panel" => Some((PANEL_W, PANEL_H)),
        _ => None,
    }
}

fn is_anchor(anchor: &str) -> bool {
    matches!(
        anchor,
        "tl" | "tc" | "tr" | "ml" | "mc" | "mr" | "bl" | "bc" | "br"
    )
}

/// Clamps a window origin into `visible`. When the window is larger than the
/// visible frame the max bound collapses onto the min side, so a window
/// bigger than its screen still gets an on-screen origin (never panics on
/// `min > max`).
fn clamp_origin(visible: Rect, win: (f64, f64), x: f64, y: f64) -> (f64, f64) {
    let (vx, vy, vw, vh) = visible;
    let (ww, wh) = win;
    let max_x = (vx + vw - ww).max(vx);
    let max_y = (vy + vh - wh).max(vy);
    (x.clamp(vx, max_x), y.clamp(vy, max_y))
}

fn centred(visible: Rect, win: (f64, f64)) -> (f64, f64) {
    let (vx, vy, vw, vh) = visible;
    let (ww, wh) = win;
    clamp_origin(visible, win, vx + (vw - ww) / 2.0, vy + (vh - wh) / 2.0)
}

/// The 9-cell anchor grid: `anchor` is `<row><col>` with rows `t/m/b` and
/// cols `l/c/r`. Only valid anchors reach here.
fn anchored(visible: Rect, win: (f64, f64), anchor: &str) -> (f64, f64) {
    let (vx, vy, vw, vh) = visible;
    let (ww, wh) = win;
    let bytes = anchor.as_bytes();
    let x = match bytes[1] {
        b'l' => vx + ANCHOR_INSET,
        b'c' => vx + (vw - ww) / 2.0,
        _ => vx + vw - ANCHOR_INSET - ww,
    };
    let y = match bytes[0] {
        b't' => vy + vh - ANCHOR_INSET - wh,
        b'm' => vy + (vh - wh) / 2.0,
        _ => vy + ANCHOR_INSET,
    };
    clamp_origin(visible, win, x, y)
}

/// Beside the capture: RIGHT with a 12px gap first, then LEFT, BELOW, ABOVE;
/// the first candidate that fits `visible` wins, and the result is clamped
/// so a capture wider/taller than the display still yields an on-screen
/// origin.
fn near_capture(visible: Rect, win: (f64, f64), capture: Rect) -> (f64, f64) {
    let (vx, vy, vw, vh) = visible;
    let (ww, wh) = win;
    let (cx, cy, cw, ch) = capture;
    let fits = |(x, y): (f64, f64)| x >= vx && y >= vy && x + ww <= vx + vw && y + wh <= vy + vh;
    let candidates = [
        (cx + cw + CAPTURE_GAP, cy + (ch - wh) / 2.0),
        (cx - CAPTURE_GAP - ww, cy + (ch - wh) / 2.0),
        (cx + (cw - ww) / 2.0, cy - CAPTURE_GAP - wh),
        (cx + (cw - ww) / 2.0, cy + ch + CAPTURE_GAP),
    ];
    let (x, y) = candidates
        .iter()
        .copied()
        .find(|candidate| fits(*candidate))
        .unwrap_or(candidates[0]);
    clamp_origin(visible, win, x, y)
}

/// Where a `win`-sized panel window goes on `visible` (the target screen's
/// visible frame, AppKit coords):
///
/// - `"fixed"` — the 9-anchor grid, `ANCHOR_INSET` from the edges (a capture
///   is not needed);
/// - `"near"` — beside `capture` when one is known;
/// - anything else (no capture, unknown placement/anchor) — centred, the
///   behaviour the shell shipped before placement existed.
pub fn place_panel(
    visible: Rect,
    win: (f64, f64),
    capture: Option<Rect>,
    placement: &str,
    anchor: &str,
) -> (f64, f64) {
    match (placement, anchor, capture) {
        ("fixed", a, _) if is_anchor(a) => anchored(visible, win, a),
        ("near", _, Some(capture)) => near_capture(visible, win, capture),
        _ => centred(visible, win),
    }
}

/// CG global top-left rect → AppKit bottom-left rect, anchored on the target
/// screen's own CG bounds + AppKit frame (correct on any display of a
/// multi-monitor layout, not just the primary). CG and AppKit x run the same
/// way; only y flips: the capture's bottom edge sits `capture_cg.1 +
/// capture_cg.3` below the screen's CG top, which is `screen_ns.1 +
/// screen_ns.3` in AppKit coords.
fn flip_cg_to_appkit(capture_cg: Rect, screen_cg: Rect, screen_ns: Rect) -> Rect {
    (
        capture_cg.0,
        screen_ns.1 + screen_ns.3 + screen_cg.1 - capture_cg.1 - capture_cg.3,
        capture_cg.2,
        capture_cg.3,
    )
}

/// The capture rect for placement in AppKit screen coords, or `None` when
/// there is no capture, it lives on another display, or its screen number is
/// unknown — the centred fallback covers all three.
fn capture_rect_ns(
    capture: Option<&crate::ask::CurrentCapture>,
    screen_number: u32,
    screen_ns: Rect,
) -> Option<Rect> {
    let capture = capture?;
    if screen_number == 0 || screen_number.to_string() != capture.display_id {
        return None;
    }
    let bounds = core_graphics::display::CGDisplay::new(screen_number).bounds();
    let screen_cg: Rect = (
        bounds.origin.x,
        bounds.origin.y,
        bounds.size.width,
        bounds.size.height,
    );
    Some(flip_cg_to_appkit(capture.cg_rect(), screen_cg, screen_ns))
}

/// Placement prefs from the settings file. `settings::load` falls back to
/// `Settings::default()` on any read/parse failure — whose placement values
/// are the design defaults "near"/"tr" — so this never errors.
fn placement_prefs(app: &AppHandle) -> (String, String) {
    let settings = crate::settings::load(app);
    (settings.panel_placement, settings.panel_anchor)
}

const NS_STYLE_NONACTIVATING: usize = 1 << 7;
const NS_COLL_CAN_JOIN_ALL_SPACES: usize = 1 << 0;
const NS_COLL_IGNORES_CYCLE: usize = 1 << 6;
const NS_COLL_FULLSCREEN_AUXILIARY: usize = 1 << 8;

const ESC_KEYCODE: i64 = 53; // kVK_Escape

/// A fullscreen app lives in its own Space and outranks Tauri's floating
/// level (3); pop-up-menu level is what keeps the panel drawn above it.
const NS_POPUP_MENU_WINDOW_LEVEL: isize = 101;

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
        let (): () = objc2::msg_send![
            &*ns_win,
            setCollectionBehavior: NS_COLL_CAN_JOIN_ALL_SPACES
                | NS_COLL_IGNORES_CYCLE
                | NS_COLL_FULLSCREEN_AUXILIARY
        ];
        let (): () = objc2::msg_send![&*ns_win, setHidesOnDeactivate: false];
        let (): () = objc2::msg_send![&*ns_win, setLevel: NS_POPUP_MENU_WINDOW_LEVEL];
    }
}

/// Shows a window on the current Space above fullscreen apps and (for the
/// panel) places it on the capture's display. The window is ordered in once and
/// then kept ordered in: repeated order-out/order-in cycles degrade (the window
/// stops being composited after a few hides while AppKit still reports it
/// visible). It hides by going transparent instead. Must run on the main thread
/// — setLevel asserts off it (crashed the shell, report 2026-09-21).
pub(crate) fn raise_above_fullscreen(app: &AppHandle, win: &tauri::WebviewWindow, label: &str) {
    let win_main = win.clone();
    let label_main = label.to_string();
    let win_late = win.clone();
    let label_late = label.to_string();
    let _ = app.run_on_main_thread(move || show_panel_style(&win_main, &label_main));

    // Re-assert shortly after in case a Space transition re-tags the window.
    std::thread::spawn(move || {
        let app = win_late.app_handle().clone();
        std::thread::sleep(std::time::Duration::from_millis(120));
        let win = win_late.clone();
        let label = label_late.clone();
        let _ = app.run_on_main_thread(move || show_panel_style(&win, &label));
    });
}

const PANEL_BEHAVIOR: usize =
    NS_COLL_CAN_JOIN_ALL_SPACES | NS_COLL_IGNORES_CYCLE | NS_COLL_FULLSCREEN_AUXILIARY;

/// Makes the window visible: place it, set level/behavior/alpha, and order it in
/// only if it is not already ordered in (the first show). Later shows are pure
/// alpha so the window keeps its Space membership.
fn show_panel_style(win: &tauri::WebviewWindow, label: &str) {
    let Ok(raw) = win.ns_window() else {
        return;
    };
    let ns_win = raw as *mut objc2::runtime::AnyObject;
    unsafe {
        let Some(mtm) = objc2::MainThreadMarker::new() else {
            return;
        };
        // Placement: the panel prefers the display the capture came from; the
        // overlay follows the cursor. Both fall back to the cursor's display,
        // then the primary.
        let prefer_capture = label == "panel";
        let capture = crate::ask::current_capture();
        let capture_display = capture
            .as_ref()
            .map(|c| c.display_id.clone())
            .unwrap_or_default();
        let screens = objc2_app_kit::NSScreen::screens(mtm);
        let key = objc2_foundation::NSString::from_str("NSScreenNumber");
        let screen_number = |screen: &objc2_app_kit::NSScreen| -> u32 {
            let description = screen.deviceDescription();
            match description.objectForKey(&key) {
                Some(value) => {
                    let Ok(number) = value.downcast::<objc2_foundation::NSNumber>() else {
                        return 0;
                    };
                    number.unsignedIntValue()
                }
                None => 0,
            }
        };
        let mouse = objc2_app_kit::NSEvent::mouseLocation();
        let chosen = screens
            .iter()
            .find(|screen| {
                prefer_capture
                    && !capture_display.is_empty()
                    && screen_number(screen).to_string() == capture_display
            })
            .or_else(|| {
                screens.iter().find(|screen| {
                    let frame = screen.frame();
                    mouse.x >= frame.origin.x
                        && mouse.x <= frame.origin.x + frame.size.width
                        && mouse.y >= frame.origin.y
                        && mouse.y <= frame.origin.y + frame.size.height
                })
            })
            .or_else(|| screens.iter().next());
        if let Some(screen) = chosen {
            let win_frame: objc2_foundation::NSRect = objc2::msg_send![&*ns_win, frame];
            let visible = screen.visibleFrame();
            let vis: Rect = (
                visible.origin.x,
                visible.origin.y,
                visible.size.width,
                visible.size.height,
            );
            // The panel places itself per settings + capture rect; the
            // overlay must keep landing exactly on its monitor, which the
            // centred fallback does for a monitor-sized window.
            let (capture_rect, placement, anchor) = if label == "panel" {
                let number = screen_number(&screen);
                let ns_frame = screen.frame();
                let capture_rect = capture_rect_ns(
                    capture.as_ref(),
                    number,
                    (ns_frame.origin.x, ns_frame.origin.y, ns_frame.size.width, ns_frame.size.height),
                );
                let (placement, anchor) = placement_prefs(&win.app_handle());
                (capture_rect, placement, anchor)
            } else {
                (None, String::new(), String::new())
            };
            let (x, y) = place_panel(
                vis,
                (win_frame.size.width, win_frame.size.height),
                capture_rect,
                &placement,
                &anchor,
            );
            let (): () = objc2::msg_send![
                &*ns_win,
                setFrameOrigin: objc2_foundation::NSPoint::new(x, y)
            ];
        }

        let ns_ref = &*ns_win;
        let result = objc2::exception::catch(std::panic::AssertUnwindSafe(|| {
            let (): () = objc2::msg_send![ns_ref, setCollectionBehavior: PANEL_BEHAVIOR];
            let (): () = objc2::msg_send![ns_ref, setLevel: NS_POPUP_MENU_WINDOW_LEVEL];
            let (): () = objc2::msg_send![ns_ref, setAlphaValue: 1.0f64];
            let (): () = objc2::msg_send![ns_ref, setIgnoresMouseEvents: false];
            let (): () = objc2::msg_send![ns_ref, orderFrontRegardless];
            // Nudge the frame so the window server re-commits the window: after
            // repeated show/hide the compositor can stop drawing it (and stop
            // accepting display() calls), leaving it "visible" per AppKit but
            // blank on screen. A one-point frame change forces the re-commit.
            let frame: objc2_foundation::NSRect = objc2::msg_send![ns_ref, frame];
            let nudged = objc2_foundation::NSRect {
                origin: objc2_foundation::NSPoint::new(frame.origin.x, frame.origin.y + 1.0),
                size: frame.size,
            };
            let (): () = objc2::msg_send![ns_ref, setFrame: nudged, display: true];
            let (): () = objc2::msg_send![ns_ref, setFrame: frame, display: true];
        }));
        if let Err(e) = result {
            eprintln!("ruoxi: {label} show caught ObjC exception: {e:?}");
        }
        let level: isize = objc2::msg_send![&*ns_win, level];
        let number: isize = objc2::msg_send![&*ns_win, windowNumber];
        let on_active: bool = objc2::msg_send![&*ns_win, isOnActiveSpace];
        let visible: bool = objc2::msg_send![&*ns_win, isVisible];
        let behavior: usize = objc2::msg_send![&*ns_win, collectionBehavior];
        let win_frame: objc2_foundation::NSRect = objc2::msg_send![&*ns_win, frame];
        let alpha: f64 = objc2::msg_send![&*ns_win, alphaValue];
        eprintln!(
            "ruoxi: {label} shown alpha {alpha} display={capture_display:?} at ({:.0},{:.0}) level {level} win {number} onActiveSpace {on_active} visible {visible} behavior {behavior:#x}",
            win_frame.origin.x,
            win_frame.origin.y,
        );
    }
}

pub fn show(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(PANEL_LABEL) {
        // Tauri's show() must not order the window in first: the order-in is
        // where the window server reads collectionBehavior to tag the Space.
        VISIBILITY_EPOCH.fetch_add(1, Ordering::SeqCst);
        raise_above_fullscreen(app, &win, "panel");
        PANEL_VISIBLE.store(true, Ordering::Relaxed);
        eprintln!("ruoxi: panel shown (non-activating)");
    }
}

pub fn hide(app: &AppHandle) {
    crate::tts::stop_speaking();
    crate::ask::clear_current_capture();
    crate::ask::discard_ask();
    if let Some(win) = app.get_webview_window(PANEL_LABEL) {
        // Transparent + click-through, never ordered out: repeated order cycles
        // leave the window uncomposited after a few hides while AppKit still
        // reports it visible. Staying ordered-in keeps its Space membership.
        let epoch = VISIBILITY_EPOCH.load(Ordering::SeqCst);
        let win_main = win.clone();
        let _ = app.run_on_main_thread(move || {
            if VISIBILITY_EPOCH.load(Ordering::SeqCst) != epoch {
                eprintln!("ruoxi: panel hide superseded by a newer show");
                return;
            }
            let Ok(raw) = win_main.ns_window() else {
                return;
            };
            let ns_win = raw as *mut objc2::runtime::AnyObject;
            unsafe {
                // A dismissed panel comes back as a panel: restore the full
                // size while hidden. A live collapse (set_panel_mode "mini"
                // on a visible panel) never passes through here, so it stays
                // collapsed.
                if let Err(e) = win_main.set_size(tauri::LogicalSize::new(PANEL_W, PANEL_H)) {
                    eprintln!("ruoxi: panel hide restore size failed: {e}");
                }
                let (): () = objc2::msg_send![&*ns_win, setAlphaValue: 0.0f64];
                let (): () = objc2::msg_send![&*ns_win, setIgnoresMouseEvents: true];
            }
        });
        PANEL_VISIBLE.store(false, Ordering::Relaxed);
        if let Err(e) = app.emit("panel:mode", serde_json::json!({ "mode": "panel" })) {
            eprintln!("ruoxi: panel:mode emit failed: {e}");
        }
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
        // The callback cannot capture the tap it belongs to, so the tap port
        // reaches it through this slot and is used to re-arm on disable.
        let tap_port: Arc<AtomicPtr<std::os::raw::c_void>> =
            Arc::new(AtomicPtr::new(std::ptr::null_mut()));
        let port = Arc::clone(&tap_port);
        let tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            // KeyDown only: a mask bit for the TapDisabled* variants shifts
            // 1u64 by 0xFFFFFFFE/0xFFFFFFFF, which panics on overflow in
            // debug builds. Those events are delivered to the callback
            // regardless of the mask, so the re-arm below still fires.
            vec![CGEventType::KeyDown],
            move |_proxy, ty, event| {
                use core_graphics::event::CGEventFlags as F;
                if matches!(
                    ty,
                    CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
                ) {
                    eprintln!("ruoxi: esc tap {ty:?} — re-enabling");
                    let port = port.load(Ordering::Relaxed);
                    if !port.is_null() {
                        unsafe { CGEventTapEnable(port, true) };
                    }
                    return CallbackResult::Keep;
                }
                let flags = event.get_flags();
                let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                eprintln!("ruoxi: esc tap {ty:?} keycode={keycode} flags={flags:?}");
                if keycode == ESC_KEYCODE && PANEL_VISIBLE.load(Ordering::Relaxed) {
                    if crate::ask::is_active() {
                        // Accepted with the ⌥⇧V chord still held — chord + Esc
                        // is the design's cancel gesture.
                        crate::ask::cancel_ask(&app);
                    } else {
                        let modifier_mask = F::CGEventFlagCommand
                            | F::CGEventFlagShift
                            | F::CGEventFlagAlternate
                            | F::CGEventFlagControl;
                        let bare = (flags & modifier_mask) == F::CGEventFlagNull;
                        if bare {
                            // Esc stops the read-aloud; the dismiss hotkey owns
                            // hiding, so Esc can never leave the panel unshown.
                            eprintln!("ruoxi: esc -> stop read-aloud");
                            crate::tts::stop_speaking();
                        }
                    }
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
        tap_port.store(
            tap.mach_port().as_concrete_TypeRef() as *mut std::os::raw::c_void,
            Ordering::Relaxed,
        );
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
    eprintln!("ruoxi: hide requested by hide_panel command");
    hide(&app);
}

/// Switches the panel window between its two sizes — `"mini"` (the 44×44
/// mark) and `"panel"` (424×480) — re-placing it after the resize. All
/// window work runs on the main thread and reuses `show_panel_style`, so
/// level/Space/behaviour/nudge logic stays in one place. An unknown mode
/// changes nothing and returns an error.
#[tauri::command]
pub fn set_panel_mode(app: AppHandle, mode: String) -> Result<(), String> {
    let Some((w, h)) = mode_size(&mode) else {
        return Err(format!(
            "unknown panel mode {mode:?} — expected \"mini\" or \"panel\""
        ));
    };
    let Some(win) = app.get_webview_window(PANEL_LABEL) else {
        return Err("panel window is not built".to_string());
    };
    eprintln!("ruoxi: panel mode -> {mode}");
    let win_main = win.clone();
    let logged_mode = mode.clone();
    let _ = app.run_on_main_thread(move || {
        if let Err(e) = win_main.set_size(tauri::LogicalSize::new(w, h)) {
            eprintln!("ruoxi: panel mode {logged_mode} resize failed: {e}");
            return;
        }
        // Re-place with the new size and re-assert level/Space/alpha/order
        // through the one shared show path.
        show_panel_style(&win_main, "panel");
    });
    if let Err(e) = app.emit("panel:mode", serde_json::json!({ "mode": mode })) {
        eprintln!("ruoxi: panel:mode emit failed: {e}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIS: Rect = (0.0, 0.0, 1920.0, 1080.0);
    const WIN: (f64, f64) = (PANEL_W, PANEL_H);
    const CENTRED: (f64, f64) = (748.0, 300.0);

    #[test]
    fn fixed_anchor_grid_covers_all_nine_cells() {
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "tl"), (24.0, 576.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "tc"), (748.0, 576.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "tr"), (1472.0, 576.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "ml"), (24.0, 300.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "mc"), (748.0, 300.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "mr"), (1472.0, 300.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "bl"), (24.0, 24.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "bc"), (748.0, 24.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "br"), (1472.0, 24.0));
    }

    #[test]
    fn mini_mode_shares_the_anchor_grid() {
        let mini = (MINI_W, MINI_H);
        assert_eq!(place_panel(VIS, mini, None, "fixed", "tr"), (1852.0, 1012.0));
        assert_eq!(place_panel(VIS, mini, None, "fixed", "bl"), (24.0, 24.0));
    }

    #[test]
    fn near_prefers_the_right_of_the_capture() {
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 300.0, 200.0, 200.0)), "near", "tr"),
            (312.0, 160.0)
        );
    }

    #[test]
    fn near_falls_back_left_when_right_does_not_fit() {
        assert_eq!(
            place_panel(VIS, WIN, Some((1600.0, 300.0, 200.0, 200.0)), "near", "tr"),
            (1164.0, 160.0)
        );
    }

    #[test]
    fn near_falls_back_below_then_above() {
        assert_eq!(
            place_panel(VIS, WIN, Some((200.0, 500.0, 1520.0, 80.0)), "near", "tr"),
            (748.0, 8.0)
        );
        assert_eq!(
            place_panel(VIS, WIN, Some((200.0, 20.0, 1520.0, 80.0)), "near", "tr"),
            (748.0, 112.0)
        );
    }

    #[test]
    fn near_clamps_at_every_screen_edge() {
        // Wider than the display AND too tall: no candidate fits, the right
        // one clamps to the right edge (and stays on-screen vertically).
        assert_eq!(
            place_panel(VIS, WIN, Some((-100.0, 300.0, 2200.0, 500.0)), "near", "tr"),
            (1496.0, 310.0)
        );
        // Nothing fits at the top: the right candidate clamps to max y.
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 900.0, 200.0, 100.0)), "near", "tr"),
            (312.0, 600.0)
        );
        // Nothing fits at the bottom: clamps to y = 0.
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 10.0, 200.0, 100.0)), "near", "tr"),
            (312.0, 0.0)
        );
        // Capture hanging off the left: clamps to x = 0.
        assert_eq!(
            place_panel(VIS, WIN, Some((-200.0, 300.0, 100.0, 200.0)), "near", "tr"),
            (0.0, 160.0)
        );
    }

    #[test]
    fn no_capture_or_bogus_input_falls_back_to_centred() {
        assert_eq!(place_panel(VIS, WIN, None, "near", "tr"), CENTRED);
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "zz"), CENTRED);
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 300.0, 200.0, 200.0)), "fixed", "top-right"),
            CENTRED
        );
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 300.0, 200.0, 200.0)), "beside", "tr"),
            CENTRED
        );
    }

    #[test]
    fn flip_maps_cg_top_left_to_appkit_bottom_left() {
        // A 1440×900 primary: a capture 100pt over, 100pt down, 300pt tall
        // has its bottom edge 400pt from the CG top → AppKit y 500.
        assert_eq!(
            flip_cg_to_appkit(
                (100.0, 100.0, 200.0, 300.0),
                (0.0, 0.0, 1440.0, 900.0),
                (0.0, 0.0, 1440.0, 900.0),
            ),
            (100.0, 500.0, 200.0, 300.0)
        );
    }

    #[test]
    fn flip_anchored_on_a_secondary_display() {
        // Secondary right of a 1440×900 primary, tops aligned: CG bounds
        // (1440, 0, 1920, 1080) ↔ AppKit frame (1440, -180, 1920, 1080).
        // The same physical capture must land at the same AppKit y as on the
        // primary case, and x passes through unchanged.
        assert_eq!(
            flip_cg_to_appkit(
                (1540.0, 100.0, 200.0, 300.0),
                (1440.0, 0.0, 1920.0, 1080.0),
                (1440.0, -180.0, 1920.0, 1080.0),
            ),
            (1540.0, 500.0, 200.0, 300.0)
        );
    }

    #[test]
    fn mode_size_knows_exactly_the_two_modes() {
        assert_eq!(mode_size("mini"), Some((44.0, 44.0)));
        assert_eq!(mode_size("panel"), Some((424.0, 480.0)));
        assert_eq!(mode_size("Mini"), None);
        assert_eq!(mode_size(""), None);
    }
}
