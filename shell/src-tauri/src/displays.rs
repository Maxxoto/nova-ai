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
pub fn list_displays(app: tauri::AppHandle) -> Vec<DisplayInfo> {
    let mut displays = platform::list();
    let names = platform::localized_names(&app);
    for display in displays.iter_mut() {
        if let Some(name) = names.get(&display.id) {
            display.name = name.clone();
        }
    }
    displays
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

    /// OS-level display names (NSScreen.localizedName) keyed by display id,
    /// replacing xcap's "Display #N" placeholders. NSScreen is main-thread
    /// only, so the lookup hops to the app's main thread.
    pub fn localized_names(app: &tauri::AppHandle) -> std::collections::HashMap<String, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let spawn = app.run_on_main_thread(move || {
            let _ = tx.send(screen_names_on_main());
        });
        if spawn.is_err() {
            return std::collections::HashMap::new();
        }
        rx.recv_timeout(std::time::Duration::from_secs(2))
            .unwrap_or_default()
    }

    pub(super) fn screen_names_on_main() -> std::collections::HashMap<String, String> {
        use objc2_app_kit::NSScreen;
        use objc2_foundation::{NSNumber, NSString};

        let mut names = std::collections::HashMap::new();
        let Some(mtm) = objc2::MainThreadMarker::new() else {
            return names;
        };
        let screen_number_key = NSString::from_str("NSScreenNumber");
        for screen in NSScreen::screens(mtm) {
            let description = screen.deviceDescription();
            let Some(value) = description.objectForKey(&screen_number_key) else {
                continue;
            };
            let Ok(number) = value.downcast::<NSNumber>() else {
                continue;
            };
            names.insert(
                number.unsignedIntValue().to_string(),
                screen.localizedName().to_string(),
            );
        }
        names
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::DisplayInfo;

    pub fn list() -> Vec<DisplayInfo> {
        Vec::new()
    }
}

/// Probe shims for examples (no AppHandle available there).
pub fn xcap_list() -> Vec<DisplayInfo> {
    platform::list()
}

#[cfg(target_os = "macos")]
pub fn screen_names_probe() -> std::collections::HashMap<String, String> {
    platform::screen_names_on_main()
}

#[cfg(not(target_os = "macos"))]
pub fn screen_names_probe() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}
