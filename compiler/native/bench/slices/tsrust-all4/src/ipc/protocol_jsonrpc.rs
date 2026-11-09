//! Port of internal/ipc/protocol_jsonrpc.go (internal/api/protocol_jsonrpc.go
//! before tsgo#4712).

use crate::ipc::prelude::*;

use crate::frontend::json::json_marshal;
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::gostd::{GoError, errors};
use crate::ipc::protocol::{};
use crate::ipc::transport::{ConnReader, ReadWriteCloser};
use crate::jsonrpc;
// PORT: the marshaled text is in the port form (see
// `scanner_util::GO_STRING_MARKER`); the connection gets its Go bytes.
use std::io::BufReader;
use std::sync::Arc;

// Go: ipc/protocol_jsonrpc.go:12 JSONRPCProtocol
// JSONRPCProtocol implements the Protocol interface using JSON-RPC 2.0
// with the LSP base protocol framing (Content-Length headers).
pub struct JSONRPCProtocol {
    reader: jsonrpc::Reader,
    writer: jsonrpc::Writer,
}

// Go: ipc/protocol_jsonrpc.go:17
// var _ Protocol = (*JSONRPCProtocol)(nil) is the `impl Protocol` below.

// Go: ipc/protocol_jsonrpc.go:20 NewJSONRPCProtocol
// NewJSONRPCProtocol creates a new JSON-RPC protocol handler.
// PORT: Go takes an `io.ReadWriter`; the port takes the shared connection.
// Go `jsonrpc.NewReader` adds a 4096-byte `bufio.Reader`; the Rust reader
// takes the buffered reader from the caller.

