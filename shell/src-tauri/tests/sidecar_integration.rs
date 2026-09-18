//! Cross-language integration tests: the Rust supervisor driving the REAL
//! Python brain sidecar (`python -m app.interfaces.sidecar`) over stdio.
//!
//! Falls back to a `uv` ephemeral environment when the system python lacks
//! pydantic, so the tests run on bare CI boxes too.

use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::time::Duration;

use nova_shell::settings::Settings;
use nova_shell::supervisor::{LineState, SidecarProcess};

const ASK_TIMEOUT: Duration = Duration::from_secs(30);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|shell| shell.parent())
        .map(PathBuf::from)
        .expect("src-tauri sits at <repo>/shell/src-tauri")
}

fn system_python_has_pydantic() -> bool {
    StdCommand::new("python3")
        .args(["-c", "import pydantic"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn test_sidecar_settings() -> Settings {
    std::env::set_var("PYTHONPATH", repo_root().join("src"));
    if system_python_has_pydantic() {
        Settings::default()
    } else {
        Settings {
            sidecar_command: "uv".to_string(),
            sidecar_args: vec![
                "run".to_string(),
                "--no-project".to_string(),
                "--with".to_string(),
                "pydantic".to_string(),
                "python".to_string(),
                "-m".to_string(),
                "app.interfaces.sidecar".to_string(),
            ],
            ..Settings::default()
        }
    }
}

async fn spawn_test_sidecar() -> SidecarProcess {
    let settings = test_sidecar_settings();
    SidecarProcess::spawn(&settings).expect("python sidecar must spawn")
}

#[tokio::test]
async fn python_sidecar_answers_ping() {
    let work = tokio::time::timeout(ASK_TIMEOUT, async {
        let mut sidecar = spawn_test_sidecar().await;
        let alive = sidecar.ping(Duration::from_secs(10)).await;
        sidecar.kill().await;
        alive
    });
    assert!(work.await.is_ok_and(|alive| alive), "ping must round-trip");
}

#[tokio::test]
async fn python_sidecar_streams_tokens_then_final_answer() {
    let work = tokio::time::timeout(ASK_TIMEOUT, async {
        let mut sidecar = spawn_test_sidecar().await;
        let id = sidecar
            .request(
                "session.ask",
                r#"{"transcript":"integration check","capture_ids":[]}"#,
            )
            .await;
        let mut tokens = 0u32;
        loop {
            match sidecar.next_line(Duration::from_secs(10)).await {
                LineState::Received(line) => {
                    let value: serde_json::Value = serde_json::from_str(&line).expect("valid json");
                    if value.get("id").and_then(|v| v.as_u64()) == Some(id) {
                        let answer = value["result"]["answer"].as_str().unwrap_or_default();
                        let answer_id = value["result"]["answer_id"].as_str().unwrap_or_default();
                        sidecar.kill().await;
                        return Some((tokens, answer.to_string(), answer_id.to_string()));
                    }
                    if value.get("method").and_then(|m| m.as_str()) == Some("agent.token") {
                        tokens += 1;
                    }
                }
                _ => {
                    sidecar.kill().await;
                    return None;
                }
            }
        }
    });
    let (tokens, answer, answer_id) = work
        .await
        .expect("ask completes within timeout")
        .expect("final response with streamed tokens");
    assert!(tokens >= 1, "expected at least one agent.token");
    assert!(answer.contains("integration check"));
    assert!(answer_id.starts_with("ans_"));
}

#[tokio::test]
async fn python_sidecar_can_be_respawned_after_kill() {
    let work = tokio::time::timeout(ASK_TIMEOUT, async {
        let mut first = spawn_test_sidecar().await;
        let healthy = first.ping(Duration::from_secs(10)).await;
        first.kill().await;

        let mut second = spawn_test_sidecar().await;
        let healthy_after = second.ping(Duration::from_secs(10)).await;
        second.kill().await;
        (healthy, healthy_after)
    });
    assert!(
        work.await.is_ok_and(|(before, after)| before && after),
        "sidecar must ping before and after respawn"
    );
}
