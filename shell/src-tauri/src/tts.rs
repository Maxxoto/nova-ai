//! TTS setup (RFC-0005): system voices as the always-available fallback
//! engine (macOS `say`); the Piper-class primary engine arrives with M3.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemVoice {
    pub name: String,
    pub lang: String,
}

/// Parses `say -v ?` output: `<name ...> <locale>  # comment`. Voice names may
/// contain spaces ("Bad News"), so the locale is matched as the final
/// hyphen/underscore token before the comment.
pub fn parse_say_voices(output: &str) -> Vec<SystemVoice> {
    output
        .lines()
        .filter_map(|line| {
            let without_comment = line.split('#').next()?.trim_end();
            let tokens: Vec<&str> = without_comment.split_whitespace().collect();
            if tokens.len() < 2 {
                return None;
            }
            let locale = tokens[tokens.len() - 1];
            if !locale.contains('-') && !locale.contains('_') {
                return None;
            }
            let name = tokens[..tokens.len() - 1].join(" ");
            if name.is_empty() {
                return None;
            }
            Some(SystemVoice {
                name,
                lang: locale.to_string(),
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn say_voices() -> Vec<SystemVoice> {
    match std::process::Command::new("say").arg("-v").arg("?").output() {
        Ok(output) if output.status.success() => {
            parse_say_voices(&String::from_utf8_lossy(&output.stdout))
        }
        _ => Vec::new(),
    }
}

#[cfg(not(target_os = "macos"))]
fn say_voices() -> Vec<SystemVoice> {
    Vec::new()
}

#[tauri::command]
pub fn tts_list_voices() -> Vec<SystemVoice> {
    say_voices()
}

#[tauri::command]
pub fn tts_test_voice(voice: String, text: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let phrase = if text.trim().is_empty() {
            "Hello from Ruòxī."
        } else {
            text.trim()
        };
        std::process::Command::new("say")
            .arg("-v")
            .arg(&voice)
            .arg(phrase)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("say failed: {e}"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (voice, text);
        Err("unsupported on this platform".to_string())
    }
}

#[tauri::command]
pub fn tts_save_voice(app: tauri::AppHandle, voice: String) -> Result<(), String> {
    let mut current = crate::settings::load(&app);
    current.tts.engine = "system".to_string();
    current.tts.voice = voice;
    crate::settings::save(&app, &current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_locales_and_multiname_voices() {
        let raw = "Alex                en_US    # Most people recognize me by my voice.\nBad News           en_US    # The light you see...\nTingting           zh_CN    # 你好\n";
        let voices = parse_say_voices(raw);
        assert_eq!(voices.len(), 3);
        assert_eq!(voices[0].name, "Alex");
        assert_eq!(voices[0].lang, "en_US");
        assert_eq!(voices[1].name, "Bad News");
        assert_eq!(voices[2].lang, "zh_CN");
    }

    #[test]
    fn junk_lines_are_skipped() {
        let raw = "no locale here\n\nX\n";
        assert!(parse_say_voices(raw).is_empty());
    }
}
