//! Display enumeration for Settings/Onboarding (xcap-backed, no capture).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub primary: bool,
}

#[tauri::command]
pub fn list_displays() -> Vec<DisplayInfo> {
    platform::list()
}

#[cfg(target_os = "macos")]
mod platform {
    use super::DisplayInfo;
    use xcap::Monitor;

    pub fn list() -> Vec<DisplayInfo> {
        match Monitor::all() {
            Ok(monitors) => monitors
                .iter()
                .map(|m| DisplayInfo {
                    id: m.id().map(|v| v.to_string()).unwrap_or_default(),
                    name: m.name().unwrap_or_default(),
                    width: m.width().unwrap_or(0),
                    height: m.height().unwrap_or(0),
                    scale: m.scale_factor().unwrap_or(1.0) as f64,
                    primary: m.is_primary().unwrap_or(false),
                })
                .collect(),
            Err(e) => {
                eprintln!("ruoxi: list_displays failed: {e}");
                Vec::new()
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::DisplayInfo;

    pub fn list() -> Vec<DisplayInfo> {
        Vec::new()
    }
}
