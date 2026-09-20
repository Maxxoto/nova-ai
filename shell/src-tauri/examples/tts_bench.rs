//! TTS benchmark (RFC-0005): kokoro engine init, cold-vs-warm synthesis,
//! and per-length RTF. Model files come from the app-data models dir.
//!
//! Usage: cargo run --release --example tts_bench

use std::time::Instant;

const RATE: f64 = 24_000.0;

#[tokio::main]
async fn main() {
    let dir = {
        let home = std::env::var("HOME").expect("HOME");
        std::path::PathBuf::from(home).join("Library/Application Support/com.ruoxi.shell/models")
    };
    let model = dir.join("kokoro-tiny.onnx");
    let voices = dir.join("voices-v1.0.bin");
    for (name, path) in [("model", &model), ("voices", &voices)] {
        if !path.is_file() {
            eprintln!("missing {name}: {}", path.display());
            std::process::exit(1);
        }
    }

    let t0 = Instant::now();
    let engine = kokoro_micro::TtsEngine::with_paths(
        model.to_str().unwrap_or_default(),
        voices.to_str().unwrap_or_default(),
    )
    .await
    .expect("engine init");
    println!("engine init (cold): {:>6} ms", t0.elapsed().as_millis());

    let cases: &[(&str, &str)] = &[
        ("short en", "Hello from Ruo."),
        (
            "medium en",
            "The Krebs cycle turns pyruvate into energy carriers through eight enzymatic steps inside the mitochondria.",
        ),
        (
            "long en",
            "Tomorrow looks fairly packed. You have a lunch break scheduled from eleven to twelve, then a short agency tracking review at half past twelve, and the design sync occupies the afternoon block from two until four. After that the evening is mostly free apart from a reminder to review the capture timeline before bed.",
        ),
        ("short zh", "你好，我是若曦。"),
        ("medium zh", "明天上午十一点到十二点是午餐休息，下午两点有设计同步会议。"),
    ];

    for round in 0..2 {
        println!("══ round {} ══", if round == 0 { "cold" } else { "warm" });
        for (label, text) in cases {
            let t1 = Instant::now();
            let samples = engine
                .synthesize_with_options(text, Some("af_heart"), 1.0, 1.0, None)
                .expect("synth");
            let ms = t1.elapsed().as_millis();
            let audio_s = samples.len() as f64 / RATE;
            let rtf = ms as f64 / 1000.0 / audio_s.max(0.001);
            println!(
                "{:<10} {:>3} chars  {:>5} ms  {:>5.1}s audio  RTF {:.2}",
                label,
                text.chars().count(),
                ms,
                audio_s,
                rtf
            );
        }
    }

    let (_, long) = cases[2];
    let first_sentence = long.split('.').next().unwrap_or(long);
    let t2 = Instant::now();
    engine
        .synthesize_with_options(first_sentence, Some("af_heart"), 1.0, 1.0, None)
        .expect("synth");
    println!(
        "chunked time-to-first-audio (first sentence, {} chars): {} ms",
        first_sentence.chars().count(),
        t2.elapsed().as_millis()
    );
}
