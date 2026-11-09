//! Port of internal/ipc/conn.go (internal/api/conn.go before tsgo#4712).
//!
//! PORT: the API session and project state live on one thread, so the
//! handler and the connections are `Rc` values and their methods take
//! `&self`. Go `json.Value` is `JsonValue`; Go `any` params and results are
//! `Option<Box<dyn AnyValue>>` (nil is `None`).

use crate::ipc::prelude::*;

use crate::frontend::json::UnmarshalerFrom;
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::gostd::{Context, GoError, errors};
use std::sync::LazyLock;

// Go: ipc/conn.go:10

// Go: ipc/conn.go:16 Handler
// Handler processes incoming API requests and notifications.

// Go: ipc/conn.go:24 Conn
// Conn represents a bidirectional connection for API communication.

// Go: ipc/conn.go:37 UnmarshalParams
// UnmarshalParams is a helper to unmarshal params into a typed struct.
// PORT: Go returns `*T`; nil is `None`.

/// Go `%v` of the value `recover()` returns in the connections' request
/// handlers.
/// PORT: a Rust panic payload is the panic message (`&str` or `String`,
/// or the message of a `core::go_panic`); any other payload has no text.
pub fn recovered_value(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(panic) = payload.downcast_ref::<crate::core::GoPanic>() {
        panic.message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::new()
    }
}
