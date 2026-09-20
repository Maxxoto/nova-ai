//! Brain sidecar supervisor (M0). RFC-0002 §4.7 — spawn, ping health-check,
//! exponential-backoff restart, give up into a degraded tray state.
//!
//! `SidecarProcess` is the transport core (spawn + JSON-RPC over stdio) and is
//! exercised by the cross-language integration tests in `tests/`; `run` is the
//! tray-facing pump: it pings on a cadence and writes requests queued by the
//! Tauri commands (`crate::brain`), streaming `session.ask` back as panel events.

use std::collections::VecDeque;
use std::io;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{sleep, Instant};

use crate::brain::{BrainLink, BrainRequest};
use crate::settings;
use crate::RequestRouter;

const PING_TIMEOUT: Duration = Duration::from_secs(3);
const ASK_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESTARTS: u32 = 5;

pub async fn run(app: tauri::AppHandle, link: BrainLink) {
    let mut restarts: u32 = 0;
    // Taken once so queued requests survive sidecar respawns.
    let mut requests = link.take_receiver();
    loop {
        supervise_once(&app, &link, &mut requests).await;
        restarts += 1;
        if restarts > MAX_RESTARTS {
            set_tooltip(&app, "Ruoxi — resting (brain offline — restart app)");
            return;
        }
        let backoff_secs = 1u64 << (restarts - 1).min(5);
        sleep(Duration::from_secs(backoff_secs)).await;
    }
}

/// One sidecar generation: spawn, pump until it dies, then kill it.
async fn supervise_once(
    app: &tauri::AppHandle,
    link: &BrainLink,
    requests: &mut Option<UnboundedReceiver<BrainRequest>>,
) {
    let settings = settings::load(app);
    let interval = Duration::from_secs(settings.ping_interval_secs.max(1));

    link.set_online(false);
    let mut sidecar = match SidecarProcess::spawn(&settings) {
        Ok(sidecar) => match capture_router(app) {
            Some(router) => sidecar.with_router(router),
            None => sidecar,
        },
        Err(e) => {
            eprintln!("ruoxi: brain sidecar failed to spawn: {e}");
            return;
        }
    };
    link.set_online(true);
    set_tooltip(app, "Ruoxi — brain connected");

    let mut ping = tokio::time::interval(interval);
    ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut deferred: VecDeque<BrainRequest> = VecDeque::new();

    loop {
        let event = match deferred.pop_front() {
            Some(request) => PumpEvent::Request(request),
            None => tokio::select! {
                _ = ping.tick() => PumpEvent::Ping,
                maybe = recv(requests) => match maybe {
                    Some(request) => PumpEvent::Request(request),
                    None => PumpEvent::Closed,
                },
            },
        };

        let alive = match event {
            PumpEvent::Ping => sidecar.ping(PING_TIMEOUT).await,
            PumpEvent::Closed => false,
            PumpEvent::Request(request) => {
                handle_request(app, &mut sidecar, requests, &mut deferred, request).await
            }
        };
        if !alive {
            break;
        }
    }

    link.set_online(false);
    sidecar.kill().await;
}

enum PumpEvent {
    Ping,
    Request(BrainRequest),
    Closed,
}

/// Writes one queued request; `session.ask` streams back as panel events.
/// Returns `false` once the sidecar is gone and supervision should restart.
async fn handle_request(
    app: &tauri::AppHandle,
    sidecar: &mut SidecarProcess,
    requests: &mut Option<UnboundedReceiver<BrainRequest>>,
    deferred: &mut VecDeque<BrainRequest>,
    request: BrainRequest,
) -> bool {
    if let Some(reply) = request.reply {
        let id = sidecar
            .request(&request.method, &request.params.to_string())
            .await;
        let response = sidecar.wait_response(id, ASK_TIMEOUT).await;
        let outcome = match response {
            Some(value) => {
                if let Some(result) = value.get("result") {
                    Ok(result.clone())
                } else {
                    Err(value
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .unwrap_or("brain error")
                        .to_string())
                }
            }
            None => Err("brain request timed out".to_string()),
        };
        let _ = reply.send(outcome);
        true
    } else if request.method == "session.ask" {
        stream_ask(app, sidecar, requests, deferred, &request.params).await
    } else {
        let _ = sidecar
            .request(&request.method, &request.params.to_string())
            .await;
        true
    }
}

/// Writes `session.ask` then pumps its stream: `agent.token` → `panel:token`,
/// the final response → `panel:complete`, errors/timeouts → `panel:error`.
/// Aborts queued meanwhile are written immediately; other requests defer until
/// the ask ends.
async fn stream_ask(
    app: &tauri::AppHandle,
    sidecar: &mut SidecarProcess,
    requests: &mut Option<UnboundedReceiver<BrainRequest>>,
    deferred: &mut VecDeque<BrainRequest>,
    params: &Value,
) -> bool {
    let ask_id = sidecar.request("session.ask", &params.to_string()).await;
    let deadline = Instant::now() + ASK_TIMEOUT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            emit(app, "panel:error", serde_json::json!({ "message": "brain timed out" }));
            return true;
        }
        let event = tokio::select! {
            state = sidecar.next_message(remaining) => AskEvent::Line(state),
            maybe = recv(requests) => match maybe {
                Some(request) if request.method == "session.abort" => AskEvent::Abort(request),
                Some(request) => AskEvent::Defer(request),
                None => AskEvent::Closed,
            },
        };
        match event {
            AskEvent::Line(LineState::Gone) => {
                emit(app, "panel:error", serde_json::json!({ "message": "brain restarted" }));
                return false;
            }
            AskEvent::Line(LineState::Idle) => {
                emit(app, "panel:error", serde_json::json!({ "message": "brain timed out" }));
                return true;
            }
            AskEvent::Line(LineState::Received(line)) => match classify_ask_line(&line, ask_id) {
                AskSignal::Token(delta) => {
                    emit(app, "panel:token", serde_json::json!({ "delta": delta }))
                }
                AskSignal::ToolStep(step, of, tool) => {
                    emit(
                        app,
                        "panel:tool_step",
                        serde_json::json!({ "step": step, "of": of, "tool": tool }),
                    );
                }
                AskSignal::Complete(answer) => {
                    emit(app, "panel:complete", serde_json::json!({ "answer": answer }));
                    crate::tts::speak_answer(app, &answer);
                    log_episodic(app, params, &answer);
                    return true;
                }
                AskSignal::Failed(message) => {
                    emit(app, "panel:error", serde_json::json!({ "message": message }));
                    return true;
                }
                AskSignal::Ignored => {}
            },
            AskEvent::Abort(request) => {
                let _ = sidecar
                    .request(&request.method, &request.params.to_string())
                    .await;
            }
            AskEvent::Defer(request) => deferred.push_back(request),
            AskEvent::Closed => return true,
        }
    }
}

enum AskEvent {
    Line(LineState),
    Abort(BrainRequest),
    Defer(BrainRequest),
    Closed,
}

#[derive(Debug, PartialEq, Eq)]
enum AskSignal {
    Token(String),
    ToolStep(u32, u32, String),
    Complete(String),
    Failed(String),
    Ignored,
}

/// Classifies one sidecar line seen while waiting for ask `ask_id`.
fn classify_ask_line(line: &str, ask_id: u64) -> AskSignal {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return AskSignal::Ignored;
    };
    if value.get("id").and_then(Value::as_u64) == Some(ask_id) {
        if let Some(result) = value.get("result") {
            let answer = result
                .get("answer")
                .and_then(Value::as_str)
                .unwrap_or_default();
            return AskSignal::Complete(answer.to_string());
        }
        let message = value
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("brain error");
        return AskSignal::Failed(message.to_string());
    }
    let method = value.get("method").and_then(Value::as_str);
    if method == Some("agent.token") {
        if let Some(delta) = value.pointer("/params/delta").and_then(Value::as_str) {
            return AskSignal::Token(delta.to_string());
        }
    }
    if method == Some("agent.tool_step") {
        let step = value.pointer("/params/step").and_then(Value::as_u64).unwrap_or(0) as u32;
        let of = value.pointer("/params/of").and_then(Value::as_u64).unwrap_or(3) as u32;
        let tool = value
            .pointer("/params/tool")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        return AskSignal::ToolStep(step, of, tool);
    }
    AskSignal::Ignored
}

/// Awaits the next queued request, or stays pending forever when the supervisor
/// never received the link.
async fn recv(rx: &mut Option<UnboundedReceiver<BrainRequest>>) -> Option<BrainRequest> {
    match rx {
        Some(rx) => rx.recv().await,
        None => std::future::pending().await,
    }
}

fn capture_router(app: &tauri::AppHandle) -> Option<Arc<dyn RequestRouter>> {
    use tauri::Manager;
    let dir = app.path().app_data_dir().ok()?;
    let store = crate::capture_store::CaptureStore::open(&dir).ok()?;
    let memory = crate::memory::MemoryStore::open(&dir).ok()?;
    Some(Arc::new(ChainedRouter(vec![
        Arc::new(crate::capture_store::CaptureRouter::new(store)),
        Arc::new(crate::memory::MemoryRouter::new(Arc::new(memory))),
    ])))
}

/// Tries each router in order; the first that knows the method answers.
struct ChainedRouter(Vec<Arc<dyn RequestRouter>>);

impl RequestRouter for ChainedRouter {
    fn route(&self, method: &str, params: &Value) -> Result<Value, String> {
        for router in &self.0 {
            if router.handles(method) {
                return router.route(method, params);
            }
        }
        Err(format!("unknown method: {method}"))
    }
}

/// Episodic auto-log (RFC-0006 §4.3): every completed ask becomes a diary
/// entry referencing its captures. Never agent-writable.
fn log_episodic(app: &tauri::AppHandle, params: &Value, answer: &str) {
    use tauri::Manager;
    let Ok(dir) = app.path().app_data_dir() else {
        return;
    };
    let Ok(store) = crate::memory::MemoryStore::open(&dir) else {
        return;
    };
    let question = params
        .get("transcript")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let captures = params
        .get("capture_ids")
        .and_then(|v| v.as_array())
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let mut entry = crate::memory::MemoryEntry::new(
        crate::memory::MemoryType::Episodic,
        format!("Q: {question}\nA: {answer}"),
    );
    entry.source_refs = captures;
    if let Err(e) = store.write(&entry) {
        eprintln!("ruoxi: episodic log failed: {e}");
    }
}

fn emit(app: &tauri::AppHandle, event: &str, payload: Value) {
    if let Err(e) = app.emit(event, payload) {
        eprintln!("ruoxi: {event} emit failed: {e}");
    }
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
    router: Option<Arc<dyn RequestRouter>>,
}

impl SidecarProcess {
    pub fn spawn(settings: &settings::Settings) -> io::Result<Self> {
        let mut child = Command::new(&settings.sidecar_command)
            .args(&settings.sidecar_args)
            .envs(crate::llm::env_for_sidecar(settings))
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
            router: None,
        })
    }

    pub fn with_router(mut self, router: Arc<dyn RequestRouter>) -> Self {
        self.router = Some(router);
        self
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
            match self.next_message(remaining).await {
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

    /// Yields the next non-request line; inbound JSON-RPC requests from the
    /// brain are routed and answered inline, never surfaced to the caller.
    pub async fn next_message(&mut self, within: Duration) -> LineState {
        loop {
            match self.next_line(within).await {
                LineState::Idle => return LineState::Idle,
                LineState::Gone => return LineState::Gone,
                LineState::Received(line) => {
                    if !self.answer_if_request(&line).await {
                        return LineState::Received(line);
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

    async fn answer_if_request(&mut self, line: &str) -> bool {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            return false;
        };
        if value.get("method").is_none() || value.get("id").is_none() {
            return false;
        }
        let method = value["method"].as_str().unwrap_or_default().to_string();
        let params = value.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let id = value["id"].clone();

        let outcome = match self.router.as_ref() {
            Some(router) => router.route(&method, &params).map_err(|message| {
                serde_json::json!({ "code": -32000, "message": message })
            }),
            None => Err(serde_json::json!({
                "code": -32601,
                "message": format!("unknown method: {method}")
            })),
        };

        let response = match outcome {
            Ok(result) => serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(error) => serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": error }),
        };
        let _ = self
            .stdin
            .write_all(format!("{response}\n").as_bytes())
            .await;
        true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_notification_is_extracted() {
        let line = r#"{"jsonrpc":"2.0","method":"agent.token","params":{"answer_id":"ans_1","delta":"hi "}}"#;
        assert_eq!(
            classify_ask_line(line, 7),
            AskSignal::Token("hi ".to_string())
        );
    }

    #[test]
    fn matching_response_is_complete_and_errors_are_failed() {
        let done = r#"{"jsonrpc":"2.0","id":7,"result":{"answer_id":"ans_1","answer":"done","steps":[],"aborted":false}}"#;
        assert_eq!(
            classify_ask_line(done, 7),
            AskSignal::Complete("done".to_string())
        );
        let aborted = r#"{"jsonrpc":"2.0","id":7,"error":{"code":-32000,"message":"aborted by user"}}"#;
        assert_eq!(
            classify_ask_line(aborted, 7),
            AskSignal::Failed("aborted by user".to_string())
        );
    }

    #[test]
    fn other_ids_and_junk_are_ignored() {
        let other = r#"{"jsonrpc":"2.0","id":8,"result":{"answer":"nope"}}"#;
        assert_eq!(classify_ask_line(other, 7), AskSignal::Ignored);
        assert_eq!(classify_ask_line("not json", 7), AskSignal::Ignored);
        let other_notification = r#"{"jsonrpc":"2.0","method":"agent.other","params":{}}"#;
        assert_eq!(
            classify_ask_line(other_notification, 7),
            AskSignal::Ignored
        );
    }
}
