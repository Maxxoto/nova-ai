//! Brain link (M0 wiring): the bridge between Tauri commands and the
//! supervised sidecar pump. Commands queue `BrainRequest`s here; the
//! supervisor (`crate::supervisor`) takes the receiver once and writes them
//! to the sidecar, streaming `session.ask` back as panel events.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct BrainRequest {
    pub method: String,
    pub params: Value,
    /// Set for request/response round-trips (e.g. `config.test`); `None`
    /// leaves handling to the pump (asks stream as panel events).
    pub reply: Option<std::sync::mpsc::Sender<Result<Value, String>>>,
}

#[derive(Clone)]
pub struct BrainLink {
    tx: Arc<UnboundedSender<BrainRequest>>,
    online: Arc<AtomicBool>,
    receiver: Arc<Mutex<Option<UnboundedReceiver<BrainRequest>>>>,
}

impl BrainLink {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            tx: Arc::new(tx),
            online: Arc::new(AtomicBool::new(false)),
            receiver: Arc::new(Mutex::new(Some(rx))),
        }
    }

    /// Hands the request queue to the supervisor exactly once; `None` on
    /// subsequent calls (the pump never gets a second receiver).
    pub fn take_receiver(&self) -> Option<UnboundedReceiver<BrainRequest>> {
        self.receiver.lock().unwrap().take()
    }

    pub fn set_online(&self, value: bool) {
        self.online.store(value, Ordering::Relaxed);
    }

    pub fn is_online(&self) -> bool {
        self.online.load(Ordering::Relaxed)
    }

    pub fn send(&self, method: &str, params: Value) -> Result<(), String> {
        if !self.is_online() {
            return Err("brain offline".to_string());
        }
        self.tx
            .send(BrainRequest {
                method: method.to_string(),
                params,
                reply: None,
            })
            .map_err(|_| "brain offline".to_string())
    }

    /// Blocking request/response round-trip through the sidecar pump.
    pub fn rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let (reply, inbox) = std::sync::mpsc::channel();
        self.tx
            .send(BrainRequest {
                method: method.to_string(),
                params,
                reply: Some(reply),
            })
            .map_err(|_| "brain offline".to_string())?;
        inbox
            .recv_timeout(std::time::Duration::from_secs(20))
            .map_err(|_| "request timed out".to_string())?
    }
}

#[tauri::command]
pub fn session_ask(
    brain: tauri::State<BrainLink>,
    transcript: String,
    capture_ids: Vec<String>,
) -> Result<(), String> {
    brain.send(
        "session.ask",
        json!({ "transcript": transcript, "capture_ids": capture_ids }),
    )
}

#[tauri::command]
pub fn session_abort(brain: tauri::State<BrainLink>) -> Result<(), String> {
    brain.send("session.abort", json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_link_rejects_requests() {
        let link = BrainLink::new();
        assert_eq!(
            link.send("session.ask", json!({})),
            Err("brain offline".to_string())
        );
    }

    #[test]
    fn online_link_queues_and_yields_once() {
        let link = BrainLink::new();
        link.set_online(true);
        assert!(link.send("session.ask", json!({"transcript": "hi"})).is_ok());
        let mut rx = link.take_receiver().expect("first take");
        assert!(link.take_receiver().is_none());
        let request = rx.try_recv().expect("queued request");
        assert_eq!(request.method, "session.ask");
        assert_eq!(request.params["transcript"], "hi");
    }
}
