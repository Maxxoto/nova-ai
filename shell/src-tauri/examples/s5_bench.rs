//! S5 spike bench: decode latency + accuracy per whisper quant over a corpus.
//! Corpus: 16 kHz mono 16-bit WAVs named utt-*.wav plus optional
//! utt-*.expected.txt sidecars holding what was actually said.
//!
//! Usage: cargo run --release --example s5_bench -- <corpus_dir> <model.ggml>...

use std::time::Instant;

fn read_wav(path: &std::path::Path) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() < 44 || &bytes[..4] != b"RIFF" || &bytes[36..40] != b"data" {
        return Err(format!("{}: not a canonical WAV", path.display()));
    }
    let data_len = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
    let data = &bytes[44..44 + data_len];
    Ok(data
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / i16::MAX as f32)
        .collect())
}

fn levenshtein<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    let (mut prev, mut cur) = ((0..=b.len()).collect::<Vec<_>>(), vec![0usize; b.len() + 1]);
    for (i, x) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            cur[j + 1] = if x == y {
                prev[j]
            } else {
                1 + prev[j].min(cur[j]).min(prev[j + 1])
            };
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

fn normalize_for_cer(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn accuracy(expected: &str, got: &str) -> f64 {
    let zh = expected.chars().any(|c| matches!(c as u32, 0x4E00..=0x9FFF));
    if zh {
        let (e, g) = (normalize_for_cer(expected), normalize_for_cer(got));
        let chars_of = |s: &str| s.chars().collect::<Vec<_>>();
        let dist = levenshtein(&chars_of(&e), &chars_of(&g));
        1.0 - dist as f64 / e.chars().count().max(1) as f64
    } else {
        let (e, g) = (expected.split_whitespace().collect::<Vec<_>>(),
                      got.split_whitespace().collect::<Vec<_>>());
        let dist = levenshtein(&e, &g);
        1.0 - dist as f64 / e.len().max(1) as f64
    }
}

fn percentile(mut v: Vec<f64>, p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx.min(v.len() - 1)]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: s5_bench <corpus_dir> <model.ggml>...");
        std::process::exit(1);
    }
    let corpus = std::path::Path::new(&args[0]);
    let mut wavs: Vec<std::path::PathBuf> = std::fs::read_dir(corpus)
        .map_err(|e| format!("corpus dir: {e}"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("utt-") && n.ends_with(".wav"))
                .unwrap_or(false)
        })
        .collect();
    wavs.sort();
    if wavs.is_empty() {
        eprintln!("no utt-*.wav in {} — record with RUOXI_S5_RECORD first", corpus.display());
        std::process::exit(1);
    }
    println!("corpus: {} utterances", wavs.len());

    for model_arg in &args[1..] {
        let model_path = std::path::PathBuf::from(model_arg);
        let name = model_path.file_stem().unwrap_or_default().to_string_lossy();
        let t0 = Instant::now();
        let mut params = whisper_rs::WhisperContextParameters::new();
        params.use_gpu = true;
        let context = match whisper_rs::WhisperContext::new_with_params(
            &model_path.display().to_string(),
            params,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}: load failed: {e:?}", name);
                continue;
            }
        };
        println!("══ model {} (load {} ms) ══", name, t0.elapsed().as_millis());

        let mut times = Vec::new();
        let mut scores = Vec::new();
        for wav in &wavs {
            let samples = read_wav(wav).expect("wav");
            let expected = std::fs::read_to_string(wav.with_extension("expected.txt"))
                .map(|s| s.trim().to_string())
                .ok();
            let mut state = context.create_state().expect("state");
            let mut full =
                whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
            full.set_language(Some("auto"));
            full.set_translate(false);
            full.set_print_progress(false);
            full.set_print_special(false);
            full.set_print_realtime(false);
            full.set_print_timestamps(false);
            let t1 = Instant::now();
            state.full(full.clone(), &samples).expect("decode");
            let ms = t1.elapsed().as_millis() as f64;
            times.push(ms);
            let mut text = String::new();
            for i in 0..state.full_n_segments() {
                if let Some(seg) = state.get_segment(i) {
                    if let Ok(t) = seg.to_str() {
                        text.push_str(t);
                    }
                }
            }
            let text = text.trim();
            match expected {
                Some(exp) => {
                    let acc = accuracy(&exp, text);
                    scores.push(acc);
                    println!(
                        "  {:>22}  {:>5.0} ms  acc {:>5.1}%  exp {:?} got {:?}",
                        wav.file_stem().unwrap().to_string_lossy(),
                        ms,
                        acc * 100.0,
                        exp,
                        text
                    );
                }
                None => println!(
                    "  {:>22}  {:>5.0} ms  got {:?}",
                    wav.file_stem().unwrap().to_string_lossy(),
                    ms,
                    text
                ),
            }
        }
        println!(
            "  decode ms: p50 {:.0}  p95 {:.0}  max {:.0}",
            percentile(times.clone(), 0.5),
            percentile(times.clone(), 0.95),
            times.iter().cloned().fold(0.0, f64::max)
        );
        if !scores.is_empty() {
            println!(
                "  accuracy: mean {:.1}%  min {:.1}%  ({} scored)",
                scores.iter().sum::<f64>() / scores.len() as f64 * 100.0,
                scores.iter().cloned().fold(f64::INFINITY, f64::min) * 100.0,
                scores.len()
            );
        }
    }
}
