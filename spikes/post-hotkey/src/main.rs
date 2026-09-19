use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

fn main() {
    let key: u16 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).expect("source");
    let flags = CGEventFlags::CGEventFlagAlternate | CGEventFlags::CGEventFlagShift;
    let down = CGEvent::new_keyboard_event(src.clone(), key, true).expect("down");
    down.set_flags(flags);
    down.post(CGEventTapLocation::HID);
    std::thread::sleep(std::time::Duration::from_millis(80));
    let up = CGEvent::new_keyboard_event(src, key, false).expect("up");
    up.set_flags(flags);
    up.post(CGEventTapLocation::HID);
    println!("posted Alt+Shift+keycode({key})");
}
