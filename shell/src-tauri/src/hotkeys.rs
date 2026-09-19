//! Capture hotkeys (W2): accelerators → capture intents. Registration happens
//! via tauri-plugin-global-shortcut in `lib::run`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureIntent {
    Region,
    Window,
    Fullscreen,
}

pub const ALL_ACCELERATORS: [&str; 3] = ["Alt+Shift+R", "Alt+Shift+W", "Alt+Shift+F"];

pub fn accelerator(intent: CaptureIntent) -> &'static str {
    match intent {
        CaptureIntent::Region => ALL_ACCELERATORS[0],
        CaptureIntent::Window => ALL_ACCELERATORS[1],
        CaptureIntent::Fullscreen => ALL_ACCELERATORS[2],
    }
}

pub fn intent_from_accelerator(shortcut: &str) -> Option<CaptureIntent> {
    use std::collections::BTreeSet;
    let tokens: BTreeSet<String> = shortcut
        .to_lowercase()
        .split('+')
        .map(|t| match t.trim() {
            "option" => "alt".to_string(),
            other => other.trim_start_matches("key").to_string(),
        })
        .collect();
    if !tokens.contains("alt") || !tokens.contains("shift") {
        return None;
    }
    if tokens.contains("r") {
        Some(CaptureIntent::Region)
    } else if tokens.contains("w") {
        Some(CaptureIntent::Window)
    } else if tokens.contains("f") {
        Some(CaptureIntent::Fullscreen)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accelerators_round_trip() {
        for intent in [
            CaptureIntent::Region,
            CaptureIntent::Window,
            CaptureIntent::Fullscreen,
        ] {
            assert_eq!(intent_from_accelerator(accelerator(intent)), Some(intent));
        }
    }

    #[test]
    fn parsing_is_case_and_space_insensitive() {
        assert_eq!(
            intent_from_accelerator("ALT+SHIFT+F"),
            Some(CaptureIntent::Fullscreen)
        );
        assert_eq!(
            intent_from_accelerator("Alt + Shift + W"),
            Some(CaptureIntent::Window)
        );
    }

    #[test]
    fn unknown_accelerators_are_ignored() {
        assert_eq!(intent_from_accelerator("Cmd+Q"), None);
        assert_eq!(intent_from_accelerator(""), None);
    }

    #[test]
    fn plugin_shortcut_display_round_trips() {
        use tauri_plugin_global_shortcut::Shortcut;
        for accelerator in ALL_ACCELERATORS {
            let shortcut: Shortcut = accelerator
                .parse()
                .unwrap_or_else(|e| panic!("parse {accelerator}: {e:?}"));
            let displayed = shortcut.to_string();
            assert_eq!(
                intent_from_accelerator(&displayed),
                intent_from_accelerator(accelerator),
                "display form {displayed:?} of {accelerator} lost its intent"
            );
        }
    }
}
