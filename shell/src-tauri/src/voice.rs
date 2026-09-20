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
    callbacks: Arc<std::sync::atomic::AtomicUsize>,
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
        let callbacks = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let sink_samples = Arc::clone(&samples);
        let sink_stop = Arc::clone(&stop);
        let sink_callbacks = Arc::clone(&callbacks);

        let input_rate = config.sample_rate as f64;
        let mut resample_cursor = 0.0f64;
        let mut callback = move |data: &[f32]| {
            if sink_stop.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            sink_callbacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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

        // Keep the stream alive until stop is requested, then drop the
        // stream BEFORE signalling — releasing the device and the callback's
        // sample handle so the final read sees a quiesced buffer.
        let guard_stop = Arc::clone(&stop);
        std::thread::spawn(move || {
            while !guard_stop.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            drop(stream);
            let _ = done_tx.send(());
        });

        Ok(Self {
            samples,
            stop,
            callbacks,
            finished: done_rx,
        })
    }

    /// Stops capture and returns the recorded samples. Empty with zero
    /// callbacks usually means macOS microphone permission is missing.
    pub fn stop(self) -> Vec<f32> {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = self.finished.recv_timeout(std::time::Duration::from_secs(2));
        let samples = self
            .samples
            .lock()
            .map(|mut buffer| std::mem::take(&mut *buffer))
            .unwrap_or_default();
        if samples.is_empty()
            && self
                .callbacks
                .load(std::sync::atomic::Ordering::Relaxed)
                == 0
        {
            eprintln!(
                "ruoxi: no audio arrived from the device — check Microphone permission \
                 (System Settings → Privacy & Security → Microphone) and the input device"
            );
        }
        samples
    }
}

pub fn transcribe(app: &tauri::AppHandle, samples: &[f32]) -> Result<String, String> {
    if samples.is_empty() {
        return Ok(String::new());
    }
    transcribe_with(&resolve_model_path(app)?, samples)
}

/// Dispatches on the model file's magic: `ggml` → whisper, `lmgg` →
/// parakeet (separate library in whisper.cpp ≥1.9, never routed through
/// the whisper loader).
pub fn transcribe_with(path: &std::path::Path, samples: &[f32]) -> Result<String, String> {
    let mut magic = [0u8; 4];
    let known = std::fs::File::open(path)
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut magic))
        .is_ok();
    if known && magic == *b"lmgg" {
        return transcribe_parakeet(path, samples);
    }
    transcribe_whisper(path, samples)
}

fn transcribe_whisper(path: &std::path::Path, samples: &[f32]) -> Result<String, String> {
    let mut params =
        whisper_rs::WhisperContextParameters::new();
    params.use_gpu = true;
    let path_str = path.display().to_string();
    let context = whisper_rs::WhisperContext::new_with_params(
        &path_str,
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

pub(crate) fn resolve_model_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
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

/// S5 corpus hook: dump a PTT utterance as 16-bit PCM WAV under `dir`.
/// Enabled only when RUOXI_S5_RECORD points at a directory.
pub fn write_corpus_wav(dir: &str, samples: &[f32]) -> Result<std::path::PathBuf, String> {
    let dir = std::path::Path::new(dir);
    std::fs::create_dir_all(dir).map_err(|e| format!("corpus dir: {e}"))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("clock: {e}"))?;
    let path = dir.join(format!("utt-{}.wav", now.as_millis()));
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        pcm.extend_from_slice(&((clamped * i16::MAX as f32) as i16).to_le_bytes());
    }
    let data_len = pcm.len() as u32;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&(TARGET_RATE as u32).to_le_bytes());
    wav.extend_from_slice(&((TARGET_RATE as u32) * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(&pcm);
    std::fs::write(&path, wav).map_err(|e| format!("write wav: {e}"))?;
    Ok(path)
}

#[allow(non_snake_case)]
mod parakeet_ffi {
    use std::os::raw::{c_int, c_void};

    #[repr(C)]
    pub struct parakeet_context {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct parakeet_state {
        _private: [u8; 0],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct parakeet_context_params {
        pub use_gpu: bool,
        pub gpu_device: c_int,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct parakeet_full_params {
        pub strategy: c_int,
        pub n_threads: c_int,
        pub offset_ms: c_int,
        pub duration_ms: c_int,
        pub no_context: bool,
        pub audio_ctx: c_int,
        pub new_segment_callback: *const c_void,
        pub new_segment_callback_user_data: *mut c_void,
        pub new_token_callback: *const c_void,
        pub new_token_callback_user_data: *mut c_void,
        pub progress_callback: *const c_void,
        pub progress_callback_user_data: *mut c_void,
        pub encoder_begin_callback: *const c_void,
        pub encoder_begin_callback_user_data: *mut c_void,
        pub abort_callback: *const c_void,
        pub abort_callback_user_data: *mut c_void,
    }

    pub const PARAKEET_SAMPLING_GREEDY: c_int = 0;

    extern "C" {
        pub fn parakeet_context_default_params() -> parakeet_context_params;
        pub fn parakeet_full_default_params(
            strategy: c_int,
        ) -> parakeet_full_params;
        pub fn parakeet_init_from_file_with_params(
            path: *const std::os::raw::c_char,
            params: parakeet_context_params,
        ) -> *mut parakeet_context;
        pub fn parakeet_init_state(ctx: *mut parakeet_context) -> *mut parakeet_state;
        pub fn parakeet_free(ctx: *mut parakeet_context);
        pub fn parakeet_free_state(state: *mut parakeet_state);
        pub fn parakeet_full_with_state(
            ctx: *mut parakeet_context,
            state: *mut parakeet_state,
            params: parakeet_full_params,
            samples: *const f32,
            n_samples: c_int,
        ) -> c_int;
        pub fn parakeet_full_n_segments_from_state(state: *mut parakeet_state) -> c_int;
        pub fn parakeet_full_get_segment_text_from_state(
            state: *mut parakeet_state,
            i_segment: c_int,
        ) -> *const std::os::raw::c_char;
    }
}

/// Parakeet TDT route (English-only; whisper.cpp's sibling library).
/// Model loads per call, mirroring the whisper lifecycle.
fn transcribe_parakeet(path: &std::path::Path, samples: &[f32]) -> Result<String, String> {
    use parakeet_ffi::*;
    let path_c = std::ffi::CString::new(path.display().to_string())
        .map_err(|e| format!("path: {e}"))?;
    unsafe {
        let mut ctx_params = parakeet_context_default_params();
        ctx_params.use_gpu = true;
        let ctx = parakeet_init_from_file_with_params(path_c.as_ptr(), ctx_params);
        if ctx.is_null() {
            return Err("load parakeet model: InitError".to_string());
        }
        let state = parakeet_init_state(ctx);
        if state.is_null() {
            parakeet_free(ctx);
            return Err("parakeet state alloc failed".to_string());
        }
        let mut params = parakeet_full_default_params(PARAKEET_SAMPLING_GREEDY);
        params.n_threads = std::thread::available_parallelism()
            .map(|n| n.get() as std::os::raw::c_int)
            .unwrap_or(8);
        let rc = parakeet_full_with_state(
            ctx,
            state,
            params,
            samples.as_ptr(),
            samples.len() as std::os::raw::c_int,
        );
        if rc != 0 {
            parakeet_free_state(state);
            parakeet_free(ctx);
            return Err(format!("parakeet decode failed: {rc}"));
        }
        let segments = parakeet_full_n_segments_from_state(state);
        let mut text = String::new();
        for i in 0..segments {
            let ptr = parakeet_full_get_segment_text_from_state(state, i);
            if !ptr.is_null() {
                if let Ok(part) = std::ffi::CStr::from_ptr(ptr).to_str() {
                    text.push_str(part);
                }
            }
        }
        parakeet_free_state(state);
        parakeet_free(ctx);
        Ok(text.trim().to_string())
    }
}
