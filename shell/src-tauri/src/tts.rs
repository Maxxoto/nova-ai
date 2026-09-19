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
    current.tts.voice = voice;
    crate::settings::save(&app, &current)
}

#[tauri::command]
pub fn tts_save_engine(app: tauri::AppHandle, engine: String) -> Result<(), String> {
    if engine != "system" && engine != "kokoro" {
        return Err(format!("unknown engine: {engine}"));
    }
    let mut current = crate::settings::load(&app);
    current.tts.engine = engine;
    crate::settings::save(&app, &current)
}

static KOKORO: tokio::sync::OnceCell<kokoro_micro::TtsEngine> = tokio::sync::OnceCell::const_new();

/// Loads (and caches) the Kokoro engine from the app-data models. Present
/// files mean its own downloader never runs — ours owns the artifacts.
async fn kokoro_engine(app: &tauri::AppHandle) -> Result<&'static kokoro_micro::TtsEngine, String> {
    let settings = crate::settings::load(app);
    let dir = crate::models::models_dir(app)?;
    let model_id = if settings.tts.model.is_empty() {
        "kokoro-onnx-fp32"
    } else {
        settings.tts.model.as_str()
    };
    let model_file = crate::models::any_entry(model_id)
        .filter(|m| m.id.starts_with("kokoro-onnx"))
        .ok_or_else(|| format!("unknown kokoro model: {model_id}"))?
        .file;
    let voices_file = crate::models::any_entry("kokoro-voices")
        .ok_or("kokoro voicepacks missing from catalog")?
        .file;
    let model_path = dir.join(model_file);
    let voices_path = dir.join(voices_file);
    if !model_path.is_file() || !voices_path.is_file() {
        return Err("Kokoro not downloaded — Settings → Voice output".to_string());
    }
    KOKORO
        .get_or_try_init(|| async {
            kokoro_micro::TtsEngine::with_paths(
                model_path.to_str().unwrap_or_default(),
                voices_path.to_str().unwrap_or_default(),
            )
            .await
        })
        .await
}

#[tauri::command]
pub async fn tts_kokoro_voices(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    Ok(kokoro_engine(&app).await?.voices())
}

#[tauri::command]
pub async fn tts_synthesize(
    app: tauri::AppHandle,
    text: String,
    voice: String,
) -> Result<(), String> {
    let engine = kokoro_engine(&app).await?;
    let phrase = if text.trim().is_empty() {
        "Hello from Ruòxī.".to_string()
    } else {
        text.trim().to_string()
    };
    let samples = tokio::task::spawn_blocking(move || {
        engine.synthesize_with_options(&phrase, Some(&voice), 1.0, 1.0, None)
    })
    .await
    .map_err(|e| format!("synthesis task failed: {e}"))?
    .map_err(|e| format!("synthesis failed: {e}"))?;
    std::thread::spawn(move || {
        if let Err(e) = play_samples(samples) {
            eprintln!("ruoxi: playback failed: {e}");
        }
    });
    Ok(())
}

fn play_samples(samples: Vec<f32>) -> Result<(), String> {
    use rodio::{buffer::SamplesBuffer, DeviceSinkBuilder, Player};
    let sink = DeviceSinkBuilder::open_default_sink().map_err(|e| format!("audio device: {e}"))?;
    let player = Player::connect_new(&sink.mixer());
    let channels = std::num::NonZeroU16::new(1).expect("nonzero channels");
    let rate = std::num::NonZeroU32::new(24_000).expect("nonzero rate");
    player.append(SamplesBuffer::new(channels, rate, samples));
    player.sleep_until_end();
    Ok(())
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
