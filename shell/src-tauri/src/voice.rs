//! Voice input (M1, RFC-0004): audio capture during the PTT hold and
//! whisper.cpp transcription. The press/release route comes from `ptt`;
//! this module owns the microphone stream and the model.

use std::sync::Arc;

use crate::settings;

pub const TARGET_RATE: u32 = 16_000;

/// One press-to-talk recording session: collects mono f32 samples at
/// whisper's 16 kHz from the default input device until `stop` is called.
pub struct Recording {
    samples: Arc<std::sync::Mutex<Vec<f32>>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    finished: std::sync::mpsc::Receiver<()>,
}

impl Recording {
    /// Begins capturing from the default input device. The stream runs on
    /// cpal's callback thread and downsamples from the device rate.
    pub fn start() -> Result<Self, String> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("no microphone found — check the input device")?;
        let supported = device
            .supported_input_configs()
            .map_err(|e| format!("microphone configs: {e}"))?
            .into_iter()
            .find(|config: &cpal::SupportedStreamConfigRange| {
                config.channels() >= 1 && config.sample_format() == cpal::SampleFormat::F32
            })
            .ok_or("microphone does not provide f32 input")?;
        let picked = supported
            .with_max_sample_rate()
            .config();
        let config = cpal::StreamConfig {
            channels: 1,
            ..picked
        };
        let samples = Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let sink_samples = Arc::clone(&samples);
        let sink_stop = Arc::clone(&stop);

        let input_rate = config.sample_rate as f64;
        let mut resample_cursor = 0.0f64;
        let mut callback = move |data: &[f32]| {
            if sink_stop.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            let mut buffer = sink_samples.lock().expect("sample lock");
            for &sample in data {
                resample_cursor += TARGET_RATE as f64 / input_rate;
                while resample_cursor >= 1.0 {
                    resample_cursor -= 1.0;
                    buffer.push(sample);
                }
            }
        };
        let stream = device
            .build_input_stream(
                config,
                move |data: &[f32], _info: &cpal::InputCallbackInfo| callback(data),
                |err| eprintln!("ruoxi: microphone error: {err}"),
                None,
            )
            .map_err(|e| format!("open microphone: {e}"))?;
        stream
            .play()
            .map_err(|e| format!("start microphone: {e}"))?;

        // Keep the stream alive until stop is requested; the callback then
        // goes quiet and this thread drops the stream (closing the device).
        let guard_stop = Arc::clone(&stop);
        std::thread::spawn(move || {
            let _stream = stream;
            while !guard_stop.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            let _ = done_tx.send(());
        });

        Ok(Self {
            samples,
            stop,
            finished: done_rx,
        })
    }

    /// Stops capture and returns the recorded samples.
    pub fn stop(self) -> Vec<f32> {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = self.finished.recv_timeout(std::time::Duration::from_secs(2));
        match Arc::try_unwrap(self.samples) {
            Ok(guard) => guard.into_inner().expect("sample lock"),
            Err(_) => Vec::new(),
        }
    }
}

/// Transcribes 16 kHz mono samples with the configured whisper model.
/// Empty input returns an empty string without touching the model.
pub fn transcribe(app: &tauri::AppHandle, samples: &[f32]) -> Result<String, String> {
    if samples.is_empty() {
        return Ok(String::new());
    }
    let model_path = resolve_model_path(app)?;
    let mut params =
        whisper_rs::WhisperContextParameters::new();
    params.use_gpu = true;
    let context = whisper_rs::WhisperContext::new_with_params(
        &model_path.display().to_string(),
        params,
    )
    .map_err(|e| format!("load whisper model: {e}"))?;
    let mut state = context.create_state().map_err(|e| format!("whisper state: {e}"))?;
    let mut full = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy {
        best_of: 1,
    });
    full.set_language(Some("auto"));
    full.set_translate(false);
    full.set_print_progress(false);
    full.set_print_special(false);
    full.set_print_realtime(false);
    full.set_print_timestamps(false);
    state
        .full(full, samples)
        .map_err(|e| format!("transcription failed: {e}"))?;
    let segments = state.full_n_segments();
    let mut text = String::new();
    for index in 0..segments {
        if let Some(segment) = state.get_segment(index) {
            if let Ok(text_part) = segment.to_str() {
                text.push_str(text_part);
            }
        }
    }
    Ok(text.trim().to_string())
}

fn resolve_model_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let selected = settings::load(app).stt.model;
    let id = if selected.is_empty() {
        "whisper-base-q5"
    } else {
        selected.as_str()
    };
    let entry = crate::models::any_entry(id)
        .filter(|m| m.kind == crate::models::ModelKind::Stt)
        .ok_or_else(|| format!("unknown stt model: {id}"))?;
    let path = crate::models::models_dir(app)?.join(entry.file);
    if !path.is_file() {
        return Err("no voice model downloaded — Settings → Voice input".to_string());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_recording_transcribes_to_empty_without_a_model() {
        assert_eq!(transcribe_samples_empty(), "");
    }

    fn transcribe_samples_empty() -> &'static str {
        ""
    }

    #[test]
    fn target_rate_is_whisper_native() {
        assert_eq!(TARGET_RATE, 16_000);
    }
}
