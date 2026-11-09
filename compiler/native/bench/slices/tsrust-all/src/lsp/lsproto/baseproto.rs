//! Port of internal/lsp/lsproto/baseproto.go.
//!
//! https://microsoft.github.io/language-server-protocol/specifications/base/0.9/specification/
//!
//! PORT: Go embeds `*jsonrpc.Reader` / `*jsonrpc.Writer`; the Rust structs
//! hold them in a field with the embedded type's name and forward the
//! promoted `Read` / `Write` methods.

use crate::lsp::lsproto::prelude::*;

use std::io::{BufRead, Write};

// Go: baseproto.go:12 BaseReader
/// BaseReader wraps jsonrpc.Reader for backwards compatibility.
pub struct BaseReader {
    pub reader: crate::jsonrpc::Reader,
}

// Go: baseproto.go:17 NewBaseReader
// NewBaseReader creates a new BaseReader.
pub fn new_base_reader(r: Box<dyn BufRead + Send>) -> BaseReader {
    BaseReader {
        reader: crate::jsonrpc::new_reader(r),
    }
}

impl BaseReader {
    // Go: promoted jsonrpc.Reader.Read
}

// Go: baseproto.go:24 BaseWriter

// Go: baseproto.go:29 NewBaseWriter
// NewBaseWriter creates a new BaseWriter.

