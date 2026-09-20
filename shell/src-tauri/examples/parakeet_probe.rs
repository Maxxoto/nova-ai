use std::time::Instant;

fn main() {
    let path = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("model path arg"),
    );
    let samples: Vec<f32> = (0..16000)
        .map(|i| {
            (i as f64 * 2.0 * std::f64::consts::PI * 3.0 / 16000.0).sin() as f32 * 0.3
        })
        .collect();
    let t = Instant::now();
    match nova_shell::voice::transcribe_with(&path, &samples) {
        Ok(text) => println!("OK in {} ms — transcript: {:?}", t.elapsed().as_millis(), text),
        Err(e) => println!("ERR in {} ms — {e}", t.elapsed().as_millis()),
    }
}
