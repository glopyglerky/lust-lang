//! Port of execute/incremental/emitfileshandler.go.
//!
//! PORT: Go emits the affected files on a work group; here
//! `emit_files_incremental` sends them as one `emit_batch`, in the order Go
//! queues them. Each emit runs on the file's checker thread, and each
//! checker thread runs its files in that order. The `WriteFile` callback
//! runs on the checker threads, so what it reads from the snapshot is
//! copied into it, and what it records (Go `signatures`, `emitSignatures`,
//! `latestChangedDtsFiles` SyncMaps) goes to `EmitFilesShared` behind a
//! mutex. Go `ctx` is dropped. `start_emit_files` and `finish_emit_files`
//! split the emit of all affected files at the wait for the emit jobs, so
//! `Program::start_emit` can send them behind the check.

use super::hash::FileInfo;
use super::hash::*;
use super::program::{SignatureUpdateKind};
use super::snapshot::*;
use crate::emitter::emitter::EmitOnly;
use crate::emitter::program_emit::{
    
    
};
use crate::frontend::prelude::*;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

// Go: incremental/emitfileshandler.go:15 emitUpdate


/// One file that `emit_files_incremental` emits: its path, its pending
/// emit kind, the kind it emits now, and the file.
type QueuedEmit = (Path, FileEmitKind, FileEmitKind, Node);

// Go: incremental/emitfileshandler.go:21 emitFilesHandler





/// The snapshot entries of one file in `DtsWriteContext`.
struct DtsWriteFile {
    old_emit_signature: Option<EmitSignature>,
    file_info: Option<FileInfo>,
}



// Go: the end of the `getEmitOptions` WriteFile callback
// (incremental/emitfileshandler.go:231): the write, and with
// `differsOnlyInMap` the revert of the file's modified time.



thread_local! {
    /// True while `buffer_early_emit_writes` runs its `start`.
    static BUFFER_EARLY_EMIT_WRITES: Cell<bool> = const { Cell::new(false) };
}


/// The most threads that `flush_writes` writes on.
const MAX_FLUSH_THREADS: usize = 8;


// Go: incremental/emitfileshandler.go:252 skipDtsOutputOfComposite
// Compare to existing computed signature and store it or handle the changes in d.ts map option from before
// returning undefined means that, we dont need to emit this d.ts file since its contents didnt change
// PORT: Go `file` is `path` (its path) and `entry` (its snapshot entries).

// Go: incremental/emitfileshandler.go:338 emitFiles




#[cfg(test)]
mod tests {
    use super::{BUFFER_EARLY_EMIT_WRITES, buffer_early_emit_writes};

    /// k2gaps1: a panic inside the buffered start ends the buffering. `tsc -b`
    /// keeps a task's panic and goes on, so a flag left on would make every
    /// later emit on this thread keep its writes in a buffer.
    #[test]
    fn buffer_early_emit_writes_ends_on_a_panic() {
        let result = std::panic::catch_unwind(|| {
            buffer_early_emit_writes(|| {
                assert!(BUFFER_EARLY_EMIT_WRITES.get(), "the start buffers");
                panic!("the start panics");
            });
        });
        assert!(result.is_err(), "the panic reaches the caller");
        assert!(
            !BUFFER_EARLY_EMIT_WRITES.get(),
            "the buffering ended with the panic"
        );
    }
}
