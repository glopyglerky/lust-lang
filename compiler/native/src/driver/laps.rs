// Collect laps: phase times inside collect, for the request timer
// (`--timings`, `LUST_STATS`). A thread-local list, so collect needs no
// new Program field: the driver calls `start` before collect, collect
// calls `mark` at each phase end, and the driver folds `take` into its
// own laps (`Timer::lap_parts`). Collect runs its phases on the request
// thread, so the marks of one collect never mix with another's.
use std::cell::RefCell;
use std::time::Instant;

thread_local! {
    static LAPS: RefCell<(Option<Instant>, Vec<(&'static str, f64)>)> = const { RefCell::new((None, Vec::new())) };
}

/// Starts a new lap list on this thread (drops any old marks).
pub(crate) fn start() {
    LAPS.with(|l| *l.borrow_mut() = (Some(Instant::now()), Vec::new()));
}

/// Ends the phase `name`: its time is the time since the last mark (or
/// `start`). A no-op until `start` runs on this thread.
pub(crate) fn mark(name: &'static str) {
    LAPS.with(|l| {
        let mut l = l.borrow_mut();
        if let Some(last) = l.0 {
            let now = Instant::now();
            l.1.push((name, (now - last).as_secs_f64() * 1e3));
            l.0 = Some(now);
        }
    });
}

/// The marks since `start`, in order; stops recording until the next
/// `start`.
pub(crate) fn take() -> Vec<(&'static str, f64)> {
    LAPS.with(|l| {
        let mut l = l.borrow_mut();
        l.0 = None;
        std::mem::take(&mut l.1)
    })
}
