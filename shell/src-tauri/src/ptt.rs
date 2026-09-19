//! Push-to-talk raw-event route (W2, macOS). S1 spike validated global
//! key-down + key-up delivery via a listen-only session event tap
//! (RFC-0002 §4.3; findings in plans/m0-spike-plan.md).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;

use core_foundation::base::TCFType;
use core_foundation::runloop::CFRunLoop;
use core_foundation::string::CFString;
use core_graphics::event::{
    CallbackResult, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField,
};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PttError {
    PermissionDenied,
    TapCreateFailed,
}

pub fn permission_granted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub const DEFAULT_PTT_KEYCODE: i64 = 100;

/// Next PTT state for a key event; `None` when nothing changed (other key,
/// auto-repeat, or a repeat of the current direction).
pub fn transition(pressed: bool, keycode: i64, down: bool, autorepeat: bool, target: i64) -> Option<bool> {
    if keycode != target || autorepeat || pressed == down {
        return None;
    }
    Some(down)
}

/// Listens for global key events of `target` and sends `true` on press,
/// `false` on release to `tx`. The listener thread runs until the process exits.
pub fn spawn_listener(target: i64, tx: Sender<bool>) -> Result<JoinHandle<()>, PttError> {
    if !permission_granted() {
        return Err(PttError::PermissionDenied);
    }
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), PttError>>();

    let handle = std::thread::spawn(move || {
        let pressed = Arc::new(AtomicBool::new(false));
        let cb_pressed = Arc::clone(&pressed);
        let tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown, CGEventType::KeyUp],
            move |_proxy, ty, event| {
                let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                let autorepeat =
                    event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0;
                let down = matches!(ty, CGEventType::KeyDown);
                let prev = cb_pressed.load(Ordering::Relaxed);
                if let Some(next) = transition(prev, keycode, down, autorepeat, target) {
                    cb_pressed.store(next, Ordering::Relaxed);
                    let _ = tx.send(next);
                }
                CallbackResult::Keep
            },
        );
        let tap = match tap {
            Ok(t) => t,
            Err(_) => {
                let _ = ready_tx.send(Err(PttError::TapCreateFailed));
                return;
            }
        };
        let source = match tap.mach_port().create_runloop_source(0) {
            Ok(s) => s,
            Err(_) => {
                let _ = ready_tx.send(Err(PttError::TapCreateFailed));
                return;
            }
        };
        // The source must attach to the default mode: common modes is an
        // empty set on a bare thread runloop and the tap never fires (S1).
        let mode = CFString::new("kCFRunLoopDefaultMode");
        let rl = CFRunLoop::get_current();
        rl.add_source(&source, mode.as_concrete_TypeRef());
        tap.enable();
        let _ = ready_tx.send(Ok(()));
        CFRunLoop::run_current();
    });

    match ready_rx.recv() {
        Ok(Ok(())) => Ok(handle),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(PttError::TapCreateFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET: i64 = 100;

    #[test]
    fn other_keys_are_ignored() {
        assert_eq!(transition(false, 99, true, false, TARGET), None);
        assert_eq!(transition(true, 99, false, false, TARGET), None);
    }

    #[test]
    fn autorepeat_is_ignored() {
        assert_eq!(transition(true, TARGET, true, true, TARGET), None);
    }

    #[test]
    fn press_transitions_to_held() {
        assert_eq!(transition(false, TARGET, true, false, TARGET), Some(true));
    }

    #[test]
    fn release_transitions_to_idle() {
        assert_eq!(transition(true, TARGET, false, false, TARGET), Some(false));
    }

    #[test]
    fn duplicate_direction_is_ignored() {
        assert_eq!(transition(true, TARGET, true, false, TARGET), None);
        assert_eq!(transition(false, TARGET, false, false, TARGET), None);
    }
}
