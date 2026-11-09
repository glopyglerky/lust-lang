//! Go: internal/fswatch/fanotify_linux.go (the Linux fanotify backend).
//!
//! PORT: D-W1 (no `libc`, no `unsafe`). The syscalls go through the `unix`
//! shim (`nix` for fanotify, `name-to-handle-at` for file handles, `rustix`
//! for the rest). `fanotify_available()` probes `FanotifyInit` as Go does, so
//! `Default()` picks fanotify when the kernel allows it. Records are read with
//! `unix::FanotifyEventMetadata::from_ne_bytes` and `from_ne_bytes` on the
//! info records instead of Go's `unsafe.Pointer` cast and
//! `binary.NativeEndian`.
//!
//! PORT: locks. As in inotify_linux.rs, the port takes `self.base.mu` where
//! Go takes `b.mu` and keeps the backend fields in `self.locked`, which is
//! only taken after `base.mu` and never held across a call that takes it
//! again.

use crate::fswatch::prelude::*;

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::fswatch::unix;
use crate::gostd::errors;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

// ---------------------------------------------------------------------------
// fanotify_linux.go: Linux fanotify backend
//
// Uses Linux's fanotify(7) API (kernel ≥ 5.13 without CAP_SYS_ADMIN) to
// watch directory trees. Unlike inotify, fanotify uses FID-based event
// reporting (FAN_REPORT_FID | FAN_REPORT_DFID_NAME): each event carries the
// parent directory's file handle and the child entry name, so watch
// dispatch is keyed by (fsid, handle_type, handle_bytes) instead of a wd
// integer. This avoids the inotify per-user watch limit (fs.inotify.
// max_user_watches) entirely.
//
//	┌──────────────────────────────────────────────────────────────┐
//	│                     fanotifyBackend                          │
//	│                                                              │
//	│  ┌───────────┐        poll(2)        ┌──────────────────┐    │
//	│  │ pipe[0]   ├──────────────────────►│                  │    │
//	│  │ (wakeup)  │                       │  start()         │    │
//	│  └───────────┘                       │  goroutine       │    │
//	│  ┌───────────┐                       │  (event loop)    │    │
//	│  │ fanotify  ├──────────────────────►│                  │    │
//	│  │ fd        │                       └────────┬─────────┘    │
//	│  └───────────┘                                │              │
//	│                                      handleEvents()          │
//	│                                               │              │
//	│                                  parseFanotifyDfidNames      │
//	│                                  (extract handleKey + name)  │
//	│                                               │              │
//	│                                               ▼              │
//	│                               ┌─────────────────────────┐    │
//	│                               │ subscriptions           │    │
//	│                               │ map[handleKey] → []sub  │    │
//	│                               │  sub.dirWatch.events    │    │
//	│                               └─────────────────────────┘    │
//	│                                                              │
//	│  handleKey = (fsid, handle_type, handle_bytes)               │
//	│  obtained via statfs(2) + name_to_handle_at(2) per dir       │
//	└──────────────────────────────────────────────────────────────┘
//
// Goroutines and threading:
//   - One long-lived goroutine (start), launched by watcherBase.run(). It
//     owns the poll(2) loop and runs for the process lifetime. All event
//     reading and dispatch (handleEvents, handleParsedEvent,
//     handleSubscription, handleRenameEvent) execute on this goroutine,
//     under b.mu.
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
// WatchDirectory flow (caller goroutine):
//  1. Walk the target directory.
//  2. On the first subscribe, probe FAN_RENAME support (Linux 5.17+) by
//     attempting a fanotify_mark with FAN_RENAME. If the kernel returns
//     EINVAL or EOPNOTSUPP, fall back to FAN_MOVED_FROM | FAN_MOVED_TO
//     (two separate events instead of one paired event for renames).
//  3. For every directory found:
//     a. fanotify_mark(FAN_MARK_ADD | FAN_MARK_ONLYDIR) to watch it.
//     b. name_to_handle_at(2) to obtain the directory's file handle.
//     c. statfs(2) to obtain the filesystem ID (fsid).
//     d. Map (fsid, handle_type, handle_bytes) → fanotifySubscription.
//
// Event format:
//   Each event has a FanotifyEventMetadata header followed by variable-length
//   info records. parseFanotifyDfidNames extracts DFID_NAME records
//   (FAN_EVENT_INFO_TYPE_DFID_NAME, OLD_DFID_NAME, NEW_DFID_NAME) containing
//   the parent directory's file handle and child entry name. The file handle
//   is matched against the watch map to find the watched directory.
//
// Event dispatch (on start goroutine):
//   - FAN_CREATE / FAN_MOVED_TO  → events.create (→ EventUpdate); if the new
//     entry is a directory (FAN_ONDIR), recursively walk and mark it.
//   - FAN_MODIFY               → events.update (→ EventUpdate).
//   - FAN_DELETE* / FAN_MOVE*    → events.remove (→ EventDelete); drop
//     subscriptions for the removed path and any descendants.
//   - FAN_RENAME (5.17+)         → single paired event with OLD_DFID_NAME +
//     NEW_DFID_NAME info records; handleRenameEvent deletes the old path and
//     creates the new path in one pass.
//   - FAN_Q_OVERFLOW             → set ErrOverflow on every active dirWatch.
//
//   Merged events: fanotify can merge consecutive events on the same object
//   into one event with multiple mask bits. When both create and delete bits
//   are set, handleSubscription stats the path to determine which happened
//   last (exists → delete-then-create = update; gone → create-then-delete =
//   events cancel out).
//
//   After processing all buffered events, call dirWatch.notify() on each
//   touched dirWatch to trigger the debouncer.
//
// Shutdown:
//   Write a byte to pipe[1] → poll sees POLLIN on pipe[0] → loop exits →
//   deferred closeFDs closes fanotify fd, pipe fds, and signals endedSignal.
// ---------------------------------------------------------------------------

// Go: fanotify_linux.go:115 fanotifyInitFlags
pub const FANOTIFY_INIT_FLAGS: u32 = unix::FAN_CLASS_NOTIF
    | unix::FAN_CLOEXEC
    | unix::FAN_NONBLOCK
    | unix::FAN_REPORT_FID
    | unix::FAN_REPORT_DFID_NAME;

// Go: fanotify_linux.go:118 fanotifyMarkMaskBase
pub const FANOTIFY_MARK_MASK_BASE: u64 = unix::FAN_CREATE
    | unix::FAN_DELETE
    | unix::FAN_MODIFY
    | unix::FAN_DELETE_SELF
    | unix::FAN_MOVE_SELF
    | unix::FAN_ONDIR
    | unix::FAN_EVENT_ON_CHILD;

// Go: fanotify_linux.go:123 fanotifyMarkMaskRename
// Used when FAN_RENAME is available (Linux 5.17+).
pub const FANOTIFY_MARK_MASK_RENAME: u64 = FANOTIFY_MARK_MASK_BASE | unix::FAN_RENAME;

// Go: fanotify_linux.go:126 fanotifyMarkMaskMovedFromTo
// Fallback when FAN_RENAME is not available.
pub const FANOTIFY_MARK_MASK_MOVED_FROM_TO: u64 =
    FANOTIFY_MARK_MASK_BASE | unix::FAN_MOVED_FROM | unix::FAN_MOVED_TO;

// Go: fanotify_linux.go:128 fanotifyMarkAddFlags
pub const FANOTIFY_MARK_ADD_FLAGS: u32 =
    unix::FAN_MARK_ADD | unix::FAN_MARK_ONLYDIR | unix::FAN_MARK_DONT_FOLLOW;

// Go: fanotify_linux.go:130 fanotifyBufferSize
pub const FANOTIFY_BUFFER_SIZE: usize = 8192;

// Go: fanotify_linux.go:135 fanotifyHandleKey
/// fanotifyHandleKey uniquely identifies a filesystem object by its fsid and
/// file handle. Used as a map key for watch dispatch.
///
/// PORT: Go keeps the raw handle bytes in a `string`; the port keeps them in
/// a `Vec<u8>` (the bytes are not text).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FanotifyHandleKey {
    pub fsid: [i32; 2],
    pub handle_type: i32,
    /// raw handle bytes as string for map comparability
    pub handle: Vec<u8>,
}

// Go: fanotify_linux.go:141 makeFanotifyHandleKey
pub fn make_fanotify_handle_key(
    fsid: [i32; 2],
    handle_type: i32,
    handle_bytes: &[u8],
) -> FanotifyHandleKey {
    FanotifyHandleKey {
        fsid,
        handle_type,
        handle: handle_bytes.to_vec(),
    }
}

// Go: fanotify_linux.go:150 fanotifySubscription
/// fanotifySubscription mirrors inotifySubscription for the fanotify backend.
pub struct FanotifySubscription {
    pub path: String,
    pub watch_path: String,
    pub dir_watch: Arc<DirWatch>,
    pub key: FanotifyHandleKey,
}

// Go: fanotify_linux.go:158 fanotifyDfidName
/// fanotifyDfidName holds parsed directory FID + name from an info record.
#[derive(Clone, Debug)]
pub struct FanotifyDfidName {
    pub key: FanotifyHandleKey,
    /// child entry name, or "" for self-events on directories
    pub name: String,
}

// Go: fanotify_linux.go:164 fanotifyBackend

/// PORT: the `fanotifyBackend` fields that Go guards with `b.mu`.
pub struct FanotifyLocked {
    pub pipe_fds: [i32; 2],
    pub fanotify_fd: i32,
    /// fanotifyMarkMaskRename or fanotifyMarkMaskMovedFromTo; 0 until first subscribe
    pub mark_mask: u64,
    pub subscriptions: FxHashMap<FanotifyHandleKey, Vec<Arc<FanotifySubscription>>>,
}

// Go: fanotify_linux.go:182 init
// PORT: Go's `init()` sets the factory on the package var
// `fanotifyWatcher`; the port's package var calls this when it is built.

// Go: fanotify_linux.go:190 fanotifyAvailable

// Go: fanotify_linux.go:206 newFanotifyBackend

// PORT: Go `defer func() { b.closeFDs(); close(b.endedSignal) }()` in
// start. The guard also runs when start unwinds, as a Go defer does in a
// panic.




// Go: fanotify_linux.go:372 maybeWrapUnsupportedFilesystem
// PORT: a free function after the `fanotifyBackend` methods (Go puts it
// after `markDir`).

// Go: fanotify_linux.go:602 parseFanotifyDfidNames

// Go: fanotify_linux.go:649 parseFanotifyFidRecord
