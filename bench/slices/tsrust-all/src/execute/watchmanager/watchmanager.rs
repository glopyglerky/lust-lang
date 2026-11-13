//! Go: internal/execute/watchmanager/watchmanager.go (fswatch directory
//! watches, event accumulation and DoCycle signaling for `tsc --watch` and
//! `tsc -b --watch`).
//!
//! PORT: threads. The frontend `Writer`, file system and backend are `Rc`
//! and stay on the dispatch thread. fswatch callbacks run on the debouncer
//! thread. So the Go `WatchManager` is split: `WatchManager` holds the
//! dispatch-thread fields, and `WatchManagerShared` (an `Arc`) holds the
//! fields that `onWatchEvents` and `handleWatchTerminated` touch
//! (`mu`, `watchedDirs`, the `doCycleCh` sender, `changedMu` and its data).
//! Those two methods live on `WatchManagerShared`. Their `DebugLog` and
//! `warnWriter` output goes to the process stdout (`stdio::CliStdout`): Go
//! passes `sys.Writer()`, which is `os.Stdout` for the real system, and the
//! `Rc` writer cannot cross threads.

use crate::execute::watchmanager::prelude::*;

use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::execute::tsc::{};
use crate::frontend::core_ls_ext;
use crate::frontend::tspath;
use crate::fswatch;
use crate::gostd::errors;

// Go: watchmanager.go:15 watchedDir
/// PORT: shared as `Arc<WatchedDir>`; Go compares `*watchedDir` pointers
/// (`Arc::ptr_eq`). Go sets `closer` after the watch starts, so it sits in a
/// `Mutex` (`None` is a nil closer).
pub struct WatchedDir {
    pub closer: Mutex<Option<Box<dyn fswatch::Watch>>>,
    pub recursive: bool,
}

impl WatchedDir {
    // PORT: Go `wd.closer.Close()` (the error is ignored).
    fn close_closer(&self) {
        if let Some(closer) = self.closer.lock().unwrap().as_ref() {
            let _ = closer.close();
        }
    }
}

// Go: watchmanager.go:20 dirWatchUpdate
struct DirWatchUpdate {
    dir: String,
    recursive: bool,
}

// Go: watchmanager.go:33 WatchManager

/// PORT: the `WatchManager` fields that the fswatch callback thread
/// touches.
pub struct WatchManagerShared {
    /// Go `mu`. Go locks it in `Lock` and unlocks it in `Unlock`.
    pub mu: GoMutex,
    /// Go `watchedDirs` (guarded by `mu` in Go; the `Mutex` is Rust's
    /// data lock, taken only for short reads and writes).
    pub watched_dirs: Mutex<FxHashMap<String, Arc<WatchedDir>>>,
    /// Go `doCycleCh` (see `DoCycleCh`).
    pub do_cycle_ch: DoCycleCh,

    pub changed_mu: Mutex<WatchManagerChanged>,
}

/// PORT: the `WatchManager` fields that Go `changedMu` guards.
#[derive(Default)]
pub struct WatchManagerChanged {
    /// Go `map[string]fswatch.EventKind`; `None` is Go's nil map.
    pub changed_paths: Option<FxHashMap<String, fswatch::EventKind>>,
    pub changed_overflow: bool,
}

/// PORT: Go `sync.Mutex` that one method locks and another unlocks
/// (`Lock` and `Unlock` around a DoCycle body). A std `MutexGuard` cannot
/// cross those calls, so this is a flag and a condition variable.
#[derive(Default)]
pub struct GoMutex {
    locked: Mutex<bool>,
    cond: Condvar,
}

impl GoMutex {
    /// Go `m.Lock()`.
    pub fn lock(&self) {
        let mut locked = self.locked.lock().unwrap();
        while *locked {
            locked = self.cond.wait(locked).unwrap();
        }
        *locked = true;
    }

    /// Go `m.Unlock()`.
    pub fn unlock(&self) {
        let mut locked = self.locked.lock().unwrap();
        if !*locked {
            panic!("sync: unlock of unlocked mutex");
        }
        *locked = false;
        self.cond.notify_one();
    }
}

/// PORT: Go `doCycleCh`, a `chan struct{}` of capacity 1 that `RunLoop`
/// receives from and `signalDoCycle` sends to without blocking. A Go send
/// while the receiver waits hands the value to it and leaves the buffer
/// empty, so a second send fits before the receiver runs. A std
/// `sync_channel(1)` keeps the first value in its slot until the receiver
/// thread runs, and then drops the second send.
#[derive(Default)]
pub struct DoCycleCh {
    state: Mutex<DoCycleState>,
    cond: Condvar,
}

#[derive(Default)]
struct DoCycleState {
    /// Values sent and not received yet: one handed to the waiting
    /// receiver and one in the buffer at most.
    pending: u8,
    /// The receiver waits in `recv_timeout`.
    waiting: bool,
}

impl DoCycleCh {
    /// Go `select { case ch <- struct{}{}: default: }`. False when the
    /// value does not fit.
    pub fn try_send(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let capacity = if state.waiting { 2 } else { 1 };
        if state.pending >= capacity {
            return false;
        }
        state.pending += 1;
        self.cond.notify_one();
        true
    }

}

// Go: watchmanager.go:50 NewWatchManager


impl WatchManagerShared {
    // Go: watchmanager.go:95 WatchManager.signalDoCycle
    pub fn signal_do_cycle(&self) {
        // Signal sent; the DoCycle loop will pick it up. Or a signal is
        // already pending; coalesced.
        let _ = self.do_cycle_ch.try_send();
    }

    // Go: watchmanager.go:104 WatchManager.onWatchEvents
    // PORT: runs on the fswatch callback thread. `debug_log` is whether Go
    // `wm.DebugLog` is non-nil; the text goes to the process stdout, as does
    // the `warnWriter` warning (see the file comment).

    // Go: watchmanager.go:147 WatchManager.handleWatchTerminated
    // PORT: runs on the fswatch callback thread (see onWatchEvents).
}

// Go: watchmanager.go:295 DirWatchSet
/// DirWatchSet accumulates the set of directories that should be watched while
/// answering coverage queries efficiently. A directory is "covered" when it is
/// already present in the set, or when it is contained within a recursive watch
/// directory already in the set.
// PORT: `names` maps each canonical key to the spelling first registered
// for it (ts#64210).
pub struct DirWatchSet {
    opts: tspath::ComparePathsOptions,
    dirs: FxHashMap<String, bool>,
    names: FxHashMap<String, String>,
}

// Go: watchmanager.go:301 NewDirWatchSet
pub fn new_dir_watch_set(opts: tspath::ComparePathsOptions) -> DirWatchSet {
    DirWatchSet {
        opts,
        dirs: FxHashMap::default(),
        names: FxHashMap::default(),
    }
}

impl DirWatchSet {
    // Go: watchmanager.go:309 DirWatchSet.canonical

    // Go: watchmanager.go:313 DirWatchSet.Set (ts#64210)

    // Go: watchmanager.go:322 DirWatchSet.Covered

    // Go: watchmanager.go:337 DirWatchSet.Dirs (ts#64210)
    // PORT: Go ranges over a map; the result is a map, so the order does
    // not matter.
    pub fn dirs(&self) -> FxHashMap<String, bool> {
        let mut dirs: FxHashMap<String, bool> =
            FxHashMap::with_capacity_and_hasher(self.dirs.len(), Default::default());
        for (key, recursive) in &self.dirs {
            dirs.insert(self.names[key].clone(), *recursive);
        }
        dirs
    }
}

// PORT: Go `fmt.Fprintf(w, ...)` on the callback thread, where `w` is the
// real system's `os.Stdout` (see the file comment): one write of
// `stdio::CliStdout`. Errors are ignored as in Go. `text` is in the port
// form, so this writes its Go bytes.

#[cfg(test)]
mod tests {
    use super::*;

    // PORT: not in Go. `DoCycleCh` keeps the Go channel rule: without a
    // waiting receiver one value fits; a waiting receiver takes the first
    // value, so a second one fits in the buffer.
    #[test]
    fn do_cycle_ch_hands_a_value_to_a_waiting_receiver() {
        let ch = Arc::new(DoCycleCh::default());
        assert!(ch.try_send());
        assert!(!ch.try_send());
        assert!(ch.recv_timeout(Duration::ZERO));
        assert!(!ch.recv_timeout(Duration::ZERO));

        let receiver = {
            let ch = ch.clone();
            std::thread::spawn(move || ch.recv_timeout(Duration::from_secs(60)))
        };
        while !ch.state.lock().unwrap().waiting {
            std::thread::yield_now();
        }
        assert!(ch.try_send());
        assert!(ch.try_send());
        assert!(!ch.try_send());
        assert!(receiver.join().unwrap());
        assert!(ch.recv_timeout(Duration::ZERO));
        assert!(!ch.recv_timeout(Duration::ZERO));
    }
}
