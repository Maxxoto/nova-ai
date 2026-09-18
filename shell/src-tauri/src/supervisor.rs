//! Brain sidecar supervisor (M0). RFC-0002 §4.7 — spawn, ping health-check,
//! exponential-backoff restart, give up into a degraded tray state.

use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{sleep, Instant};

use crate::settings;

const PING_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_RESTARTS: u32 = 5;

pub async fn run(app: tauri::AppHandle) {
    let mut restarts: u32 = 0;
    loop {
        supervise_once(&app).await;
        restarts += 1;
        if restarts > MAX_RESTARTS {
            set_tooltip(&app, "Ruoxi — brain offline (restart app)");
            return;
        }
        let backoff_secs = 1u64 << (restarts - 1).min(5);
        sleep(Duration::from_secs(backoff_secs)).await;
    }
}

async fn supervise_once(app: &tauri::AppHandle) {
    let settings = settings::load(app);
    let interval = Duration::from_secs(settings.ping_interval_secs.max(1));

    let mut child = match spawn(&settings) {
        Ok(child) => child,
        Err(e) => {
            eprintln!("ruoxi: brain sidecar failed to spawn: {e}");
            return;
        }
    };
    set_tooltip(app, "Ruoxi — brain connected");

    let (mut stdin, stdout) = match (child.stdin.take(), child.stdout.take()) {
        (Some(stdin), Some(stdout)) => (stdin, stdout),
        _ => {
            eprintln!("ruoxi: brain sidecar stdio was not piped");
            let _ = child.kill().await;
            return;
        }
    };
    let mut lines = BufReader::new(stdout).lines();

    let mut next_ping = Instant::now() + interval;
    let mut ping_id: u64 = 0;

    loop {
        let now = Instant::now();
        if now >= next_ping {
            ping_id += 1;
            let request = format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{ping_id},\"method\":\"ping\",\"params\":{{}}}}\n"
            );
            if stdin.write_all(request.as_bytes()).await.is_err() {
                break;
            }
            if !wait_pong(&mut lines, ping_id).await {
                let _ = child.kill().await;
                break;
            }
            next_ping = Instant::now() + interval;
        } else {
            let until_ping = next_ping - now;
            match next_line(&mut lines, until_ping).await {
                LineState::Idle => {}
                LineState::Gone => break,
                LineState::Received(_) => {}
            }
        }
    }
    let _ = child.wait().await;
}

enum LineState {
    /// No output before the deadline passed.
    Idle,
    /// stdout closed or unreadable: the child is gone.
    Gone,
    /// A line arrived within the deadline.
    Received(String),
}

async fn next_line(
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    within: Duration,
) -> LineState {
    match tokio::time::timeout(within, lines.next_line()).await {
        Err(_elapsed) => LineState::Idle,
        Ok(Err(_)) | Ok(Ok(None)) => LineState::Gone,
        Ok(Ok(Some(line))) => LineState::Received(line),
    }
}

async fn wait_pong(
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    want_id: u64,
) -> bool {
    let deadline = Instant::now() + PING_TIMEOUT;
    loop {
        let now = Instant::now();
        let remaining = if deadline > now {
            deadline - now
        } else {
            return false;
        };
        match next_line(lines, remaining).await {
            LineState::Idle | LineState::Gone => return false,
            LineState::Received(line) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    if value.get("id").and_then(|v| v.as_u64()) == Some(want_id) {
                        return true;
                    }
                }
            }
        }
    }
}

fn spawn(settings: &settings::Settings) -> std::io::Result<Child> {
    Command::new(&settings.sidecar_command)
        .args(&settings.sidecar_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
}

fn set_tooltip(app: &tauri::AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(text));
    }
}
