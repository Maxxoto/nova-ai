//! Region-capture overlay (W3, macOS; RFC-0003 §4.2). The Region hotkey opens
//! this transparent, always-on-top window in place of the system picker. The UI
//! drags a rectangle and commits logical viewport coordinates; the Rust side
//! then hides the overlay *before* grabbing pixels, so our own selection chrome
//! is never in the shot (drag → commit → overlay hides → capture).
//!
//! The display is resolved once in [`start`] and remembered in [`OverlayState`]
//! so [`capture_region_commit`] crops exactly the viewport the user saw.
//! Multi-monitor picking (F-14) is post-v0: the display under the cursor wins.

use std::sync::Mutex;

pub const OVERLAY_LABEL: &str = "overlay";

/// A selection smaller than this (logical CSS px) is treated as a stray click,
/// not a region.
const MIN_SELECTION_LOGICAL: f64 = 8.0;

/// The display the overlay was opened on. `w_logical`/`h_logical` are the
/// window's viewport in logical points; `scale` converts those to the native
/// pixels that `capture_image()` returns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayTarget {
    pub monitor_id: u32,
    pub scale: f64,
    pub w_logical: f64,
    pub h_logical: f64,
}

/// Managed like `BrainLink`: `None` when no region session is active.
#[derive(Default)]
pub struct OverlayState(Mutex<Option<OverlayTarget>>);

impl OverlayState {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    fn set(&self, target: Option<OverlayTarget>) {
        match self.0.lock() {
            Ok(mut slot) => *slot = target,
            Err(e) => eprintln!("ruoxi: overlay state poisoned: {e}"),
        }
    }

    #[cfg(target_os = "macos")]
    fn get(&self) -> Option<OverlayTarget> {
        self.0.lock().ok().and_then(|slot| *slot)
    }
}

/// Maps a logical, monitor-relative selection (CSS px in the overlay viewport)
/// onto physical pixels of the captured display bitmap. Edges are rounded,
/// clamped to the bitmap, and the result is rejected when the selection is
/// smaller than [`MIN_SELECTION_LOGICAL`] or falls entirely off-display.
pub fn crop_rect(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
    img_w: u32,
    img_h: u32,
) -> Option<(u32, u32, u32, u32)> {
    if !(scale.is_finite() && scale > 0.0)
        || !x.is_finite()
        || !y.is_finite()
        || !w.is_finite()
        || !h.is_finite()
    {
        return None;
    }

    let (mut x, mut y, mut w, mut h) = (x, y, w, h);
    if w < 0.0 {
        x += w;
        w = -w;
    }
    if h < 0.0 {
        y += h;
        h = -h;
    }
    if w < MIN_SELECTION_LOGICAL || h < MIN_SELECTION_LOGICAL {
        return None;
    }

    let left = (x * scale).round().clamp(0.0, img_w as f64);
    let top = (y * scale).round().clamp(0.0, img_h as f64);
    let right = ((x + w) * scale).round().clamp(0.0, img_w as f64);
    let bottom = ((y + h) * scale).round().clamp(0.0, img_h as f64);

    let cw = (right - left) as u32;
    let ch = (bottom - top) as u32;
    if cw == 0 || ch == 0 {
        return None;
    }
    Some((left as u32, top as u32, cw, ch))
}

/// Opens the overlay on the display under the cursor, falling back to focusing
/// an existing overlay window. The target is stored before the window is shown
/// so a commit racing the drag already finds it.
#[cfg(target_os = "macos")]
pub fn start(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        // Ordering, level and collection behavior belong to raise_above_fullscreen:
        // Tauri's show()/set_always_on_top/set_visible_on_all_workspaces would
        // order the window in or re-tag the Space before our behavior is set.
        crate::panel::raise_above_fullscreen(app, &win, "overlay", None);
        eprintln!("ruoxi: overlay re-shown (existing window)");
        let _ = win.set_focus();
        return Ok(());
    }

    let (cx, cy) = crate::capture::cursor_point();
    let monitor =
        xcap::Monitor::from_point(cx, cy).map_err(|e| format!("overlay: no display: {e}"))?;
    let scale = monitor.scale_factor().map_err(|e| e.to_string())? as f64;
    let monitor_id = monitor.id().map_err(|e| e.to_string())?;

    // `Monitor` geometry comes from macOS `CGDisplayBounds`, so x/y/width/height
    // are already logical points; the backing store is `* scale` and that is
    // what `capture_image()` below returns. The window takes the same logical
    // points, which keeps a Retina display fully covered.
    let x_logical = monitor.x().map_err(|e| e.to_string())? as f64;
    let y_logical = monitor.y().map_err(|e| e.to_string())? as f64;
    let w_logical = monitor.width().map_err(|e| e.to_string())? as f64;
    let h_logical = monitor.height().map_err(|e| e.to_string())? as f64;

    let state = app.state::<OverlayState>();
    state.set(Some(OverlayTarget {
        monitor_id,
        scale,
        w_logical,
        h_logical,
    }));

    let built = WebviewWindowBuilder::new(
        app,
        OVERLAY_LABEL,
        WebviewUrl::App("index.html?view=overlay".into()),
    )
    .title("Ruoxi region")
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .position(x_logical, y_logical)
    .inner_size(w_logical, h_logical)
    .visible(true)
    .focused(true)
    .build();

    match built {
        Ok(win) => {
            // Ordering, level and collection behavior belong to raise_above_fullscreen.
            crate::panel::raise_above_fullscreen(app, &win, "overlay", None);
            eprintln!("ruoxi: overlay built");
            let _ = win.set_focus();
            Ok(())
        }
        Err(e) => {
            state.set(None);
            Err(format!("overlay: window build failed: {e}"))
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn start(_app: &tauri::AppHandle) -> Result<(), String> {
    Err("overlay: unsupported on this platform".to_string())
}

/// Destroys the overlay window (it is rebuilt on the next Region hotkey) and
/// forgets the target.
#[tauri::command]
pub fn overlay_cancel(app: tauri::AppHandle) {
    use tauri::Manager;
    dismiss(&app);
    app.state::<OverlayState>().set(None);
}

#[tauri::command]
pub fn start_region_capture(app: tauri::AppHandle) -> Result<(), String> {
    start(&app)
}

/// Commits a drag: the overlay is hidden first, then the display is captured at
/// native resolution and cropped to the logical rect, stored, and surfaced in
/// the panel. Any failure still leaves no overlay behind.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn capture_region_commit(
    app: tauri::AppHandle,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Result<(), String> {
    use tauri::Manager;

    dismiss(&app);
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;

    let result = commit(&app, x, y, w, h);
    app.state::<OverlayState>().set(None);
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            dismiss(&app);
            Err(e)
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn capture_region_commit(
    _app: tauri::AppHandle,
    _x: f64,
    _y: f64,
    _w: f64,
    _h: f64,
) -> Result<(), String> {
    Err("overlay: unsupported on this platform".to_string())
}

#[cfg(target_os = "macos")]
fn commit(app: &tauri::AppHandle, x: f64, y: f64, w: f64, h: f64) -> Result<(), String> {
    use tauri::Manager;

    let target = app
        .state::<OverlayState>()
        .get()
        .ok_or_else(|| "overlay: no active region session".to_string())?;

    let monitor = monitor_by_id(target.monitor_id).or_else(|| {
        let (cx, cy) = crate::capture::cursor_point();
        xcap::Monitor::from_point(cx, cy).ok()
    });
    let monitor = monitor.ok_or_else(|| "overlay: display unavailable".to_string())?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("overlay: capture failed: {e}"))?;

    let (cx, cy, cw, ch) = crop_rect(x, y, w, h, target.scale, image.width(), image.height())
        .ok_or_else(|| "overlay: selection too small or off-display".to_string())?;

    let cropped = xcap::image::imageops::crop_imm(&image, cx, cy, cw, ch).to_image();
    let png =
        crate::capture::encode_png(cropped.as_raw(), cw, ch).map_err(|e| e.to_string())?;

    // The selection is CSS px in the overlay viewport, and the overlay window
    // sits at the monitor's CG origin sized to the monitor — so viewport
    // coords shift directly into CG global logical points.
    let origin = (
        monitor.x().map(|v| v as f64).unwrap_or(0.0) + x,
        monitor.y().map(|v| v as f64).unwrap_or(0.0) + y,
    );

    let captured = crate::capture::Captured {
        scope: crate::capture_store::Scope::Region,
        png,
        w_px: cw,
        h_px: ch,
        scale: target.scale as f32,
        app: None,
        title: None,
        display_id: Some(target.monitor_id),
        origin_x: origin.0,
        origin_y: origin.1,
    };

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("overlay: app data dir: {e}"))?;
    let store = crate::capture_store::CaptureStore::open(&data_dir).map_err(|e| e.to_string())?;
    let (record, _created) = store
        .insert(&crate::capture::to_new_capture(captured))
        .map_err(|e| e.to_string())?;

    // Scope the panel to this capture before the raise reads it, mirroring the
    // capture worker: show-before-set raced the display placement.
    crate::ask::set_current_capture(&record, origin);
    crate::panel::show(app);
    crate::emit_capture(app, &record, origin);
    Ok(())
}

#[cfg(target_os = "macos")]
fn dismiss(app: &tauri::AppHandle) {
    use tauri::Manager;
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        if let Err(e) = win.hide() {
            eprintln!("ruoxi: overlay hide failed: {e}");
        }
        if let Err(close_err) = win.close() {
            if let Err(destroy_err) = win.destroy() {
                eprintln!("ruoxi: overlay close failed ({close_err}), destroy failed: {destroy_err}");
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn dismiss(_app: &tauri::AppHandle) {}

#[cfg(target_os = "macos")]
fn monitor_by_id(id: u32) -> Option<xcap::Monitor> {
    xcap::Monitor::all()
        .ok()?
        .into_iter()
        .find(|m| m.id().ok() == Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_scales_logical_to_physical() {
        assert_eq!(
            crop_rect(10.0, 20.0, 100.0, 50.0, 2.0, 800, 600),
            Some((20, 40, 200, 100))
        );
    }

    #[test]
    fn crop_rounds_fractional_edges() {
        assert_eq!(
            crop_rect(1.0, 1.0, 10.0, 10.0, 1.5, 100, 100),
            Some((2, 2, 15, 15))
        );
    }

    #[test]
    fn crop_clamps_to_the_bitmap() {
        assert_eq!(
            crop_rect(-50.0, -50.0, 1000.0, 1000.0, 1.0, 500, 400),
            Some((0, 0, 500, 400))
        );
    }

    #[test]
    fn crop_rejects_tiny_selections() {
        assert_eq!(crop_rect(10.0, 10.0, 7.9, 50.0, 2.0, 800, 600), None);
        assert_eq!(crop_rect(10.0, 10.0, 50.0, 7.9, 2.0, 800, 600), None);
    }

    #[test]
    fn crop_rejects_off_display_and_degenerate() {
        assert_eq!(crop_rect(2000.0, 10.0, 100.0, 100.0, 1.0, 500, 500), None);
        assert_eq!(crop_rect(10.0, 10.0, 0.0, 0.0, 1.0, 500, 500), None);
        assert_eq!(crop_rect(10.0, 10.0, 100.0, 100.0, 0.0, 500, 500), None);
    }

    #[test]
    fn crop_normalizes_reverse_drags() {
        let forward = crop_rect(10.0, 20.0, 100.0, 50.0, 2.0, 800, 600);
        let reverse = crop_rect(110.0, 70.0, -100.0, -50.0, 2.0, 800, 600);
        assert_eq!(forward, reverse);
        assert_eq!(forward, Some((20, 40, 200, 100)));
    }
}
