//! Brain sidecar supervisor (M0). RFC-0002 §4.7 — spawn, ping health-check,
//! exponential-backoff restart, give up into a degraded tray state.
//!
//! `SidecarProcess` is the transport core (spawn + JSON-RPC over stdio) and is
//! exercised by the cross-language integration tests in `tests/`; `run` is the
//! tray-facing supervision loop.

use std::io;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
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

    let mut sidecar = match SidecarProcess::spawn(&settings) {
        Ok(sidecar) => sidecar,
        Err(e) => {
            eprintln!("ruoxi: brain sidecar failed to spawn: {e}");
            return;
        }
    };
    set_tooltip(app, "Ruoxi — brain connected");

    loop {
        if !sidecar.ping(PING_TIMEOUT).await {
            break;
        }
        match sidecar.next_line(interval).await {
            LineState::Idle => {}
            LineState::Gone => break,
            LineState::Received(_) => {}
        }
    }
    sidecar.kill().await;
}

pub enum LineState {
    /// No output before the deadline passed.
    Idle,
    /// stdout closed or unreadable: the child is gone.
    Gone,
    /// A line arrived within the deadline.
    Received(String),
}

pub struct SidecarProcess {
    child: Child,
    stdin: ChildStdin,
    lines: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    next_id: u64,
}

impl SidecarProcess {
    pub fn spawn(settings: &settings::Settings) -> io::Result<Self> {
        let mut child = Command::new(&settings.sidecar_command)
            .args(&settings.sidecar_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let (stdin, stdout) = match (child.stdin.take(), child.stdout.take()) {
            (Some(stdin), Some(stdout)) => (stdin, stdout),
            _ => {
                return Err(io::Error::other("sidecar stdio was not piped"));
            }
        };
        Ok(Self {
            child,
            stdin,
            lines: BufReader::new(stdout).lines(),
            next_id: 0,
        })
    }

    /// Sends a request and returns the id used for matching the response.
    pub async fn request(&mut self, method: &str, params_json: &str) -> u64 {
        self.next_id += 1;
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"{method}\",\"params\":{params_json}}}\n",
            self.next_id
        );
        let _ = self.stdin.write_all(line.as_bytes()).await;
        self.next_id
    }

    pub async fn ping(&mut self, timeout: Duration) -> bool {
        let id = self.request("ping", "{}").await;
        self.wait_response(id, timeout).await.is_some()
    }

    /// Reads until the response for `want_id` arrives; notifications seen on
    /// the way are skipped. Returns `None` on timeout, EOF, or IO error.
    pub async fn wait_response(
        &mut self,
        want_id: u64,
        timeout: Duration,
    ) -> Option<serde_json::Value> {
        let deadline = Instant::now() + timeout;
        loop {
            let now = Instant::now();
            let remaining = if deadline > now {
                deadline - now
            } else {
                return None;
            };
            match self.next_line(remaining).await {
                LineState::Idle | LineState::Gone => return None,
                LineState::Received(line) => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                        if value.get("id").and_then(|v| v.as_u64()) == Some(want_id) {
                            return Some(value);
                        }
                    }
                }
            }
        }
    }

    pub async fn next_line(&mut self, within: Duration) -> LineState {
        match tokio::time::timeout(within, self.lines.next_line()).await {
            Err(_elapsed) => LineState::Idle,
            Ok(Err(_)) | Ok(Ok(None)) => LineState::Gone,
            Ok(Ok(Some(line))) => LineState::Received(line),
        }
    }

    pub async fn kill(&mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
    }
}

fn set_tooltip(app: &tauri::AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(text));
    }
}
