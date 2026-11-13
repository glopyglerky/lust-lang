//! The dispatch-thread work queue. No Go source: this is the port's model
//! for Go goroutines and `time.AfterFunc` callbacks that touch
//! language-service state (PORTING.md "Threads" and "Go runtime").
//!
//! One thread (the LSP dispatch thread) owns all `Rc`/`RefCell` state. A Go
//! `go f()` over that state becomes `local::go(Box::new(f))`, and a Go
//! `time.AfterFunc(d, f)` over that state becomes `local::after_func`. Both
//! run on the dispatch thread when it calls `run_pending`, in the order they
//! became ready: a job when it is queued, a timer when it is due.
//!
//! Contract with the dispatch loop: the server calls `set_waker(f)` once. A
//! timer thread calls the waker when a `LocalTimer` becomes due. Each
//! thread that arms a `LocalTimer` gets one timer thread for all its timers,
//! made at its first arm. It ends when that thread ends. The
//! dispatch loop calls `run_pending()` after each message and after each
//! wake-up. Go `WaitForBackgroundTasks` (`background::Queue::wait`) calls
//! `run_pending()`, and `wait_pending()` while a queued task sleeps on a
//! timer, until the queue's tasks have finished.
//!
//! A job that waits for another thread is `post_later(f)`: the other
//! thread posts the `Send` handle it returns once, when its work ends, and
//! `f` then runs in `run_pending` like a `go` job (Go: a goroutine that a
//! channel send makes runnable). Nothing runs or polls before the post.
//!
//! Idle work (`go_idle`) is a second, separate queue for long work that
//! sends nothing to the client (the auto-import warm). The dispatch loop
//! runs it with `run_idle()` only when no message waits: at once or after a
//! quiet period with no message, as the job asks (`IdleStart`), so it does
//! not delay a request that has arrived. `run_pending` does not run it.
//!
//! Garbage (`drop_later`) is a third queue, for large frees that Go's
//! garbage collector does in the background (a released program, the
//! parse tasks of a load). On a thread whose dispatch loop called
//! `keep_garbage`, the values wait until the loop calls `drop_garbage`
//! after a message, so the free is not in the answer time. On other
//! threads `drop_later` drops at once. `drop_after_pause` uses this queue
//! only for the first release after a client pause (`note_message_gap`),
//! and only when the client has not sent its next edit yet
//! (`set_stream_check`). It drops at once otherwise, so only the releases
//! of one message wait (of one pipelined burst on an API connection: see
//! `drop_after_pause`).
//!
//! The queues are per thread: `go`, `post_later`, `go_idle`, `after_func`,
//! `run_pending`, `run_idle`, `drop_later` and `drop_garbage` act on the
//! calling thread's queues.

use crate::prelude::*;

use std::any::Any;
use std::cell::Cell;
use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

// PORT: Go mutexes do not poison.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}



/// The `LocalTimer`s of one thread, for its timer thread
/// (`run_local_timers`).
// PERF (perfplan4 R6): one timer thread per thread, not one per timer. The
// language server stops and makes two timers on each edit
// (`schedule_idle_cache_clean`, `schedule_cleanup_locked`), which started
// two threads per edit. Go `time.AfterFunc` starts no thread.
#[derive(Default)]
struct Timers {
    /// Go `t.when` of each armed timer, by id.
    when: FxHashMap<u64, Instant>,
    /// The armed timers by `when`, then id.
    due: BTreeSet<(Instant, u64)>,
    /// Due entries of each timer in the queue that have not run yet.
    queued: FxHashMap<u64, u32>,
    /// Whether the timer thread runs.
    thread_running: bool,
    /// Set when the thread ends: its timer thread then ends too.
    closed: bool,
}

impl Timers {
    /// Arms timer `id` for `when`. Returns whether it was armed before.
    fn arm(&mut self, id: u64, when: Instant) -> bool {
        let old = self.when.insert(id, when);
        if let Some(old) = old {
            self.due.remove(&(old, id));
        }
        self.due.insert((when, id));
        old.is_some()
    }

    /// Disarms timer `id`. Returns whether it was armed.
    fn disarm(&mut self, id: u64) -> bool {
        let old = self.when.remove(&id);
        if let Some(old) = old {
            self.due.remove(&(old, id));
        }
        old.is_some()
    }

    /// Whether timer `id` is neither armed nor queued.
    fn idle(&self, id: u64) -> bool {
        !self.when.contains_key(&id) && !self.queued.contains_key(&id)
    }

    /// Counts one run of a due entry of timer `id`.
    fn unqueue(&mut self, id: u64) {
        if let Some(count) = self.queued.get_mut(&id) {
            *count -= 1;
            if *count == 0 {
                self.queued.remove(&id);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Entry {
    /// A job from `go`, by id.
    Job(u64),
    /// A due `LocalTimer`, by id.
    Timer(u64),
}


/// The state of `drop_after_pause` on one thread.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pause {
    /// It drops at once.
    None,
    /// The client paused before the current burst of messages, and nothing
    /// was released since. The next release decides.
    Paused,
    /// The release after the pause waits for `drop_garbage`, and so do the
    /// other values of its message, until `drop_garbage` starts.
    Kept,
}










/// When the dispatch loop may start an idle job (`go_idle`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdleStart {
    /// As soon as no message waits.
    AtOnce,
    /// After a quiet period with no message (the LSP server's
    /// `IDLE_QUIET_PERIOD`).
    AfterQuiet,
}




/// The most values that `drop_garbage` keeps while a message waits. More
/// are dropped at once, so a stream of messages with no gap between them
/// does not keep old programs.
const GARBAGE_LIMIT: usize = 16;



/// A gap between two messages that is this long or longer starts a new
/// burst of messages. Shorter gaps are one burst (a client that sends a
/// didChange and its requests together).
const BURST_GAP: Duration = Duration::from_millis(1);

/// A gap before a burst that is this long or longer is a client pause
/// (`drop_after_pause`). It is longer than every gap of a client that sends
/// its messages with no pause (0.6 to 9 ms in ls_edit_bench) and shorter
/// than the gaps of a person who types.
const CLIENT_PAUSE: Duration = Duration::from_millis(20);

















// Go: time/sleep.go:24 when
fn when(d: Duration) -> Instant {
    let now = Instant::now();
    match now.checked_add(d) {
        Some(t) => t,
        // PORT: Go clamps to MaxInt64 nanoseconds.
        None => {
            let mut d = Duration::from_nanos(i64::MAX as u64);
            loop {
                if let Some(t) = now.checked_add(d) {
                    return t;
                }
                d /= 2;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the ready work of this thread until no timer is armed.
    fn run_until_idle() {
        while wait_pending() {
            run_pending();
        }
    }

    #[test]
    fn timers_of_one_thread_fire_in_deadline_order_after_stop_and_reset() {
        // A thread of its own: the timer thread ends with it.
        std::thread::spawn(|| {
            let log = Rc::new(RefCell::new(Vec::new()));
            let push = |name: &'static str| -> Box<dyn FnMut()> {
                let log = log.clone();
                Box::new(move || log.borrow_mut().push(name))
            };
            let late = after_func(Duration::from_millis(40), push("late"));
            let early = after_func(Duration::from_millis(10), push("early"));
            let stopped = after_func(Duration::from_millis(5), push("stopped"));
            assert!(stopped.stop());
            assert!(!stopped.stop());
            run_until_idle();
            assert_eq!(*log.borrow(), ["early", "late"]);
            // Reset after it fired: false, and it runs again.
            assert!(!early.reset(Duration::from_millis(1)));
            // Reset while armed moves the deadline: true.
            assert!(!late.reset(Duration::from_millis(30)));
            assert!(late.reset(Duration::from_millis(2)));
            run_until_idle();
            assert_eq!(*log.borrow(), ["early", "late", "early", "late"]);
        })
        .join()
        .expect("timer test thread");
    }

    // `next_idle` gives the start of the oldest idle job, and `run_idle`
    // runs the jobs in queue order.
    #[test]
    fn idle_jobs_run_in_order_and_give_their_start() {
        std::thread::spawn(|| {
            let log = Rc::new(RefCell::new(Vec::new()));
            assert_eq!(next_idle(), None);
            for (start, name) in [
                (IdleStart::AtOnce, "warm"),
                (IdleStart::AfterQuiet, "retry"),
            ] {
                let log = log.clone();
                go_idle(start, Box::new(move || log.borrow_mut().push(name)));
            }
            assert_eq!(next_idle(), Some(IdleStart::AtOnce));
            run_pending();
            assert!(
                log.borrow().is_empty(),
                "run_pending does not run idle work"
            );
            assert!(run_idle());
            assert_eq!(next_idle(), Some(IdleStart::AfterQuiet));
            assert!(run_idle());
            assert_eq!(next_idle(), None);
            assert!(!run_idle());
            assert_eq!(*log.borrow(), ["warm", "retry"]);
        })
        .join()
        .expect("idle test thread");
    }

    // Only the first release after a client pause waits for
    // `drop_garbage`, with the other values of its message, and only when the
    // client has not sent its next edit and no kept release waits.
    #[test]
    fn drop_after_pause_keeps_one_release_after_a_client_pause() {
        std::thread::spawn(|| {
            keep_garbage();
            let stream = Rc::new(Cell::new(false));
            {
                let stream = stream.clone();
                set_stream_check(Box::new(move || stream.get()));
            }
            // A release of two values. Returns how many wait.
            let release = || {
                let values = [Rc::new(()), Rc::new(())];
                for value in &values {
                    drop_after_pause(Box::new(value.clone()));
                }
                values.iter().filter(|v| Rc::strong_count(v) == 2).count()
            };
            // One message after `gap_ms` with a release, as the dispatch
            // loop runs it.
            let message = |gap_ms: u64| {
                drop_garbage(|| stream.get());
                note_message_gap(Duration::from_millis(gap_ms));
                release()
            };
            assert_eq!(message(0), 0, "no pause before the first message");
            assert_eq!(message(20), 2, "the first release after a pause of 20 ms");
            assert_eq!(message(0), 0, "the next release of the burst");
            assert_eq!(garbage_len(), 0, "drop_garbage after the answer");
            assert_eq!(message(5), 0, "a new burst with no pause");

            stream.set(true);
            assert_eq!(message(500), 0, "the next edit waits at the release");
            stream.set(false);

            // A message came while the kept release was freed.
            assert_eq!(message(500), 2);
            drop_garbage(|| true);
            note_message_gap(Duration::from_millis(500));
            assert_eq!(release(), 0, "a kept release waits");

            // A message with no release after the pause.
            drop_garbage(|| false);
            note_message_gap(Duration::from_millis(500));
            assert_eq!(message(0), 2, "the first release of the burst");
            drop_garbage(|| false);
            assert_eq!(garbage_len(), 0);
        })
        .join()
        .expect("drop_after_pause test thread");
    }
}
