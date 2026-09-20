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

/// Words-per-minute `say` speaks at when the rate multiplier is 1.0×
/// (NSSpeechSynthesizer's documented normal pace).
const SAY_BASE_WPM: f64 = 175.0;

#[tauri::command]
pub fn tts_test_voice(app: tauri::AppHandle, voice: String, text: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let phrase = if text.trim().is_empty() {
            "Hello from Ruo."
        } else {
            text.trim()
        };
        let rate = crate::settings::load(&app).tts.rate;
        let wpm = (SAY_BASE_WPM * rate).round().clamp(80.0, 500.0) as u32;
        std::process::Command::new("say")
            .arg("-v")
            .arg(&voice)
            .arg("-r")
            .arg(wpm.to_string())
            .arg(phrase)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("say failed: {e}"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, voice, text);
        Err("unsupported on this platform".to_string())
    }
}

#[tauri::command]
pub fn tts_save_rate(app: tauri::AppHandle, rate: f64) -> Result<(), String> {
    if !rate.is_finite() || !(0.5..=2.0).contains(&rate) {
        return Err(format!("speaking rate out of range: {rate}"));
    }
    let mut current = crate::settings::load(&app);
    current.tts.rate = rate;
    crate::settings::save(&app, &current)
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

enum Speaking {
    System(std::process::Child),
    Kokoro(rodio::MixerDeviceSink),
}

static SPEAK: std::sync::Mutex<Option<Speaking>> = std::sync::Mutex::new(None);
static SPEAK_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Cuts any current speech: kills the `say` child or drops the Kokoro sink
/// (its stream closing is what silences playback).
pub fn stop_speaking() {
    SPEAK_GEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let taken = SPEAK.lock().map(|mut slot| slot.take()).ok().flatten();
    match taken {
        Some(Speaking::System(mut child)) => {
            let _ = child.kill();
        }
        // Closing the sink's stream is what silences Kokoro playback.
        Some(Speaking::Kokoro(sink)) => drop(sink),
        None => {}
    }
}

/// Speaks a completed answer when read-aloud is on (AC-06: Esc/hide stops it).
pub fn speak_answer(app: &tauri::AppHandle, text: &str) {
    let settings = crate::settings::load(app);
    if !settings.read_aloud || text.trim().is_empty() {
        return;
    }
    speak_text(app, text);
}

/// Speaks `text` with the configured engine, rate and voice; any current
/// speech is cut first. Shared by the read-aloud setting and the panel's
/// manual Read aloud button.
pub fn speak_text(app: &tauri::AppHandle, text: &str) {
    let settings = crate::settings::load(app);
    let voice = settings.tts.voice.clone();
    let engine = settings.tts.engine.clone();
    let rate = settings.tts.rate;
    let app = app.clone();
    let phrase = text.to_string();
    stop_speaking();
    let generation = SPEAK_GEN.load(std::sync::atomic::Ordering::SeqCst);
    tauri::async_runtime::spawn(async move {
        match engine.as_str() {
            "kokoro" => {
                let Ok(engine) = kokoro_engine(&app).await else {
                    return;
                };
                let voice = if voice.is_empty() {
                    "af_heart".to_string()
                } else {
                    voice
                };
                let speed = rate as f32;
                let spoken = tokio::task::spawn_blocking(move || {
                    speak_kokoro_chunked(engine, &phrase, &voice, speed, generation)
                })
                .await
                .unwrap_or_else(|e| Err(format!("tts task: {e}")));
                if let Err(e) = spoken {
                    eprintln!("ruoxi: read-aloud playback failed: {e}");
                }
            }
            _ => {
                #[cfg(target_os = "macos")]
                {
                    let wpm = (SAY_BASE_WPM * rate).round().clamp(80.0, 500.0) as u32;
                    if let Ok(child) = std::process::Command::new("say")
                        .arg("-v")
                        .arg(&voice)
                        .arg("-r")
                        .arg(wpm.to_string())
                        .arg(&phrase)
                        .spawn()
                    {
                        if let Ok(mut slot) = SPEAK.lock() {
                            *slot = Some(Speaking::System(child));
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = &voice;
                }
            }
        }
    });
}

#[tauri::command]
pub fn tts_speak_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("nothing to read".to_string());
    }
    speak_text(&app, &text);
    Ok(())
}

#[tauri::command]
pub fn tts_stop() {
    stop_speaking();
}

/// Loads (and caches) the Kokoro engine from the app-data models. Present
/// files mean its own downloader never runs — ours owns the artifacts.
async fn kokoro_engine(app: &tauri::AppHandle) -> Result<&'static kokoro_micro::TtsEngine, String> {
    let settings = crate::settings::load(app);
    let dir = crate::models::models_dir(app)?;
    let model_id = if settings.tts.model.is_empty() {
        crate::models::KOKORO_DEFAULT_ID
    } else {
        settings.tts.model.as_str()
    };
    let model_file = crate::models::any_entry(model_id)
        .filter(|m| m.kind == crate::models::ModelKind::TtsModel)
        .ok_or_else(|| format!("unknown kokoro model: {model_id}"))?
        .file;
    let voices_file = crate::models::any_entry(crate::models::KOKORO_VOICES_ID)
        .ok_or("kokoro voicepacks missing from catalog")?
        .file;
    let model_path = dir.join(model_file);
    let voices_path = dir.join(voices_file);
    if !model_path.is_file() || !voices_path.is_file() {
        return Err("Kokoro not downloaded — Settings → Voice output".to_string());
    }
    let started = std::time::Instant::now();
    let engine = KOKORO
        .get_or_try_init(|| async {
            kokoro_micro::TtsEngine::with_paths(
                model_path.to_str().unwrap_or_default(),
                voices_path.to_str().unwrap_or_default(),
            )
            .await
        })
        .await;
    if started.elapsed().as_millis() > 0 {
        eprintln!(
            "ruoxi: tts engine {} in {} ms",
            if engine.is_ok() { "ready" } else { "failed" },
            started.elapsed().as_millis()
        );
    }
    engine
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KokoroVoice {
    pub id: String,
    pub label: String,
}

/// Kokoro voice ids encode `<language><gender>_<name>` (e.g. `af_heart`,
/// `zf_xiaoni`); the label is derived here so every surface words it the same.
fn voice_label(id: &str) -> String {
    let Some((prefix, name)) = id.split_once('_') else {
        return id.to_string();
    };
    let region = match prefix.chars().next() {
        Some('a') => "US",
        Some('b') => "UK",
        Some('e') => "ES",
        Some('f') => "FR",
        Some('h') => "HI",
        Some('i') => "IT",
        Some('j') => "JA",
        Some('p') => "PT",
        Some('z') => "ZH",
        _ => "",
    };
    let mut chars = name.chars();
    let capitalized = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => return id.to_string(),
    };
    if region.is_empty() {
        capitalized
    } else {
        format!("{capitalized} ({region})")
    }
}

#[tauri::command]
pub async fn tts_kokoro_voices(app: tauri::AppHandle) -> Result<Vec<KokoroVoice>, String> {
    Ok(kokoro_engine(&app)
        .await?
        .voices()
        .into_iter()
        .map(|id| KokoroVoice {
            label: voice_label(&id),
            id,
        })
        .collect())
}

#[tauri::command]
pub fn tts_download_kokoro(app: tauri::AppHandle) -> Result<String, String> {
    crate::models::stt_download(app.clone(), crate::models::KOKORO_DEFAULT_ID.to_string())?;
    crate::models::stt_download(app, crate::models::KOKORO_VOICES_ID.to_string())?;
    Ok(crate::models::KOKORO_DEFAULT_ID.to_string())
}

/// Total download size of the Kokoro bundle (default model + voicepacks) —
/// the install-state rows read this instead of guessing which catalog
/// entries the download covers.
#[tauri::command]
pub fn tts_bundle_bytes() -> u64 {
    [crate::models::KOKORO_DEFAULT_ID, crate::models::KOKORO_VOICES_ID]
        .iter()
        .filter_map(|id| crate::models::any_entry(id))
        .map(|model| model.size_bytes)
        .sum()
}

#[tauri::command]
pub async fn tts_synthesize(
    app: tauri::AppHandle,
    text: String,
    voice: String,
) -> Result<(), String> {
    let engine = kokoro_engine(&app).await?;
    stop_speaking();
    let phrase = if text.trim().is_empty() {
        "Hello from Ruo.".to_string()
    } else {
        text.trim().to_string()
    };
    let speed = crate::settings::load(&app).tts.rate as f32;
    let generation = SPEAK_GEN.load(std::sync::atomic::Ordering::SeqCst);
    tokio::task::spawn_blocking(move || {
        if let Err(e) = speak_kokoro_chunked(engine, &phrase, &voice, speed, generation) {
            eprintln!("ruoxi: playback failed: {e}");
        }
    });
    Ok(())
}

/// Splits text into speakable sentences, keeping the delimiter. CJK and
/// Latin terminators both count; runs of whitespace collapse.
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?' | ';' | '。' | '！' | '？' | '；' | '\n') {
            if !current.trim().is_empty() {
                sentences.push(current.trim().to_string());
            }
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }
    sentences
}

/// Synthesizes sentence-by-sentence and streams into one sink: first audio
/// starts after the first chunk instead of the whole answer. `generation`
/// bails mid-stream when the user cuts speech (AC-06).
fn speak_kokoro_chunked(
    engine: &'static kokoro_micro::TtsEngine,
    text: &str,
    voice: &str,
    speed: f32,
    generation: u64,
) -> Result<(), String> {
    use rodio::{buffer::SamplesBuffer, DeviceSinkBuilder, Player};
    let sink = DeviceSinkBuilder::open_default_sink().map_err(|e| format!("audio device: {e}"))?;
    let player = Player::connect_new(&sink.mixer());
    if let Ok(mut slot) = SPEAK.lock() {
        *slot = Some(Speaking::Kokoro(sink));
    }
    let channels = std::num::NonZeroU16::new(1).expect("nonzero channels");
    let rate = std::num::NonZeroU32::new(24_000).expect("nonzero rate");

    let sentences = split_sentences(text);
    let started = std::time::Instant::now();
    let mut first_audio_logged = false;
    for sentence in &sentences {
        if SPEAK_GEN.load(std::sync::atomic::Ordering::SeqCst) != generation {
            return Ok(());
        }
        let chunk_t0 = std::time::Instant::now();
        let synthesized = engine.synthesize_with_options(
            sentence,
            Some(voice),
            speed,
            1.0,
            None,
        );
        let samples = match synthesized {
            Ok(samples) if !samples.is_empty() => samples,
            _ => continue,
        };
        if !first_audio_logged {
            first_audio_logged = true;
            eprintln!(
                "ruoxi: tts first audio after {} ms ({} sentences queued)",
                started.elapsed().as_millis(),
                sentences.len()
            );
        }
        eprintln!(
            "ruoxi: tts synth: {} chars in {} ms ({:.1}s audio)",
            sentence.chars().count(),
            chunk_t0.elapsed().as_millis(),
            samples.len() as f64 / 24_000.0
        );
        player.append(SamplesBuffer::new(channels, rate, samples));
    }
    player.detach();
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
    fn sentences_split_on_latin_and_cjk_terminators() {
        let parts = split_sentences("Hello there. How are you? Great!\nSecond line; done");
        assert_eq!(
            parts,
            vec![
                "Hello there.",
                "How are you?",
                "Great!",
                "Second line;",
                "done"
            ]
        );
    }

    #[test]
    fn cjk_sentences_split_and_empties_drop() {
        let parts = split_sentences("你好。我是若曦！  \n很好；");
        assert_eq!(parts, vec!["你好。", "我是若曦！", "很好；"]);
    }

    #[test]
    fn junk_lines_are_skipped() {
        let raw = "no locale here\n\nX\n";
        assert!(parse_say_voices(raw).is_empty());
    }

    #[test]
    fn voice_labels_map_language_prefixes() {
        assert_eq!(voice_label("af_heart"), "Heart (US)");
        assert_eq!(voice_label("bf_emma"), "Emma (UK)");
        assert_eq!(voice_label("zf_xiaoni"), "Xiaoni (ZH)");
        assert_eq!(voice_label("jm_kumo"), "Kumo (JA)");
        assert_eq!(voice_label("fallback"), "fallback");
    }
}
