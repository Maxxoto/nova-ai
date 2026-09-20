//! macOS permission checks/requests for the onboarding ritual (AC-12).
//!
//! `permissions_status` reports the current TCC state; `permissions_request`
//! triggers the OS prompt (or opens the relevant Privacy pane) and re-reads it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsStatus {
    pub screen_recording: bool,
    pub microphone: String,
    pub accessibility: bool,
}

#[tauri::command]
pub fn permissions_status() -> Result<PermissionsStatus, String> {
    Ok(platform::status())
}

#[tauri::command]
pub fn permissions_request(kind: String) -> Result<PermissionsStatus, String> {
    platform::request(&kind)?;
    Ok(platform::status())
}

pub fn pane_for(kind: &str) -> Option<&'static str> {
    match kind {
        "screen_recording" => Some("Privacy_ScreenCapture"),
        "microphone" => Some("Privacy_Microphone"),
        "accessibility" => Some("Privacy_Accessibility"),
        _ => None,
    }
}

#[tauri::command]
pub fn open_privacy_pane(kind: String) -> Result<(), String> {
    let pane = pane_for(&kind).ok_or_else(|| format!("unknown privacy pane kind: {kind}"))?;
    platform::open_privacy_pane(pane);
    Ok(())
}

#[cfg(target_os = "macos")]
mod platform {
    use super::PermissionsStatus;
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::string::{CFString, CFStringRef};
    use core_graphics::access::ScreenCaptureAccess;
    use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        static kAXTrustedCheckOptionPrompt: CFStringRef;
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    }

    fn accessibility_trusted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    fn accessibility_prompt() -> bool {
        let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
        let options = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
        unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
    }

    pub fn open_privacy_pane(pane: &str) {
        let url = format!("x-apple.systempreferences:com.apple.preference.security?{pane}");
        if let Err(e) = std::process::Command::new("open").arg(url).spawn() {
            eprintln!("ruoxi: failed to open privacy pane: {e}");
        }
    }

    pub fn status() -> PermissionsStatus {
        PermissionsStatus {
            screen_recording: ScreenCaptureAccess.preflight(),
            microphone: microphone_status(),
            accessibility: accessibility_trusted(),
        }
    }

    fn microphone_status() -> String {
        let Some(media_type) = (unsafe { AVMediaTypeAudio }) else {
            return "unknown".to_string();
        };
        unsafe {
            let status = AVCaptureDevice::authorizationStatusForMediaType(media_type);
            if status == AVAuthorizationStatus::Authorized {
                "granted"
            } else if status == AVAuthorizationStatus::NotDetermined {
                "not_determined"
            } else if status == AVAuthorizationStatus::Denied
                || status == AVAuthorizationStatus::Restricted
            {
                "denied"
            } else {
                "unknown"
            }
            .to_string()
        }
    }

    fn request_microphone() {
        let Some(media_type) = (unsafe { AVMediaTypeAudio }) else {
            return;
        };
        let block = block2::RcBlock::new(|granted: objc2::runtime::Bool| {
            eprintln!(
                "ruoxi: microphone {}",
                if granted.as_bool() { "granted" } else { "denied" }
            );
        });
        unsafe {
            AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &block);
        }
    }

    pub fn request(kind: &str) -> Result<(), String> {
        match kind {
            "screen_recording" => {
                let _ = ScreenCaptureAccess.request();
                Ok(())
            }
            "accessibility" => {
                let _ = accessibility_prompt();
                Ok(())
            }
            "microphone" => {
                request_microphone();
                Ok(())
            }
            other => Err(format!("unknown permission kind: {other}")),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::PermissionsStatus;

    pub fn status() -> PermissionsStatus {
        PermissionsStatus {
            screen_recording: false,
            microphone: "unknown".to_string(),
            accessibility: false,
        }
    }

    pub fn open_privacy_pane(_pane: &str) {}

    pub fn request(kind: &str) -> Result<(), String> {
        match kind {
            "screen_recording" | "microphone" | "accessibility" => Ok(()),
            other => Err(format!("unknown permission kind: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_for_maps_every_known_kind() {
        assert_eq!(pane_for("screen_recording"), Some("Privacy_ScreenCapture"));
        assert_eq!(pane_for("microphone"), Some("Privacy_Microphone"));
        assert_eq!(pane_for("accessibility"), Some("Privacy_Accessibility"));
        assert_eq!(pane_for("camera"), None);
    }
}
