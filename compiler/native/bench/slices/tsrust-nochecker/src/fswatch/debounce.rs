//! Go: internal/fswatch/debounce.go (the per-backend event debouncer).
//!
//! PORT: the loop goroutine is a `std::thread`. Go `d.mu` and `d.latchMu`
//! are `std::sync::Mutex`es over the fields they guard (`DebounceLocked`,
//! `DebounceLatch`). The latch channels are `SignalChan`s.

use crate::fswatch::prelude::*;

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

// Go: debounce.go:9 defaultMinWaitTime, defaultMaxWaitTime
pub const DEFAULT_MIN_WAIT_TIME: Duration = Duration::from_millis(50);
pub const DEFAULT_MAX_WAIT_TIME: Duration = Duration::from_millis(500);

// Go: debounce.go:14 minWaitTime, maxWaitTime
// PORT: Go package vars that only tests change; statics here.
pub static MIN_WAIT_TIME: Duration = DEFAULT_MIN_WAIT_TIME;
pub static MAX_WAIT_TIME: Duration = DEFAULT_MAX_WAIT_TIME;

/// PORT: not in Go. How many debouncers deliver a fire now
/// (`fire_callbacks`). In Go a callback that sends on a channel
/// (watchmanager `signalDoCycle`) makes the receiver the next goroutine of
/// the sender's P, so the receiver nearly always runs after the debounce
/// goroutine has delivered the whole fire. A port thread wakes at once on
/// another CPU, so the receiver waits for the fire (`wait_for_fires`).
static FIRING: Mutex<usize> = Mutex::new(0);
static FIRES_DONE: Condvar = Condvar::new();

/// PORT: not in Go. Waits until no debouncer delivers a fire (see `FIRING`).
pub fn wait_for_fires() {
    let mut firing = FIRING.lock().unwrap();
    while *firing > 0 {
        firing = FIRES_DONE.wait(firing).unwrap();
    }
}

/// Counts one fire in `FIRING` while it lives, also when a callback panics.
struct Firing;

impl Firing {
    fn start() -> Firing {
        *FIRING.lock().unwrap() += 1;
        Firing
    }
}


// Go: debounce.go:28 debounce


/// PORT: the `debounce` fields that Go `d.latchMu` guards. `None` is a nil
/// channel.
#[derive(Default)]
pub struct DebounceLatch {
    pub wait_ch: Option<Arc<SignalChan>>,
    pub trigger_ch: Option<Arc<SignalChan>>,
    pub notified: bool,
}

// Go: debounce.go:41 newDebounce

