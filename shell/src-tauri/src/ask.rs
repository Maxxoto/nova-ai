//! Capture-scoped voice ask (M2, RFC-0002 §4.4): while the result panel is
//! visible, push-to-talk asks about the capture on screen instead of opening
//! a fresh global question. Owns the one microphone slot both routes record
//! through — the global PTT consumer and the panel's ask commands — so the
//! input device can never be opened twice, plus the capture the panel shows.
//!
//! Event contract (Rust → UI): `panel:listening` when the ask recording
//! starts, `panel:transcribing` on release, `panel:transcript` with the STT
//! text just before `session.ask`, and `panel:ask_cancelled` whenever the ask
//! is dropped (Esc, hide, or empty speech).

use std::sync::Mutex;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::capture_store::CaptureRecord;

/// The capture the panel is showing; the ask is scoped to it. `scope` uses
/// the wire labels (`region` | `window` | `screen`), not the store's
/// `fullscreen` spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentCapture {
    pub id: String,
    pub scope: String,
    pub width: u32,
    pub height: u32,
}

/// The capture the next ask attaches, set next to the `panel:capture` emit
/// and cleared by `panel::hide` so a dismissed panel never scopes an ask to a
/// stale id.
pub static CURRENT_CAPTURE: Mutex<Option<CurrentCapture>> = Mutex::new(None);

/// Shared recorder owner: `Some` while a recording (ask or global PTT) is in
/// flight, `None` otherwise. Both PTT routes take and stop through this slot.
pub static ASK_RECORDER: Mutex<Option<crate::voice::Recording>> = Mutex::new(None);

/// Explicit ask lifecycle. `Transcribing` covers `recording.stop()` + the
/// whisper call — the window where a cancel must suppress a late result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Recording,
    Transcribing,
}

#[derive(Debug, Clone, Copy)]
struct AskState {
    phase: Phase,
    generation: u64,
}

/// One mutex guards phase + generation, so the PTT release, the Esc tap, the
/// hide path and the commands all read a single consistent value (the old
/// separate `ASK_ACTIVE` atomic allowed a release/cancel TOCTOU).
static ASK_STATE: Mutex<AskState> = Mutex::new(AskState {
    phase: Phase::Idle,
    generation: 0,
});

fn ask_slot() -> std::sync::MutexGuard<'static, AskState> {
    // A poisoned lock still owns a usable state; recover instead of panic.
    ASK_STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// `true` while an ask recording or transcription is in flight — the gate the
/// PTT release path and the Esc tap read.
pub fn is_active() -> bool {
    ask_slot().phase != Phase::Idle
}

/// Idle → Recording, bumping the generation. Pure, so tests pin the
/// cancel/finish interplay without a microphone or an app handle.
fn claim_ask(state: &mut AskState) -> Result<u64, String> {
    if state.phase != Phase::Idle {
        return Err("an ask is already in flight".to_string());
    }
    state.generation = state.generation.wrapping_add(1);
    state.phase = Phase::Recording;
    Ok(state.generation)
}

/// Recording → Transcribing; `Err` when no recording was claimed.
fn begin_transcribing(state: &mut AskState) -> Result<u64, String> {
    if state.phase != Phase::Recording {
        return Err("no active voice ask".to_string());
    }
    state.phase = Phase::Transcribing;
    Ok(state.generation)
}

/// Cancel/discard: bump the generation so an in-flight finish suppresses
/// itself, and report whether an ask was actually in flight.
fn cancel_state(state: &mut AskState) -> bool {
    let was_active = state.phase != Phase::Idle;
    state.generation = state.generation.wrapping_add(1);
    state.phase = Phase::Idle;
    was_active
}

/// Set Idle only when `generation` is still live, so a stale finish never
/// clobbers a newer ask.
fn settle(state: &mut AskState, generation: u64) {
    if state.generation == generation {
        state.phase = Phase::Idle;
    }
}

/// Whether the finish that owns `generation` may still emit its result.
fn generation_live(generation: u64) -> bool {
    ask_slot().generation == generation
}

/// Store-scope → wire-scope: the capture store says `fullscreen`, the panel
/// event contract says `screen`. Other scopes pass through unchanged.
pub fn scope_label(store_scope: &str) -> &str {
    match store_scope {
        "fullscreen" => "screen",
        other => other,
    }
}

fn px_dim(value: i64) -> u32 {
    value.clamp(0, u32::MAX as i64) as u32
}

/// The exact `panel:capture` payload (contract-frozen keys): the record's id
/// and timestamp plus the scope and pixel size the ask route needs.
pub fn capture_event(record: &CaptureRecord) -> Value {
    json!({
        "id": &record.capture_id,
        "at_ms": record.ts,
        "scope": scope_label(&record.scope),
        "width": px_dim(record.w_px),
        "height": px_dim(record.h_px),
    })
}

fn capture_slot() -> std::sync::MutexGuard<'static, Option<CurrentCapture>> {
    // A poisoned lock still owns a usable `Option`; recover instead of panic.
    CURRENT_CAPTURE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Remembers `record` as the capture the panel is asking about.
pub fn set_current_capture(record: &CaptureRecord) {
    *capture_slot() = Some(CurrentCapture {
        id: record.capture_id.clone(),
        scope: scope_label(&record.scope).to_string(),
        width: px_dim(record.w_px),
        height: px_dim(record.h_px),
    });
}

pub fn clear_current_capture() {
    *capture_slot() = None;
}

pub fn current_capture() -> Option<CurrentCapture> {
    capture_slot().clone()
}

/// `session.ask` params: the current capture id when one is set, `[]` when
/// the panel has none (RFC-0002: an unscoped ask is still a valid ask).
pub fn ask_params(transcript: &str) -> Value {
    let capture_ids: Vec<String> = current_capture().map(|c| vec![c.id]).unwrap_or_default();
    json!({ "transcript": transcript, "capture_ids": capture_ids })
}

fn recorder_slot() -> std::sync::MutexGuard<'static, Option<crate::voice::Recording>> {
    // A poisoned lock still owns a usable slot; recover instead of panicking.
    ASK_RECORDER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Opens the shared microphone. Starting while a recording already exists is
/// refused with an error string — never a second stream, never a panic.
pub fn start_recording() -> Result<(), String> {
    let mut slot = recorder_slot();
    if slot.is_some() {
        return Err("microphone is already recording".to_string());
    }
    *slot = Some(crate::voice::Recording::start()?);
    Ok(())
}

pub fn take_recording() -> Option<crate::voice::Recording> {
    recorder_slot().take()
}

/// Drops a recording without transcribing it. The stream teardown (up to 2 s)
/// runs on a scratch thread so the Esc tap and window threads never block.
pub fn discard_recording() {
    if let Some(recording) = take_recording() {
        std::thread::spawn(move || {
            let _ = recording.stop();
        });
    }
}

fn emit(app: &AppHandle, event: &str, payload: Value) {
    if let Err(e) = app.emit(event, payload) {
        eprintln!("ruoxi: {event} emit failed: {e}");
    }
}

/// PTT press while the panel is visible (and `voice_ask_start`): claim the
/// ask lifecycle, then start the recording and announce it. The phase flips
/// to `Recording` before the microphone opens, and the state lock is held
/// across the open, so a release racing the press cannot miss the recording.
pub fn begin_ask(app: &AppHandle) -> Result<(), String> {
    let mut state = ask_slot();
    claim_ask(&mut state)?;
    if let Err(e) = start_recording() {
        state.phase = Phase::Idle;
        state.generation = state.generation.wrapping_add(1);
        return Err(e);
    }
    drop(state);
    emit(app, "panel:listening", json!({}));
    Ok(())
}

/// PTT release on the ask path: transcribe the recording, surface the text,
/// then hand the question to the brain scoped to the current capture. Once a
/// cancel has bumped the generation, every late emit/send is suppressed.
pub fn finish_ask(app: &AppHandle) -> Result<(), String> {
    let generation = {
        let mut state = ask_slot();
        begin_transcribing(&mut state)?
    };
    let Some(recording) = take_recording() else {
        settle(&mut ask_slot(), generation);
        return Err("no active voice ask recording".to_string());
    };
    emit(app, "panel:transcribing", json!({}));
    let samples = recording.stop();
    eprintln!(
        "ruoxi: ask captured {} samples ({:.1}s)",
        samples.len(),
        samples.len() as f64 / crate::voice::TARGET_RATE as f64
    );

    if !generation_live(generation) {
        eprintln!("ruoxi: ask {generation} cancelled while recording; dropping samples");
        return Ok(());
    }
    if samples.is_empty() {
        let mut state = ask_slot();
        if state.generation != generation {
            eprintln!("ruoxi: ask {generation} cancelled while recording; dropping samples");
            return Ok(());
        }
        eprintln!("ruoxi: ask: no speech detected");
        emit(app, "panel:ask_cancelled", json!({}));
        settle(&mut state, generation);
        return Ok(());
    }
    let transcription = crate::voice::transcribe(app, &samples);
    // The result leaves under the state lock: a cancel either lands before
    // this block (nothing goes out) or after the ask was already sent, so a
    // re-check followed by an unlocked emit cannot lose an Esc race.
    let mut state = ask_slot();
    if state.generation != generation {
        eprintln!("ruoxi: ask {generation} cancelled during transcription; dropping result");
        return Ok(());
    }
    match transcription {
        Ok(transcript) if !transcript.is_empty() => {
            eprintln!("ruoxi: ask transcript: {transcript}");
            emit(app, "panel:transcript", json!({ "text": transcript }));
            let brain = app.state::<crate::brain::BrainLink>();
            if let Err(e) = brain.send("session.ask", ask_params(&transcript)) {
                eprintln!("ruoxi: ask send failed: {e}");
            }
        }
        Ok(_) => {
            eprintln!("ruoxi: ask: no speech detected");
            emit(app, "panel:ask_cancelled", json!({}));
        }
        Err(e) => {
            eprintln!("ruoxi: ask transcription: {e}");
            emit(app, "panel:error", json!({ "message": e }));
        }
    }
    settle(&mut state, generation);
    Ok(())
}

/// Cancels an in-flight ask: bump the generation, drop the recording without
/// transcribing, and tell the panel exactly once. The panel stays visible.
pub fn cancel_ask(app: &AppHandle) {
    let was_active = cancel_state(&mut ask_slot());
    discard_recording();
    if was_active {
        eprintln!("ruoxi: ask cancelled");
        emit(app, "panel:ask_cancelled", json!({}));
    } else {
        eprintln!("ruoxi: ask cancel ignored (idle)");
    }
}

/// Panel hide: drop any ask in flight without emitting. The bumped
/// generation makes a late `finish_ask` a logged no-op.
pub fn discard_ask() {
    cancel_state(&mut ask_slot());
    discard_recording();
}

/// Starts the capture-scoped ask for the panel's mic button. Refuses with a
/// human-readable error when the microphone is denied or the STT model the
/// transcriber resolves is missing, instead of failing silently later.
#[tauri::command]
pub fn voice_ask_start(app: AppHandle) -> Result<(), String> {
    let permissions = crate::permissions::permissions_status()?;
    if matches!(permissions.microphone.as_str(), "denied" | "restricted") {
        return Err("microphone permission denied — grant it in System Settings → \
                    Privacy & Security → Microphone"
            .to_string());
    }
    crate::voice::resolve_model_path(&app)?;
    begin_ask(&app)
}

/// Stops the ask recording, transcribes it, and sends the scoped ask. Runs on
/// the sync threadpool: whisper blocks, and the app must not stall meanwhile.
#[tauri::command(async)]
pub fn voice_ask_stop(app: AppHandle) -> Result<(), String> {
    finish_ask(&app)
}

/// Stops and discards the ask recording without sending anything.
#[tauri::command]
pub fn voice_ask_cancel(app: AppHandle) {
    cancel_ask(&app);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The statics are process-global; serialize the tests that touch the
    /// capture slot or the ask phase.
    static LOCK: Mutex<()> = Mutex::new(());

    fn record(scope: &str, w_px: i64, h_px: i64) -> CaptureRecord {
        CaptureRecord {
            capture_id: "cap_test".to_string(),
            ts: 42,
            day: "1970-01-01".to_string(),
            scope: scope.to_string(),
            display_id: "1".to_string(),
            app: None,
            window_title: None,
            path: "captures/1970/01/01/cap_test.png".to_string(),
            w_px,
            h_px,
            scale: 1.0,
            sha256: "deadbeef".to_string(),
            retention: "default".to_string(),
            auto: false,
        }
    }

    #[test]
    fn capture_event_carries_the_extended_payload() {
        let event = capture_event(&record("region", 640, 480));
        assert_eq!(event["id"], json!("cap_test"));
        assert_eq!(event["at_ms"], json!(42));
        assert_eq!(event["scope"], json!("region"));
        assert_eq!(event["width"], json!(640));
        assert_eq!(event["height"], json!(480));
    }

    #[test]
    fn fullscreen_scope_maps_to_the_wire_screen_label() {
        assert_eq!(scope_label("fullscreen"), "screen");
        assert_eq!(scope_label("window"), "window");
        assert_eq!(scope_label("region"), "region");
    }

    #[test]
    fn negative_dimensions_clamp_to_zero() {
        assert_eq!(px_dim(-3), 0);
        assert_eq!(px_dim(i64::MAX), u32::MAX);
    }

    #[test]
    fn current_capture_scopes_the_ask_and_clears() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_current_capture();
        assert_eq!(ask_params("hi")["capture_ids"], json!([]));

        set_current_capture(&record("window", 10, 20));
        assert_eq!(ask_params("hi")["capture_ids"], json!(["cap_test"]));
        let capture = current_capture().expect("capture is set");
        assert_eq!(capture.scope, "window");
        assert_eq!((capture.width, capture.height), (10, 20));

        clear_current_capture();
        assert!(current_capture().is_none());
        assert_eq!(ask_params("hi")["capture_ids"], json!([]));
    }

    #[test]
    fn poisoned_capture_slot_is_recovered_not_panicked() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_current_capture();
        let _ = std::panic::catch_unwind(|| {
            let _slot = CURRENT_CAPTURE.lock().unwrap();
            panic!("poison the lock on purpose");
        });
        set_current_capture(&record("region", 1, 1));
        assert_eq!(ask_params("hi")["capture_ids"], json!(["cap_test"]));
        clear_current_capture();
    }

    fn fresh_state() -> AskState {
        AskState {
            phase: Phase::Idle,
            generation: 0,
        }
    }

    #[test]
    fn claim_refuses_a_second_ask_and_finish_needs_a_recording() {
        let mut state = fresh_state();
        assert_eq!(claim_ask(&mut state), Ok(1));
        assert_eq!(state.phase, Phase::Recording);
        assert!(
            claim_ask(&mut state).is_err(),
            "a second claim must be refused"
        );
        assert_eq!(begin_transcribing(&mut state), Ok(1));
        assert!(
            begin_transcribing(&mut state).is_err(),
            "finishing twice must be refused"
        );
    }

    #[test]
    fn cancel_suppresses_a_late_finish_and_a_stale_settle() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        *ask_slot() = fresh_state();

        let generation = claim_ask(&mut ask_slot()).expect("claim");
        assert_eq!(begin_transcribing(&mut ask_slot()), Ok(generation));

        assert!(cancel_state(&mut ask_slot()), "an in-flight ask cancels");
        assert!(!generation_live(generation), "a late finish must not emit");
        assert!(!is_active());

        settle(&mut ask_slot(), generation);
        assert_eq!(ask_slot().phase, Phase::Idle);

        let newer = claim_ask(&mut ask_slot()).expect("newer claim");
        assert_eq!(newer, generation + 2);
        settle(&mut ask_slot(), generation);
        assert_eq!(
            ask_slot().phase,
            Phase::Recording,
            "a stale settle must not clobber the newer ask"
        );
        assert!(cancel_state(&mut ask_slot()));
    }

    #[test]
    fn cancel_is_idempotent_and_the_generation_keeps_moving() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        *ask_slot() = fresh_state();
        assert!(
            !cancel_state(&mut ask_slot()),
            "idle cancel reports nothing to cancel"
        );

        let first = claim_ask(&mut ask_slot()).expect("claim");
        assert!(cancel_state(&mut ask_slot()));
        assert!(
            !cancel_state(&mut ask_slot()),
            "a second cancel has nothing left to cancel"
        );
        let second = claim_ask(&mut ask_slot()).expect("re-claim");
        assert!(second > first);
        assert!(cancel_state(&mut ask_slot()));
    }

    #[test]
    fn is_active_tracks_recording_and_transcribing() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        *ask_slot() = fresh_state();
        assert!(!is_active());
        claim_ask(&mut ask_slot()).expect("claim");
        assert!(is_active());
        begin_transcribing(&mut ask_slot()).expect("transcribing");
        assert!(is_active(), "transcription is still an in-flight ask");
        assert!(cancel_state(&mut ask_slot()));
        assert!(!is_active());
    }
}
