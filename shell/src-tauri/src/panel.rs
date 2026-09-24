//! Result panel window (W4) — frameless, always-on-top, hidden until shown.
//! Spec: RFC-0002 §4.4 + docs/DESIGN.md `panel-shell`.
//!
//! S4 verdict route: the panel shows as a non-activating window (keystrokes
//! keep flowing to the user's app) and a listen-only event tap delivers Esc
//! pass-through while the panel is visible (AC-01/AC-06).

use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

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
/// - `"custom"` — `custom`, the origin the native drag watcher persisted,
///   clamped into `visible` so a drag made on a since-unplugged display
///   still lands on-screen (a missing or non-finite origin falls through);
/// - anything else (no capture, unknown placement/anchor) — centred, the
///   behaviour the shell shipped before placement existed.
pub fn place_panel(
    visible: Rect,
    win: (f64, f64),
    capture: Option<Rect>,
    placement: &str,
    anchor: &str,
    custom: Option<(f64, f64)>,
) -> (f64, f64) {
    match (placement, anchor, capture, custom) {
        ("fixed", a, _, _) if is_anchor(a) => anchored(visible, win, a),
        ("near", _, Some(capture), _) => near_capture(visible, win, capture),
        ("custom", _, _, Some((x, y))) if x.is_finite() && y.is_finite() => {
            clamp_origin(visible, win, x, y)
        }
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

/// A Tauri `WindowEvent::Moved` position — physical pixels, global top-left
/// origin, y downward (tao reports the AppKit frame multiplied by the
/// window's backing scale) — converted to the AppKit bottom-left logical
/// origin `place_panel` places in. `win_h_pt` is the window's current height
/// in logical points (its AppKit frame); the screen pair anchors the y flip
/// exactly like [`flip_cg_to_appkit`]: the origin sits `win_h_pt` below the
/// top edge the converted position names.
fn moved_origin_ns(
    pos_px: (f64, f64),
    win_h_pt: f64,
    scale: f64,
    screen_cg: Rect,
    screen_ns: Rect,
) -> (f64, f64) {
    // A sane divisor only: 0/NaN would turn one bad event into a NaN origin.
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let cg_x = pos_px.0 / scale;
    let cg_y = pos_px.1 / scale;
    (
        cg_x,
        screen_ns.1 + screen_ns.3 + screen_cg.1 - cg_y - win_h_pt,
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
/// are the design defaults "near"/"tr" — so this never errors. `custom` is
/// the dragged origin: both axes or none.
fn placement_prefs(app: &AppHandle) -> (String, String, Option<(f64, f64)>) {
    let settings = crate::settings::load(app);
    let custom = match (settings.panel_custom_x, settings.panel_custom_y) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    };
    (settings.panel_placement, settings.panel_anchor, custom)
}

/// The surface a capture opens as — the `panel_open_as` pref ("mini" by
/// default). `settings::load` normalizes it; the fallback only guards a
/// corrupt enum that somehow skipped that.
fn open_mode_pref(app: &AppHandle) -> String {
    let mode = crate::settings::load(app).panel_open_as;
    if mode_size(&mode).is_some() {
        mode
    } else {
        "panel".to_string()
    }
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

/// The CGDirectDisplayID of an NSScreen (0 when its device description
/// carries none) — the key both the capture-display match and the drag
/// watcher's screen lookup dispatch on.
fn screen_number(screen: &objc2_app_kit::NSScreen) -> u32 {
    let key = objc2_foundation::NSString::from_str("NSScreenNumber");
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
}

/// Shows a window on the current Space above fullscreen apps and (for the
/// panel) places it on the capture's display. The window is ordered in once and
/// then kept ordered in: repeated order-out/order-in cycles degrade (the window
/// stops being composited after a few hides while AppKit still reports it
/// visible). It hides by going transparent instead. Must run on the main thread
/// — setLevel asserts off it (crashed the shell, report 2026-09-21).
///
/// `resize`, when set, sizes the window (logical points) via the synchronous
/// AppKit `setContentSize:` route BEFORE the frame is read for placement —
/// Tauri's deferred `set_size` would leave the place computing on a stale
/// frame. This is the one resize+place ordering, shared by the show path
/// and `set_panel_mode` so they cannot drift.
pub(crate) fn raise_above_fullscreen(
    app: &AppHandle,
    win: &tauri::WebviewWindow,
    label: &str,
    resize: Option<(f64, f64)>,
) {
    let win_main = win.clone();
    let label_main = label.to_string();
    let win_late = win.clone();
    let label_late = label.to_string();
    let _ = app
        .run_on_main_thread(move || show_panel_style(&win_main, &label_main, resize));

    // Re-assert shortly after in case a Space transition re-tags the window.
    // No resize: an interim set_panel_mode must not be collapsed back to the
    // show-time size.
    std::thread::spawn(move || {
        let app = win_late.app_handle().clone();
        std::thread::sleep(std::time::Duration::from_millis(120));
        let win = win_late.clone();
        let label = label_late.to_string();
        let _ = app.run_on_main_thread(move || show_panel_style(&win, &label, None));
    });
}

const PANEL_BEHAVIOR: usize =
    NS_COLL_CAN_JOIN_ALL_SPACES | NS_COLL_IGNORES_CYCLE | NS_COLL_FULLSCREEN_AUXILIARY;

/// Makes the window visible: optionally resize it (see
/// [`raise_above_fullscreen`]), place it, set level/behavior/alpha, and order
/// it in only if it is not already ordered in (the first show). Later shows
/// are pure alpha so the window keeps its Space membership.
fn show_panel_style(win: &tauri::WebviewWindow, label: &str, resize: Option<(f64, f64)>) {
    let Ok(raw) = win.ns_window() else {
        return;
    };
    let ns_win = raw as *mut objc2::runtime::AnyObject;
    unsafe {
        let Some(mtm) = objc2::MainThreadMarker::new() else {
            return;
        };
        // The resize must land BEFORE the frame read below, or placement
        // anchors on the stale size (Tauri's set_size funnels through the
        // event loop; setContentSize: commits synchronously). On an ObjC
        // exception, bail instead of placing a half-resized window.
        if let Some((w, h)) = resize {
            let result = objc2::exception::catch(std::panic::AssertUnwindSafe(|| {
                let (): () = objc2::msg_send![
                    &*ns_win,
                    setContentSize: objc2_foundation::NSSize::new(w, h)
                ];
            }));
            if let Err(e) = result {
                eprintln!("ruoxi: {label} resize caught ObjC exception: {e:?}");
                return;
            }
        }
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
            let (capture_rect, placement, anchor, custom) = if label == "panel" {
                let number = screen_number(&screen);
                let ns_frame = screen.frame();
                let capture_rect = capture_rect_ns(
                    capture.as_ref(),
                    number,
                    (ns_frame.origin.x, ns_frame.origin.y, ns_frame.size.width, ns_frame.size.height),
                );
                let (placement, anchor, custom) = placement_prefs(&win.app_handle());
                (capture_rect, placement, anchor, custom)
            } else {
                (None, String::new(), String::new(), None)
            };
            let (x, y) = place_panel(
                vis,
                (win_frame.size.width, win_frame.size.height),
                capture_rect,
                &placement,
                &anchor,
                custom,
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
        // End any drag still flagged: the placements below are programmatic
        // and their Moved events must not be persisted as a custom position.
        cancel_drag();
        // Open in the configured mode (`panel_open_as`, mini by default):
        // the resize rides the same main-thread turn as the place — placement
        // never sees the stale frame — and the mode event goes out
        // immediately so the webview renders the right surface, no flash.
        let mode = open_mode_pref(app);
        let size = mode_size(&mode).unwrap_or((PANEL_W, PANEL_H));
        // Tauri's show() must not order the window in first: the order-in is
        // where the window server reads collectionBehavior to tag the Space.
        VISIBILITY_EPOCH.fetch_add(1, Ordering::SeqCst);
        raise_above_fullscreen(app, &win, "panel", Some(size));
        PANEL_VISIBLE.store(true, Ordering::Relaxed);
        if let Err(e) = app.emit("panel:mode", serde_json::json!({ "mode": mode })) {
            eprintln!("ruoxi: panel:mode emit failed: {e}");
        }
        eprintln!("ruoxi: panel shown (non-activating, opens as {mode})");
    }
}

pub fn hide(app: &AppHandle) {
    crate::tts::stop_speaking();
    crate::ask::clear_current_capture();
    crate::ask::discard_ask();
    cancel_drag();
    if let Some(win) = app.get_webview_window(PANEL_LABEL) {
        // A dismissed panel comes back in the CONFIGURED open mode: restore
        // that mode's size while hidden, so the next show places a
        // right-sized window with no resize flash.
        let mode = open_mode_pref(app);
        let size = mode_size(&mode).unwrap_or((PANEL_W, PANEL_H));
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
                // Same synchronous resize route as the show path. A live
                // collapse (set_panel_mode "mini" on a visible panel) never
                // passes through here, so it stays collapsed.
                let result = objc2::exception::catch(std::panic::AssertUnwindSafe(|| {
                    let (): () = objc2::msg_send![
                        &*ns_win,
                        setContentSize: objc2_foundation::NSSize::new(size.0, size.1)
                    ];
                }));
                if let Err(e) = result {
                    // Log and still hide: an unrestored size is recoverable
                    // on the next show, a visible zombie panel is not.
                    eprintln!("ruoxi: panel hide restore size caught ObjC exception: {e:?}");
                }
                let (): () = objc2::msg_send![&*ns_win, setAlphaValue: 0.0f64];
                let (): () = objc2::msg_send![&*ns_win, setIgnoresMouseEvents: true];
            }
        });
        PANEL_VISIBLE.store(false, Ordering::Relaxed);
        if let Err(e) = app.emit("panel:mode", serde_json::json!({ "mode": mode })) {
            eprintln!("ruoxi: panel:mode emit failed: {e}");
        }
        eprintln!("ruoxi: panel hidden");
    }
}

/// Re-places a VISIBLE panel after its placement settings change (Settings →
/// Panel placement). No resize: `show_panel_style` sizes nothing, it reads
/// the live frame — so a mini stays 44×44 and a panel stays 424×480 — and
/// re-asserts level/behaviour/alpha/order/nudge through the one shared show
/// path. A hidden panel is left alone; the next `show` picks the new setting
/// up on its own.
pub fn reposition(app: &AppHandle) {
    if !is_visible() {
        return;
    }
    // A settings-driven re-place is programmatic: end any live drag so its
    // watcher cannot persist the frame this is about to move.
    cancel_drag();
    let Some(win) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };
    if let Err(e) = app.run_on_main_thread(move || show_panel_style(&win, "panel", None)) {
        eprintln!("ruoxi: panel reposition dispatch failed: {e}");
    }
}

// --- Native drag (the mini disc) --------------------------------------------
//
// The webview has no window-position API and a JS drag would die the moment
// the pointer leaves a 44px window, so the drag is native: mousedown on the
// disc calls `begin_panel_drag`, macOS moves the window, and the Moved
// watcher below persists where the user dropped it.

/// Set between the UI's `begin_panel_drag` and the debounced persist. Every
/// programmatic placement (`show_panel_style`'s `setFrameOrigin` plus its
/// 1-point nudge) also emits `Moved`; this flag is the only thing keeping
/// those from being persisted as a "custom" position.
static PANEL_DRAGGING: AtomicBool = AtomicBool::new(false);

/// The latest drag origin (AppKit bottom-left logical points) while a drag
/// is live. A poisoned lock still owns a usable slot; recover, not panic.
static DRAG_ORIGIN: Mutex<Option<(f64, f64)>> = Mutex::new(None);

/// Bumped by every `Moved`, `begin_panel_drag` and `cancel_drag`. The
/// debounce thread fires only when its 350 ms window saw no further bump —
/// i.e. the drag went quiet.
static DRAG_TICKET: AtomicU64 = AtomicU64::new(0);

/// Whether a debounce thread is already watching this drag, so one burst of
/// `Moved` events costs one thread, not one per event.
static DRAG_PERSIST_ARMED: AtomicBool = AtomicBool::new(false);

const DRAG_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(350);

fn drag_origin_slot() -> MutexGuard<'static, Option<(f64, f64)>> {
    DRAG_ORIGIN.lock().unwrap_or_else(|e| e.into_inner())
}

/// The UI calls this on mousedown on the disc: from here until the debounced
/// persist (or a `cancel_drag`), `Moved` events are the user's drag. The
/// native drag is started by the disc's `data-tauri-drag-region` — NOT here:
/// a second `performWindowDragWithEvent:` from this command would nest two
/// modal drag sessions and hang the main thread. This command only arms the
/// Moved watcher.
#[tauri::command]
pub fn begin_panel_drag(_app: AppHandle) {
    // Drop anything a previous burst left pending — including a sleeper
    // thread's ticket — so a fresh drag cannot inherit a stale origin.
    *drag_origin_slot() = None;
    DRAG_TICKET.fetch_add(1, Ordering::SeqCst);
    PANEL_DRAGGING.store(true, Ordering::Relaxed);
    eprintln!("ruoxi: panel drag begin");
}

/// Ends any live drag: the pending origin is dropped and the debounce thread
/// exits on its next wake without persisting. Every programmatic re-place
/// (show, hide, reposition, set_panel_mode) runs through this, so a drag
/// that never moved — or one cut short — cannot corrupt the placement prefs.
fn cancel_drag() {
    PANEL_DRAGGING.store(false, Ordering::Relaxed);
    *drag_origin_slot() = None;
    DRAG_TICKET.fetch_add(1, Ordering::SeqCst);
}

/// Registers the `Moved` watcher on the panel window (called once from
/// `lib.rs` after the window is built).
pub fn install_drag_watcher(app: &AppHandle) {
    let Some(win) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };
    let app = app.clone();
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::Moved(pos) = event {
            on_panel_moved(&app, *pos);
        }
    });
}

/// The `Moved` handler. The first line is the whole contract: without the
/// dragging flag, every programmatic placement would land here too and be
/// persisted as "custom".
fn on_panel_moved(app: &AppHandle, pos: tauri::PhysicalPosition<i32>) {
    if !PANEL_DRAGGING.load(Ordering::Relaxed) {
        return;
    }
    let app = app.clone();
    let runner = app.clone();
    if let Err(e) = runner.run_on_main_thread(move || {
        let Some(win) = app.get_webview_window(PANEL_LABEL) else {
            return;
        };
        let Ok(raw) = win.ns_window() else {
            return;
        };
        let ns_win = raw as *mut objc2::runtime::AnyObject;
        unsafe {
            let Some(mtm) = objc2::MainThreadMarker::new() else {
                return;
            };
            // The live frame carries the height; backingScaleFactor is the
            // factor tao multiplied into the physical position it reports.
            let frame: objc2_foundation::NSRect = objc2::msg_send![&*ns_win, frame];
            let scale: f64 = objc2::msg_send![&*ns_win, backingScaleFactor];
            let logical_x = pos.x as f64 / scale;
            let logical_y = pos.y as f64 / scale;
            // The screen the window is on now (a drag can cross displays):
            // the first whose CG bounds contain the converted point, else
            // the primary.
            let screens = objc2_app_kit::NSScreen::screens(mtm);
            let screen = screens
                .iter()
                .find(|screen| {
                    let number = screen_number(screen);
                    if number == 0 {
                        return false;
                    }
                    let bounds = core_graphics::display::CGDisplay::new(number).bounds();
                    logical_x >= bounds.origin.x
                        && logical_x <= bounds.origin.x + bounds.size.width
                        && logical_y >= bounds.origin.y
                        && logical_y <= bounds.origin.y + bounds.size.height
                })
                .or_else(|| screens.iter().next());
            let Some(screen) = screen else {
                return;
            };
            let bounds =
                core_graphics::display::CGDisplay::new(screen_number(&screen)).bounds();
            let ns = screen.frame();
            let origin = moved_origin_ns(
                (pos.x as f64, pos.y as f64),
                frame.size.height,
                scale,
                (
                    bounds.origin.x,
                    bounds.origin.y,
                    bounds.size.width,
                    bounds.size.height,
                ),
                (ns.origin.x, ns.origin.y, ns.size.width, ns.size.height),
            );
            *drag_origin_slot() = Some(origin);
            DRAG_TICKET.fetch_add(1, Ordering::SeqCst);
            arm_drag_persist(app.clone());
        }
    }) {
        eprintln!("ruoxi: panel moved dispatch failed: {e}");
    }
}

/// Resets [`DRAG_PERSIST_ARMED`] when the debounce thread ends — on break
/// and on panic alike (a leaked `true` would disable drag persistence for
/// the rest of the process).
struct DisarmOnDrop;

impl Drop for DisarmOnDrop {
    fn drop(&mut self) {
        DRAG_PERSIST_ARMED.store(false, Ordering::SeqCst);
    }
}

/// Spawns the one debounce thread a drag burst gets: after 350 ms with no
/// further `Moved`, the origin is persisted as the custom placement and the
/// drag ends.
fn arm_drag_persist(app: AppHandle) {
    if DRAG_PERSIST_ARMED.swap(true, Ordering::SeqCst) {
        return; // a debounce thread already watches this drag
    }
    std::thread::spawn(move || {
        // Hold DRAG_PERSIST_ARMED for the thread's whole lifetime, not just
        // the spawn: the reset rides Drop, so it fires only when the loop
        // below actually ends.
        let _disarm = DisarmOnDrop;
        loop {
            let seen = DRAG_TICKET.load(Ordering::SeqCst);
            std::thread::sleep(DRAG_DEBOUNCE);
            if DRAG_TICKET.load(Ordering::SeqCst) != seen {
                continue; // movement continued — restart the quiet window
            }
            if !PANEL_DRAGGING.load(Ordering::Relaxed) {
                break; // show/hide cancelled the drag: persist nothing
            }
            if let Some(origin) = drag_origin_slot().take() {
                PANEL_DRAGGING.store(false, Ordering::Relaxed);
                persist_custom_origin(&app, origin);
                // A Moved that landed between the take and the persist re-filled
                // the slot (or re-armed a drag): keep watching instead of
                // orphaning it.
                if drag_origin_slot().is_some() || PANEL_DRAGGING.load(Ordering::Relaxed) {
                    continue;
                }
                break;
            }
            // begin_panel_drag with no Moved yet — keep watching while live.
        }
    });
}

/// Persists where the user dropped the panel as the custom placement.
/// Deliberately no `settings:changed` emit and no `reposition`: the window
/// is already exactly where the user put it, and re-placing it would fight
/// the drag that just ended.
fn persist_custom_origin(app: &AppHandle, origin: (f64, f64)) {
    let mut settings = crate::settings::load(app);
    settings.panel_placement = "custom".to_string();
    settings.panel_custom_x = Some(origin.0);
    settings.panel_custom_y = Some(origin.1);
    match crate::settings::save(app, &settings) {
        Ok(()) => eprintln!(
            "ruoxi: panel drag persisted custom ({:.0},{:.0})",
            origin.0, origin.1
        ),
        Err(e) => eprintln!("ruoxi: panel drag persist failed: {e}"),
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
/// mark) and `"panel"` (424×480) — re-placing it after the resize. The
/// resize + place ordering lives in `show_panel_style`'s resize param (the
/// same route the show path uses, so the two cannot drift), and the rest of
/// the window work reuses `show_panel_style`, so level/Space/behaviour/nudge
/// logic stays in one place. An unknown mode changes nothing and returns an
/// error.
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
    // A mode switch is a programmatic re-place: end any live drag so its
    // watcher cannot persist the position this resize is about to overwrite.
    cancel_drag();
    let win_main = win.clone();
    let _ = app.run_on_main_thread(move || {
        show_panel_style(&win_main, "panel", Some((w, h)));
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
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "tl", None), (24.0, 576.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "tc", None), (748.0, 576.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "tr", None), (1472.0, 576.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "ml", None), (24.0, 300.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "mc", None), (748.0, 300.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "mr", None), (1472.0, 300.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "bl", None), (24.0, 24.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "bc", None), (748.0, 24.0));
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "br", None), (1472.0, 24.0));
    }

    #[test]
    fn mini_mode_shares_the_anchor_grid() {
        let mini = (MINI_W, MINI_H);
        assert_eq!(place_panel(VIS, mini, None, "fixed", "tr", None), (1852.0, 1012.0));
        assert_eq!(place_panel(VIS, mini, None, "fixed", "bl", None), (24.0, 24.0));
    }

    #[test]
    fn near_prefers_the_right_of_the_capture() {
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 300.0, 200.0, 200.0)), "near", "tr", None),
            (312.0, 160.0)
        );
    }

    #[test]
    fn near_falls_back_left_when_right_does_not_fit() {
        assert_eq!(
            place_panel(VIS, WIN, Some((1600.0, 300.0, 200.0, 200.0)), "near", "tr", None),
            (1164.0, 160.0)
        );
    }

    #[test]
    fn near_falls_back_below_then_above() {
        assert_eq!(
            place_panel(VIS, WIN, Some((200.0, 500.0, 1520.0, 80.0)), "near", "tr", None),
            (748.0, 8.0)
        );
        assert_eq!(
            place_panel(VIS, WIN, Some((200.0, 20.0, 1520.0, 80.0)), "near", "tr", None),
            (748.0, 112.0)
        );
    }

    #[test]
    fn near_clamps_at_every_screen_edge() {
        // Wider than the display AND too tall: no candidate fits, the right
        // one clamps to the right edge (and stays on-screen vertically).
        assert_eq!(
            place_panel(VIS, WIN, Some((-100.0, 300.0, 2200.0, 500.0)), "near", "tr", None),
            (1496.0, 310.0)
        );
        // Nothing fits at the top: the right candidate clamps to max y.
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 900.0, 200.0, 100.0)), "near", "tr", None),
            (312.0, 600.0)
        );
        // Nothing fits at the bottom: clamps to y = 0.
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 10.0, 200.0, 100.0)), "near", "tr", None),
            (312.0, 0.0)
        );
        // Capture hanging off the left: clamps to x = 0.
        assert_eq!(
            place_panel(VIS, WIN, Some((-200.0, 300.0, 100.0, 200.0)), "near", "tr", None),
            (0.0, 160.0)
        );
    }

    #[test]
    fn no_capture_or_bogus_input_falls_back_to_centred() {
        assert_eq!(place_panel(VIS, WIN, None, "near", "tr", None), CENTRED);
        assert_eq!(place_panel(VIS, WIN, None, "fixed", "zz", None), CENTRED);
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 300.0, 200.0, 200.0)), "fixed", "top-right", None),
            CENTRED
        );
        assert_eq!(
            place_panel(VIS, WIN, Some((100.0, 300.0, 200.0, 200.0)), "beside", "tr", None),
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

    #[test]
    fn custom_inside_the_visible_frame_is_kept_verbatim() {
        assert_eq!(
            place_panel(VIS, WIN, None, "custom", "tr", Some((100.0, 200.0))),
            (100.0, 200.0)
        );
    }

    #[test]
    fn custom_off_every_edge_is_clamped_back_on_screen() {
        assert_eq!(
            place_panel(VIS, WIN, None, "custom", "tr", Some((-500.0, -500.0))),
            (0.0, 0.0)
        );
        assert_eq!(
            place_panel(VIS, WIN, None, "custom", "tr", Some((5000.0, 5000.0))),
            (1496.0, 600.0)
        );
    }

    #[test]
    fn custom_without_a_usable_origin_falls_back_to_centred() {
        assert_eq!(place_panel(VIS, WIN, None, "custom", "tr", None), CENTRED);
        // A corrupt (non-finite) origin must not leak NaN into a frame.
        assert_eq!(
            place_panel(VIS, WIN, None, "custom", "tr", Some((f64::NAN, 10.0))),
            CENTRED
        );
        assert_eq!(
            place_panel(VIS, WIN, None, "custom", "tr", Some((10.0, f64::NAN))),
            CENTRED
        );
    }

    #[test]
    fn custom_position_also_places_the_mini_mark() {
        assert_eq!(
            place_panel(VIS, (MINI_W, MINI_H), None, "custom", "bl", Some((300.0, 400.0))),
            (300.0, 400.0)
        );
    }

    #[test]
    fn moved_origin_divides_by_scale_on_a_retina_primary() {
        // 2× 1440×900 primary: physical (400,300) is logical (200,150) from
        // the top-left; a 480pt-tall panel's bottom edge sits 480pt lower,
        // i.e. AppKit y 900 − 150 − 480 = 270.
        assert_eq!(
            moved_origin_ns(
                (400.0, 300.0),
                480.0,
                2.0,
                (0.0, 0.0, 1440.0, 900.0),
                (0.0, 0.0, 1440.0, 900.0),
            ),
            (200.0, 270.0)
        );
    }

    #[test]
    fn moved_origin_flips_on_a_secondary_display() {
        // The layout of `flip_anchored_on_a_secondary_display`: secondary
        // right of a 1440×900 primary, tops aligned — CG bounds
        // (1440, 0, 1920, 1080) ↔ AppKit frame (1440, -180, 1920, 1080).
        // A window whose top-left is CG (1540,100), 300pt tall, lands at the
        // same AppKit y as the primary flip: 500.
        assert_eq!(
            moved_origin_ns(
                (1540.0, 100.0),
                300.0,
                1.0,
                (1440.0, 0.0, 1920.0, 1080.0),
                (1440.0, -180.0, 1920.0, 1080.0),
            ),
            (1540.0, 500.0)
        );
    }

    #[test]
    fn moved_origin_treats_a_broken_scale_factor_as_one() {
        assert_eq!(
            moved_origin_ns(
                (400.0, 300.0),
                480.0,
                0.0,
                (0.0, 0.0, 1440.0, 900.0),
                (0.0, 0.0, 1440.0, 900.0),
            ),
            (400.0, 120.0)
        );
    }
}
