//! Go: internal/fswatch/inotify_linux.go (the Linux inotify backend).
//!
//! PORT: D-W1 (no `libc`, no `unsafe`). The syscalls go through the `unix`
//! shim, which calls rustix. Records are read with
//! `unix::InotifyEvent::from_ne_bytes` instead of Go's `unsafe.Pointer` cast.
//!
//! PORT: locks. Go guards the backend fields with the embedded
//! `watcherBase.mu` (`b.mu`). The port takes `self.base.mu` where Go takes
//! `b.mu` and keeps the fields in `self.locked`, a second mutex that is only
//! taken after `base.mu` (or by the start thread before it shares them) and
//! never held across a call that takes it again.

use crate::fswatch::prelude::*;

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::fswatch::unix;
use crate::gostd::errors;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

// ---------------------------------------------------------------------------
// inotify_linux.go: Linux inotify backend
//
// Uses the kernel's inotify(7) subsystem to watch directory trees. A single
// inotify instance serves all subscriptions for the process lifetime.
//
//	┌───────────────────────────────────────────────────────────┐
//	│                    inotifyBackend                         │
//	│                                                           │
//	│  ┌───────────┐        poll(2)        ┌─────────────────┐  │
//	│  │ pipe[0]   ├──────────────────────►│                 │  │
//	│  │ (wakeup)  │                       │  start()        │  │
//	│  └───────────┘                       │  goroutine      │  │
//	│  ┌───────────┐                       │  (event loop)   │  │
//	│  │ inotify   ├──────────────────────►│                 │  │
//	│  │ fd        │                       └────────┬────────┘  │
//	│  └───────────┘                                │           │
//	│                                      handleEvents()       │
//	│                                               │           │
//	│                                               ▼           │
//	│                               ┌─────────────────────────┐ │
//	│                               │ subscriptions           │ │
//	│                               │ map[wd] → []sub         │ │
//	│                               │  sub.dirWatch.events    │ │
//	│                               └─────────────────────────┘ │
//	└───────────────────────────────────────────────────────────┘
//
// Goroutines and threading:
//   - One long-lived goroutine (start), launched by watcherBase.run(). It
//     owns the poll(2) loop and runs for the process lifetime. All event
//     reading and dispatch (handleEvents, handleEvent, handleSubscription)
//     execute on this goroutine, under b.mu.
//   - subscribe/closeWatch run on the caller's goroutine under
//     watcherBase.mu. The event loop acquires b.mu for watch map
//     access, providing safe interleaving.
//
// Callback delivery:
//   dirWatch.notify() posts to the shared process-wide debouncer. After a
//   coalescing window (50 ms min / 500 ms max), the debouncer invokes all
//   registered WatchCallbacks on its own dedicated goroutine; never on
//   the caller's goroutine or the event-loop goroutine.
//
// WatchDirectory flow:
//  1. Walk the target directory (caller goroutine).
//  2. For every directory found, call inotify_add_watch to obtain a
//     watch descriptor (wd). Map wd → inotifySubscription.
//
// Event dispatch (handleEvents → handleSubscription, on start goroutine):
//   - IN_CREATE / IN_MOVED_TO  → events.create (→ EventUpdate); if the new
//     entry is a directory (IN_ISDIR), recursively walk and watch it.
//   - IN_MODIFY                → events.update.
//   - IN_DELETE* / IN_MOVE*    → events.remove; drop inotify subscriptions
//     for the removed path and any descendants.
//   - IN_Q_OVERFLOW            → set ErrOverflow on every active dirWatch.
//   After processing all buffered events, call dirWatch.notify() on each
//   touched dirWatch to trigger the debouncer.
//
// Shutdown:
//   Write a byte to pipe[1] → poll sees POLLIN on pipe[0] → loop exits →
//   deferred closeFDs closes inotify fd, pipe fds, and signals endedSignal.
// ---------------------------------------------------------------------------

// Go: inotify_linux.go:77 inotifyMask
pub const INOTIFY_MASK: u32 = unix::IN_CREATE
    | unix::IN_DELETE
    | unix::IN_DELETE_SELF
    | unix::IN_MODIFY
    | unix::IN_MOVE_SELF
    | unix::IN_MOVED_FROM
    | unix::IN_MOVED_TO
    | unix::IN_DONT_FOLLOW
    | unix::IN_ONLYDIR
    | unix::IN_EXCL_UNLINK;
// Go: inotify_linux.go:87 inotifyBufferSize
pub const INOTIFY_BUFFER_SIZE: usize = 8192;

// Go: inotify_linux.go:91 inotifySubscription

// Go: inotify_linux.go:99 inotifyBackend


// Go: inotify_linux.go:117 init
// PORT: Go's `init()` sets the factory on the package var
// `inotifyWatcher`; the port's package var calls this when it is built.

// Go: inotify_linux.go:121 newInotifyBackend

// PORT: Go `defer func() { b.closeFDs(); close(b.endedSignal) }()` in
// start. The guard also runs when start unwinds, as a Go defer does in a
// panic.




// PORT: Go's inline test `s.path == path || (len(s.path) > len(path) &&
// s.path[len(path)] == '/' && s.path[:len(path)] == path)` on bytes.
fn is_path_or_descendant(s_path: &str, path: &str) -> bool {
    let s = s_path.as_bytes();
    let p = path.as_bytes();
    s == p || (s.len() > p.len() && s[p.len()] == b'/' && &s[..p.len()] == p)
}
