//! Capture hotkeys (W2): accelerators → capture intents. Registration happens
//! via tauri-plugin-global-shortcut in `lib::run`.
//!
//! Also owns the rebindable push-to-talk helpers: [`ptt_keycode`] maps an
//! accelerator to the macOS virtual keycode the `ptt` event tap matches, and
//! [`ptt_conflict`] / [`validate_ptt_hotkey`] reject capture-shortcut clashes.
//! All pure and unit-tested; [`validate_hotkey`] is the thin command wrapper.

use std::collections::BTreeSet;

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

/// The one accelerator normalization shared by intent matching, conflict
/// detection and keycode lookup: lowercase, split on `+`, `option` aliases to
/// `alt`, and a leading `key` (DOM `KeyboardEvent.code` style) is stripped.
/// Order and duplicates are discarded.
fn accelerator_tokens(shortcut: &str) -> BTreeSet<String> {
    shortcut
        .to_lowercase()
        .split('+')
        .map(|t| match t.trim() {
            "option" => "alt".to_string(),
            other => other.trim_start_matches("key").to_string(),
        })
        .collect()
}

pub fn intent_from_accelerator(shortcut: &str) -> Option<CaptureIntent> {
    let tokens = accelerator_tokens(shortcut);
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

/// Modifier tokens. The PTT event tap matches only the key code and does not
/// enforce modifiers yet, so they are parsed out and ignored during keycode
/// lookup.
const PTT_MODIFIERS: [&str; 10] = [
    "cmd", "command", "ctrl", "control", "alt", "option", "shift", "meta", "super", "fn",
];

/// macOS virtual keycode (`kVK_*`) for the single key named by a PTT
/// accelerator, or `None` when no one known key remains. Modifiers are ignored
/// by design — `"Cmd+Shift+F8"` still resolves to F8 — and `key`/`digit`
/// prefixes from DOM-style captures are stripped.
///
/// Codes are in the same space as `ptt::DEFAULT_PTT_KEYCODE` (100 = F8), the
/// space `EventField::KEYBOARD_EVENT_KEYCODE` reports.
pub fn ptt_keycode(accel: &str) -> Option<i64> {
    keycode_for_key(&ptt_key_token(accel)?)
}

pub const MOD_SHIFT: u32 = 1 << 0;
pub const MOD_ALT: u32 = 1 << 1;
pub const MOD_CTRL: u32 = 1 << 2;
pub const MOD_CMD: u32 = 1 << 3;
pub const MOD_FN: u32 = 1 << 4;

/// Modifier bitmask of `accel` — the enforcement side of [`ptt_keycode`].
/// `"Alt+Shift+V"` → `MOD_ALT | MOD_SHIFT`; `"F8"` → `0` (bare key).
pub fn ptt_modifiers(accel: &str) -> u32 {
    accelerator_tokens(accel)
        .into_iter()
        .filter(|token| PTT_MODIFIERS.contains(&token.as_str()))
        .fold(0u32, |mask, token| {
            mask
                | match token.as_str() {
                    "shift" => MOD_SHIFT,
                    "alt" | "option" => MOD_ALT,
                    "ctrl" | "control" => MOD_CTRL,
                    "cmd" | "command" | "meta" | "super" => MOD_CMD,
                    "fn" => MOD_FN,
                    _ => 0,
                }
        })
}

/// The lone non-modifier token of `accel`, normalized (`"KeyR"` → `"r"`,
/// `"Digit1"` → `"1"`). `None` when there is no key or more than one.
fn ptt_key_token(accel: &str) -> Option<String> {
    let mut keys = accelerator_tokens(accel)
        .into_iter()
        .filter(|token| !token.is_empty() && !PTT_MODIFIERS.contains(&token.as_str()))
        .map(|token| token.trim_start_matches("digit").to_string());
    let key = keys.next()?;
    if keys.next().is_some() {
        return None;
    }
    Some(key)
}

/// Hardware keycodes from Carbon's `Events.h`: `kVK_ANSI_*`, `kVK_F*`,
/// `kVK_Space`, `kVK_Return`, `kVK_Tab`, `kVK_Escape`.
fn keycode_for_key(key: &str) -> Option<i64> {
    let code = match key {
        "a" => 0,
        "s" => 1,
        "d" => 2,
        "f" => 3,
        "h" => 4,
        "g" => 5,
        "z" => 6,
        "x" => 7,
        "c" => 8,
        "v" => 9,
        "b" => 11,
        "q" => 12,
        "w" => 13,
        "e" => 14,
        "r" => 15,
        "y" => 16,
        "t" => 17,
        "1" => 18,
        "2" => 19,
        "3" => 20,
        "4" => 21,
        "6" => 22,
        "5" => 23,
        "9" => 25,
        "7" => 26,
        "8" => 28,
        "0" => 29,
        "o" => 31,
        "u" => 32,
        "i" => 34,
        "p" => 35,
        "return" | "enter" => 36,
        "l" => 37,
        "j" => 38,
        "k" => 40,
        "n" => 45,
        "m" => 46,
        "tab" => 48,
        "space" => 49,
        "escape" | "esc" => 53,
        "f5" => 96,
        "f6" => 97,
        "f7" => 98,
        "f3" => 99,
        "f8" => 100,
        "f9" => 101,
        "f11" => 103,
        "f10" => 109,
        "f12" => 111,
        "f4" => 118,
        "f2" => 120,
        "f1" => 122,
        _ => return None,
    };
    Some(code)
}

/// True when `accel` duplicates one of `capture_accels` under the shared
/// normalization (`"Option + Shift + R"` == `"Alt+Shift+R"`). Token order and
/// spacing do not matter.
pub fn ptt_conflict(accel: &str, capture_accels: &[&str]) -> bool {
    let tokens = accelerator_tokens(accel);
    capture_accels
        .iter()
        .any(|candidate| accelerator_tokens(candidate) == tokens)
}

/// Validates a candidate push-to-talk accelerator. The `Err` string is shown
/// verbatim in the Settings window, so it is plain language.
pub fn validate_ptt_hotkey(accel: &str) -> Result<(), String> {
    if accel.trim().is_empty() {
        return Err("Press a key to use for push-to-talk.".to_string());
    }
    if ptt_conflict(accel, &ALL_ACCELERATORS) {
        return Err(format!(
            "{accel} is already a capture shortcut — choose a different key."
        ));
    }
    if ptt_keycode(accel).is_none() {
        return Err(format!(
            "{accel} can't be used for push-to-talk. Try a letter, a number, F1–F12, Space, Return, Tab or Escape."
        ));
    }
    Ok(())
}

/// Settings UI entry point: validates a candidate PTT hotkey before it is
/// persisted through `set_settings`.
#[tauri::command]
pub fn validate_hotkey(accel: String) -> Result<(), String> {
    validate_ptt_hotkey(&accel)
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

    #[test]
    fn ptt_keycode_maps_every_required_key_family() {
        for (name, code) in [
            ("F1", 122),
            ("F2", 120),
            ("F3", 99),
            ("F4", 118),
            ("F5", 96),
            ("F6", 97),
            ("F7", 98),
            ("F8", 100),
            ("F9", 101),
            ("F10", 109),
            ("F11", 103),
            ("F12", 111),
            ("Space", 49),
            ("Return", 36),
            ("Tab", 48),
            ("Escape", 53),
        ] {
            assert_eq!(ptt_keycode(name), Some(code), "{name}");
        }
        for c in 'a'..='z' {
            assert!(ptt_keycode(&c.to_string()).is_some(), "letter {c}");
        }
        for c in '0'..='9' {
            assert!(ptt_keycode(&c.to_string()).is_some(), "digit {c}");
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn ptt_keycode_agrees_with_default_listener_keycode() {
        assert_eq!(ptt_keycode("F8"), Some(crate::ptt::DEFAULT_PTT_KEYCODE));
    }

    #[test]
    fn ptt_keycode_ignores_modifiers_and_ui_prefixes() {
        assert_eq!(ptt_keycode("Cmd+Shift+F8"), Some(100));
        assert_eq!(ptt_keycode("Option + Alt + Space"), Some(49));
        assert_eq!(ptt_keycode("Control+KeyR"), Some(15));
        assert_eq!(ptt_keycode("Digit1"), Some(18));
        assert_eq!(ptt_keycode("enter"), Some(36));
        assert_eq!(ptt_keycode("ESC"), Some(53));
    }

    #[test]
    fn ptt_keycode_rejects_unmappable_or_ambiguous_accelerators() {
        assert_eq!(ptt_keycode(""), None);
        assert_eq!(ptt_keycode("Shift"), None);
        assert_eq!(ptt_keycode("Shift+Alt"), None);
        assert_eq!(ptt_keycode("F13"), None);
        assert_eq!(ptt_keycode("MediaPlayPause"), None);
        assert_eq!(ptt_keycode("A+B"), None);
    }

    #[test]
    fn ptt_conflict_matches_every_capture_accelerator() {
        for accel in ALL_ACCELERATORS {
            assert!(ptt_conflict(accel, &ALL_ACCELERATORS), "{accel}");
        }
    }

    #[test]
    fn ptt_conflict_is_case_space_and_order_insensitive() {
        assert!(ptt_conflict("ALT+SHIFT+R", &ALL_ACCELERATORS));
        assert!(ptt_conflict("Alt + Shift + W", &ALL_ACCELERATORS));
        assert!(ptt_conflict("Option+Shift+F", &ALL_ACCELERATORS));
        assert!(ptt_conflict("Shift+Alt+R", &ALL_ACCELERATORS));
    }

    #[test]
    fn ptt_conflict_leaves_free_keys_alone() {
        assert!(!ptt_conflict("F8", &ALL_ACCELERATORS));
        assert!(!ptt_conflict("Cmd+Shift+F8", &ALL_ACCELERATORS));
        assert!(!ptt_conflict("Alt+Shift", &ALL_ACCELERATORS));
        assert!(!ptt_conflict("", &ALL_ACCELERATORS));
    }

    #[test]
    fn validate_accepts_rebindable_keys() {
        assert!(validate_ptt_hotkey("F8").is_ok());
        assert!(validate_ptt_hotkey("Cmd+Shift+Space").is_ok());
        assert!(validate_ptt_hotkey("KeyQ").is_ok());
        assert!(validate_ptt_hotkey("Digit7").is_ok());
    }

    #[test]
    fn validate_rejects_capture_shortcut_conflicts() {
        for accel in ALL_ACCELERATORS {
            let err = validate_ptt_hotkey(accel).expect_err("capture accelerator must conflict");
            assert!(err.contains(accel), "error must name the shortcut: {err}");
        }
        assert!(validate_ptt_hotkey("Option + Shift + W").is_err());
    }

    #[test]
    fn validate_rejects_unmappable_keys_with_plain_reason() {
        for bad in ["", "   ", "F13", "MediaPlayPause", "Shift"] {
            let err = validate_ptt_hotkey(bad).expect_err("must reject");
            assert!(!err.trim().is_empty());
        }
        assert!(validate_ptt_hotkey("F13").unwrap_err().contains("F13"));
    }

    #[test]
    fn validate_hotkey_command_delegates_to_validator() {
        assert_eq!(validate_hotkey("F8".to_string()), Ok(()));
        assert!(validate_hotkey("Alt+Shift+R".to_string()).is_err());
    }
}
