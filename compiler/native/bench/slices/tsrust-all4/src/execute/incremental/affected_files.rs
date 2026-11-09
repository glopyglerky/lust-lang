//! Port of execute/incremental/affectedfileshandler.go.
//!
//! PORT: Go runs `getFilesAffectedBy` and `handleDtsMayChangeOfAffectedFile`
//! on work groups, with `SyncMap`s and mutexes for the shared state. Here
//! they run one after another on the loading thread, in the order Go queues
//! them, so the handler state is plain fields. Go `dtsMayChange` maps are
//! shared by reference between the list and the queued call; here the call
//! gets the index of its map in the list.

use super::checker_access::*;
use super::hash::FileInfo;
use super::hash::*;
use super::program::{SignatureUpdateKind};
use super::snapshot::*;
use crate::emitter::emitter::EmitOnly;
use crate::emitter::program_emit::{};
use crate::frontend::prelude::*;
use std::sync::{Arc, Mutex};

// Go: incremental/affectedfileshandler.go:18 dtsMayChange
pub type DtsMayChange = FxIndexMap<Path, FileEmitKind>;

// Go: incremental/affectedfileshandler.go:24 updatedSignature
// PORT: Go `mu` guards the parallel computation; there is none here.
#[derive(Clone, Debug, Default)]
pub struct UpdatedSignature {
    pub signature: String,
    pub kind: SignatureUpdateKind,
}

// Go: incremental/affectedfileshandler.go:30 affectedFilesHandler
// PORT: Go `ctx` is dropped (the port has no cancellation).




// Go: incremental/affectedfileshandler.go:359 collectAllAffectedFiles
