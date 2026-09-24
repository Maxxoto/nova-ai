//! Screen capture scopes (W3, macOS): fullscreen display-under-cursor,
//! frontmost window excluding our own chrome, region via the system picker
//! (RFC-0003 §4.1–4.2). Geometry is captured at physical pixels; the store
//! keeps logical dimensions plus scale factor.

use crate::capture_store::{NewCapture, Scope};

#[derive(Debug)]
pub enum CaptureError {
    Xcap(xcap::XCapError),
    Io(std::io::Error),
    Png(String),
    Cancelled,
    NoTarget,
    Unsupported,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::Xcap(e) => write!(f, "capture: {e}"),
            CaptureError::Io(e) => write!(f, "io: {e}"),
            CaptureError::Png(e) => write!(f, "png: {e}"),
            CaptureError::Cancelled => write!(f, "capture cancelled"),
            CaptureError::NoTarget => write!(f, "no capturable target"),
            CaptureError::Unsupported => write!(f, "unsupported on this platform"),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<xcap::XCapError> for CaptureError {
    fn from(e: xcap::XCapError) -> Self {
        CaptureError::Xcap(e)
    }
}

impl From<std::io::Error> for CaptureError {
    fn from(e: std::io::Error) -> Self {
        CaptureError::Io(e)
    }
}

pub struct Captured {
    pub scope: Scope,
    pub png: Vec<u8>,
    pub w_px: u32,
    pub h_px: u32,
    pub scale: f32,
    pub app: Option<String>,
    pub title: Option<String>,
    pub display_id: Option<u32>,
    /// Capture origin in CG global top-left logical points — what the panel's
    /// "near" placement anchors against. Set by each capture path; `(0, 0)`
    /// means unknown and the panel falls back to centred.
    pub origin_x: f64,
    pub origin_y: f64,
}

pub struct WindowMeta {
    pub pid: u32,
    pub app: String,
    pub title: String,
    pub minimized: bool,
    pub focused: bool,
    pub w: u32,
    pub h: u32,
}

fn is_system_app(app: &str) -> bool {
    matches!(
        app,
        "" | "Window Server" | "Dock" | "SystemUIServer" | "ControlCenter" | "WindowManager"
    )
}

/// Index of the frontmost capturable window: the focused one when it is not
/// ours, else the first candidate in z-order. Minimized, zero-size, system,
/// and own-process windows are never eligible.
pub fn frontmost_candidate(windows: &[WindowMeta], own_pid: u32) -> Option<usize> {
    let eligible = |w: &WindowMeta| {
        !w.minimized && w.w > 0 && w.h > 0 && w.pid != own_pid && !is_system_app(&w.app)
    };
    windows
        .iter()
        .enumerate()
        .find(|(_, w)| w.focused && eligible(w))
        .or_else(|| windows.iter().enumerate().find(|(_, w)| eligible(w)))
        .map(|(i, _)| i)
}

pub fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, CaptureError> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| CaptureError::Png(e.to_string()))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| CaptureError::Png(e.to_string()))?;
    drop(writer);
    Ok(out)
}

/// Reads the IHDR chunk: width/height are the two big-endian u32 fields after
/// the 8-byte signature, 4-byte length, and `IHDR` tag.
pub fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[12..16] != b"IHDR" || !bytes.starts_with(&[0x89, b'P', b'N', b'G'])
    {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((w, h))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn to_new_capture(cap: Captured) -> NewCapture {
    NewCapture {
        scope: cap.scope,
        ts_ms: now_ms(),
        display_id: cap
            .display_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        app: cap.app,
        window_title: cap.title,
        w_px: cap.w_px as i64,
        h_px: cap.h_px as i64,
        scale: cap.scale as f64,
        image: cap.png,
        ext: "png".to_string(),
        auto: false,
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::{Captured, CaptureError, WindowMeta};
    use crate::capture_store::Scope;
    use xcap::{Monitor, Window};

    /// Cursor position in the global display coordinate space (logical points,
    /// top-left origin) — shared with the region overlay.
    pub(crate) fn cursor_point() -> (i32, i32) {
        use core_graphics::event::CGEvent;
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
        let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState);
        let point = match src {
            Ok(src) => CGEvent::new(src).map(|e| e.location()).unwrap_or_default(),
            Err(_) => Default::default(),
        };
        (point.x as i32, point.y as i32)
    }

    fn encode_monitor_image(
        image: xcap::image::RgbaImage,
        scope: Scope,
        app: Option<String>,
        title: Option<String>,
        display_id: Option<u32>,
        scale: f32,
    ) -> Result<Captured, CaptureError> {
        let (w, h) = (image.width(), image.height());
        let png = super::encode_png(image.as_raw(), w, h)?;
        Ok(Captured {
            scope,
            png,
            w_px: w,
            h_px: h,
            scale,
            app,
            title,
            display_id,
            // Callers overwrite the origin with the path's real screen rect.
            origin_x: 0.0,
            origin_y: 0.0,
        })
    }

    pub fn capture_fullscreen(
        preferred_display_id: Option<u32>,
    ) -> Result<Captured, CaptureError> {
        let monitor = preferred_display_id
            .and_then(|id| {
                Monitor::all()
                    .ok()?
                    .into_iter()
                    .find(|m| m.id().ok() == Some(id))
            })
            .or_else(|| {
                let (x, y) = cursor_point();
                Monitor::from_point(x, y).ok()
            })
            .ok_or(CaptureError::NoTarget)?;
        let scale = monitor.scale_factor().unwrap_or(1.0);
        let display_id = monitor.id().ok();
        let image = monitor.capture_image()?;
        let mut captured =
            encode_monitor_image(image, Scope::Fullscreen, None, None, display_id, scale)?;
        // The screen path's rect is the display itself: CGDisplayBounds
        // origin, already logical top-left points.
        captured.origin_x = monitor.x().unwrap_or(0) as f64;
        captured.origin_y = monitor.y().unwrap_or(0) as f64;
        Ok(captured)
    }

    pub fn capture_window(own_pid: u32) -> Result<Captured, CaptureError> {
        let windows = Window::all()?;
        let metas: Vec<WindowMeta> = windows
            .iter()
            .map(|w| WindowMeta {
                pid: w.pid().unwrap_or(0),
                app: w.app_name().unwrap_or_default(),
                title: w.title().unwrap_or_default(),
                minimized: w.is_minimized().unwrap_or(true),
                focused: w.is_focused().unwrap_or(false),
                w: w.width().unwrap_or(0),
                h: w.height().unwrap_or(0),
            })
            .collect();
        let idx = super::frontmost_candidate(&metas, own_pid).ok_or(CaptureError::NoTarget)?;
        let window = &windows[idx];
        let image = window.capture_image()?;
        let scale = window
            .current_monitor()
            .and_then(|m| m.scale_factor())
            .unwrap_or(1.0);
        let display_id = window.current_monitor().ok().and_then(|m| m.id().ok());
        let mut captured = encode_monitor_image(
            image,
            Scope::Window,
            Some(metas[idx].app.clone()),
            Some(metas[idx].title.clone()),
            display_id,
            scale,
        )?;
        // The window path's rect is the frontmost window: its kCGWindowBounds
        // origin, already logical top-left points.
        captured.origin_x = window.x().unwrap_or(0) as f64;
        captured.origin_y = window.y().unwrap_or(0) as f64;
        Ok(captured)
    }

    pub fn capture_region_interactive() -> Result<Captured, CaptureError> {
        use std::process::Command;
        let path = std::env::temp_dir().join(format!(
            "ruoxi-region-{}.png",
            ulid::Ulid::generate()
        ));
        let status = Command::new("/usr/sbin/screencapture")
            .arg("-i")
            .arg("-x")
            .arg(&path)
            .status()?;
        if !status.success() {
            return Err(CaptureError::Cancelled);
        }
        let png = std::fs::read(&path)?;
        let _ = std::fs::remove_file(&path);
        let (w, h) = super::png_dimensions(&png).ok_or(CaptureError::NoTarget)?;
        let (x, y) = cursor_point();
        let monitor = Monitor::from_point(x, y).ok();
        Ok(Captured {
            scope: Scope::Region,
            png,
            w_px: w,
            h_px: h,
            scale: monitor
                .as_ref()
                .and_then(|m| m.scale_factor().ok())
                .unwrap_or(1.0),
            app: None,
            title: None,
            display_id: monitor.and_then(|m| m.id().ok()),
            // The system picker never reports the rect it grabbed; the cursor
            // at return time (the drag's release point) is the nearest anchor
            // to the selection, so the panel still lands beside what was
            // boxed rather than falling back to centred.
            origin_x: x as f64,
            origin_y: y as f64,
        })
    }
}

#[cfg(target_os = "macos")]
pub use mac::{capture_fullscreen, capture_region_interactive, capture_window};

#[cfg(target_os = "macos")]
pub(crate) use mac::cursor_point;

#[cfg(not(target_os = "macos"))]
pub fn capture_fullscreen(
    _preferred_display_id: Option<u32>,
) -> Result<Captured, CaptureError> {
    Err(CaptureError::Unsupported)
}

#[cfg(not(target_os = "macos"))]
pub fn capture_window(_own_pid: u32) -> Result<Captured, CaptureError> {
    Err(CaptureError::Unsupported)
}

#[cfg(not(target_os = "macos"))]
pub fn capture_region_interactive() -> Result<Captured, CaptureError> {
    Err(CaptureError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(pid: u32, app: &str, minimized: bool, focused: bool, w: u32, h: u32) -> WindowMeta {
        WindowMeta {
            pid,
            app: app.to_string(),
            title: format!("{app} window"),
            minimized,
            focused,
            w,
            h,
        }
    }

    #[test]
    fn focused_foreign_window_wins() {
        let windows = vec![
            meta(10, "Safari", false, false, 800, 600),
            meta(20, "Terminal", false, true, 600, 400),
        ];
        assert_eq!(frontmost_candidate(&windows, 1), Some(1));
    }

    #[test]
    fn own_focused_window_is_skipped() {
        let windows = vec![
            meta(1, "nova-shell", false, true, 400, 300),
            meta(30, "Mail", false, false, 900, 700),
        ];
        assert_eq!(frontmost_candidate(&windows, 1), Some(1));
    }

    #[test]
    fn system_and_minimized_windows_are_ineligible() {
        let windows = vec![
            meta(2, "Dock", false, false, 1920, 1080),
            meta(3, "Safari", true, false, 800, 600),
            meta(4, "", false, false, 100, 100),
            meta(5, "Notes", false, false, 0, 400),
            meta(6, "Mail", false, false, 500, 500),
        ];
        assert_eq!(frontmost_candidate(&windows, 1), Some(4));
    }

    #[test]
    fn empty_list_has_no_candidate() {
        assert_eq!(frontmost_candidate(&[], 1), None);
    }

    #[test]
    fn png_roundtrip_dimensions() {
        let rgba = vec![255u8; 2 * 3 * 4];
        let png = encode_png(&rgba, 2, 3).expect("encode");
        assert_eq!(png_dimensions(&png), Some((2, 3)));
        assert!(png.len() > 8);
    }

    #[test]
    fn garbage_is_not_a_png() {
        assert_eq!(png_dimensions(b"not a png at all........"), None);
    }

    #[test]
    #[ignore = "needs a real display; run explicitly to exercise the live path"]
    fn fullscreen_capture_runs() {
        match capture_fullscreen(None) {
            Ok(cap) => println!("captured {}x{} png {} bytes", cap.w_px, cap.h_px, cap.png.len()),
            Err(e) => println!("capture error (expected without Screen Recording grant): {e}"),
        }
    }
}
