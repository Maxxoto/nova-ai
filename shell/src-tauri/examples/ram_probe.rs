//! S2 spike probe: whisper model RAM envelope, mirroring voice.rs params.
//! Usage: cargo run --release --example ram_probe -- [base|small|path-to-ggml]

use std::time::Instant;

fn rss_mb() -> f64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps");
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0)
        / 1024.0
}

fn lcg_noise(seconds: usize) -> Vec<f32> {
    let mut state: u32 = 0x1234_5678;
    (0..seconds * 16_000)
        .map(|i| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state as f64 / u32::MAX as f64 - 0.5) * 0.05;
            let tone = (i as f64 * 440.0 * 2.0 * std::f64::consts::PI / 16_000.0).sin() as f32 * 0.10;
            noise as f32 + tone
        })
        .collect()
}

fn main() {
    let models_dir = {
        let home = std::env::var("HOME").expect("HOME");
        std::path::PathBuf::from(home).join("Library/Application Support/com.ruoxi.shell/models")
    };
    let arg = std::env::args().nth(1).unwrap_or_else(|| "base".into());
    let path = match arg.as_str() {
        "base" => models_dir.join("ggml-base-q5_1.bin"),
        "small" => models_dir.join("ggml-small-q5_1.bin"),
        p => std::path::PathBuf::from(p),
    };
    if !path.exists() {
        eprintln!("model not found: {}", path.display());
        std::process::exit(1);
    }

    println!("model: {}", path.display());
    println!("rss start            : {:>7.1} MB", rss_mb());

    let t0 = Instant::now();
    let mut params = whisper_rs::WhisperContextParameters::new();
    params.use_gpu = true;
    let context = whisper_rs::WhisperContext::new_with_params(&path.display().to_string(), params)
        .expect("load model");
    let load_ms = t0.elapsed().as_millis();
    println!("rss after load       : {:>7.1} MB  (load {load_ms} ms)", rss_mb());

    let mut state = context.create_state().expect("state");
    let mut full = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    full.set_language(Some("auto"));
    full.set_translate(false);
    full.set_print_progress(false);
    full.set_print_special(false);
    full.set_print_realtime(false);
    full.set_print_timestamps(false);

    for seconds in [1usize, 5] {
        let samples = lcg_noise(seconds);
        let t1 = Instant::now();
        state.full(full.clone(), &samples).expect("transcribe");
        let segments = state.full_n_segments();
        println!(
            "rss after {seconds}s pass : {:>7.1} MB  (decode {} ms, {segments} segments)",
            rss_mb(),
            t1.elapsed().as_millis()
        );
    }

    drop(state);
    drop(context);
    println!("rss after drop       : {:>7.1} MB", rss_mb());
}
