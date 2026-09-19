use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use core_foundation::base::TCFType;
use core_foundation::runloop::CFRunLoop;
use core_foundation::string::CFString;
use core_graphics::event::{
    CallbackResult, CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use foreign_types::ForeignType;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn CGEventGetTimestamp(event: *const std::os::raw::c_void) -> u64;
}

#[derive(Clone, Copy)]
struct Rec {
    is_down: bool,
    keycode: i64,
    event_ns: u64,
    recv_ns: u64,
}

fn timebase() -> &'static libc::mach_timebase_info_data_t {
    static TB: OnceLock<libc::mach_timebase_info_data_t> = OnceLock::new();
    TB.get_or_init(|| unsafe {
        let mut tb = libc::mach_timebase_info_data_t { numer: 0, denom: 0 };
        libc::mach_timebase_info(&mut tb);
        tb
    })
}

fn now_ns_since_boot() -> u64 {
    unsafe {
        let t = libc::mach_absolute_time();
        let tb = timebase();
        t * tb.numer as u64 / tb.denom as u64
    }
}

fn percentile(sorted: &[u128], p: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let keycode: i64 = args
        .iter()
        .position(|a| a == "--key")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let secs: u64 = args
        .iter()
        .position(|a| a == "--secs")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let posts: u32 = args
        .iter()
        .position(|a| a == "--posts")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let trusted = unsafe { AXIsProcessTrusted() };
    println!("AXIsProcessTrusted (Accessibility): {trusted}");
    println!(
        "target keycode: {keycode} (F8=100) | listen window: {secs}s | synthetic posts: {posts}"
    );

    let stats: Arc<Mutex<Vec<Rec>>> = Arc::new(Mutex::new(Vec::new()));
    let cb_stats = Arc::clone(&stats);
    let cb_key = keycode;

    let tap_thread = thread::spawn(move || {
        let tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown, CGEventType::KeyUp],
            move |_proxy, ty, event| {
                let code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                let event_ns = unsafe { CGEventGetTimestamp(event.as_ptr() as *const _) };
                let is_down = matches!(ty, CGEventType::KeyDown);
                let recv = now_ns_since_boot();
                if code == cb_key {
                    eprintln!("EVENT key={code} {} latency_us={}", if is_down { "DOWN" } else { "UP" }, recv.saturating_sub(event_ns) / 1000);
                }
                cb_stats.lock().unwrap().push(Rec {
                    is_down,
                    keycode: code,
                    event_ns,
                    recv_ns: recv,
                });
                CallbackResult::Keep
            },
        );
        let tap = match tap {
            Ok(t) => t,
            Err(_) => {
                eprintln!("RESULT: TAP_CREATE_FAILED — event tap could not be created.");
                eprintln!("  (on macOS this almost always means Accessibility/Input Monitoring not granted to the host process)");
                std::process::exit(2);
            }
        };
        unsafe {
            let source = tap
                .mach_port()
                .create_runloop_source(0)
                .expect("runloop source");
            let mode = CFString::new("kCFRunLoopDefaultMode");
            let rl = CFRunLoop::get_current();
            rl.add_source(&source, mode.as_concrete_TypeRef());
            tap.enable();
            eprintln!("TAP: enabled, entering runloop");
            CFRunLoop::run_current();
            eprintln!("TAP: RUNLOOP EXITED");
        }
    });

    thread::sleep(Duration::from_millis(400));
    eprintln!("MAIN: warmup done");

    if posts > 0 {
        let src =
            CGEventSource::new(CGEventSourceStateID::CombinedSessionState).expect("event source");
        for i in 0..posts {
            let down = CGEvent::new_keyboard_event(src.clone(), keycode as u16, true)
                .expect("down event");
            down.post(CGEventTapLocation::HID);
            thread::sleep(Duration::from_millis(60));
            let up = CGEvent::new_keyboard_event(src.clone(), keycode as u16, false)
                .expect("up event");
            up.post(CGEventTapLocation::HID);
            thread::sleep(Duration::from_millis(140));
            eprintln!("post {i} done @ {:?}", std::time::Instant::now());
        }
        println!("posted {posts} synthetic down/up pairs");
    }

    let remaining = secs.saturating_sub((posts as u64 * 200) / 1000 + 1);
    eprintln!("MAIN: posts done, sleeping {remaining}s for real keys");
    thread::sleep(Duration::from_secs(remaining.max(1)));
    eprintln!("MAIN: listen window over, computing summary");

    let s = stats.lock().unwrap();
    let target: Vec<&Rec> = s.iter().filter(|r| r.keycode == keycode).collect();
    let other = s.len() - target.len();
    let downs = target.iter().filter(|r| r.is_down).count();
    let ups = target.iter().filter(|r| !r.is_down).count();
    let mut latencies: Vec<u128> = target
        .iter()
        .filter(|r| r.recv_ns >= r.event_ns && r.recv_ns - r.event_ns < 60_000_000_000)
        .map(|r| (r.recv_ns - r.event_ns) as u128)
        .collect();
    latencies.sort_unstable();

    println!("---- S1 PROBE SUMMARY ----");
    println!(
        "target-key events: {} (down: {downs}, up: {ups}) | other-key events: {other}",
        target.len()
    );
    println!(
        "latency us: p50={} p95={} max={}",
        percentile(&latencies, 0.50),
        percentile(&latencies, 0.95),
        latencies.last().copied().unwrap_or(0)
    );
    let expected = (posts as usize) * 2;
    let missing = expected.saturating_sub(target.len());
    println!(
        "synthetic delivery: {}/{} events {}",
        target.len(),
        expected,
        if missing == 0 {
            "(COMPLETE)".to_string()
        } else {
            format!("(MISSING {missing})")
        }
    );
    let p50_us = percentile(&latencies, 0.50);
    if missing == 0 && downs == ups && p50_us <= 50_000 {
        println!(
            "VERDICT-CANDIDATE: PASS (complete delivery, paired down/up, p50 {p50_us}us <= 50ms)"
        );
    } else {
        println!("VERDICT-CANDIDATE: INVESTIGATE (see numbers above)");
    }
    std::process::exit(0);
}
