//! Go `printer/singlelinestringwriter.go`: the `EmitTextWriter` that writes a
//! line break as one space.

use crate::prelude::*;

/// Go `printer.singleLineStringWriter`.
// Go: printer/singlelinestringwriter.go:29 singleLineStringWriter
#[derive(Clone, Debug, Default)]
pub struct SingleLineStringWriter {
    builder: String,
    last_written: String,
}

// Go: printer/singlelinestringwriter.go:21 GetSingleLineStringWriter
// PORT: Go takes a writer from a `sync.Pool` and returns a release function
// that puts it back. There is no pool here: this returns a new cleared writer
// that the caller owns, and nothing needs to be released.

