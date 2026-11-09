//! Port of internal/ipc/protocol.go (internal/api/protocol.go before tsgo#4712).

use crate::ipc::prelude::*;

use crate::frontend::json_ext::AnyValue;
use crate::gostd::GoError;
use crate::jsonrpc;

// Go: ipc/protocol.go:8 Message
// Message is an alias for jsonrpc.Message for convenience.

// Go: ipc/protocol.go:11 Protocol
// Protocol defines the interface for reading and writing API messages.
// PORT: Go `*jsonrpc.ID` parameters are `Option<&jsonrpc::ID>` (nil is
// `None`); Go `any` params and results are `Option<Box<dyn AnyValue>>`
// (nil is `None`). The methods take `&mut self` because the reader and
// writer buffers change; the connections keep the protocol in a `RefCell`.
