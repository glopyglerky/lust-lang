//! Port of internal/lsp/lsproto/jsonrpc.go.
//!
//! PORT: Go `any` fields (`Message.msg`, `RequestMessage.Params`,
//! `ResponseMessage.Result`) are `Option<Box<dyn AnyValue>>` (`None` is
//! nil). An inbound message keeps its params as the raw `JsonValue`; the
//! handler decodes them (`lsp.rs` `unmarshal_params`). Go `UnmarshalJSON`
//! methods are kept as `unmarshal_json`, which returns the `GoError` chain
//! (`errors::is(&err, &from_value(ErrorCode::X))` works on it). The
//! `UnmarshalerFrom` impls call them and keep only the error text, so
//! callers that test the error code call `unmarshal_json`.

use crate::lsp::lsproto::prelude::*;

// Go: `fmt.Errorf("%w: %w", code, err)`.

// Go: jsonrpc.go:12 NewID
// NewID creates an ID from an IntegerOrString value.
// This wrapper exists because lsproto has its own IntegerOrString type.
// PORT: Go returns `*jsonrpc.ID`; it is never nil, so the Rust result is
// the value.
pub fn new_id(raw_value: &IntegerOrString) -> crate::jsonrpc::ID {
    if let Some(s) = &raw_value.string {
        return crate::jsonrpc::new_id_string(s);
    }
    crate::jsonrpc::new_id_int(
        raw_value
            .integer
            .unwrap_or_else(|| crate::core::go_nil_dereference()),
    )
}

// Go: jsonrpc.go:19 Message



// Go v2 calls MarshalJSON and writes its output with WriteValue.

// Go v2 calls UnmarshalJSON with the raw value.

// Go: jsonrpc.go:34 the anonymous `raw` struct of Message.UnmarshalJSON.

// Go v2 default struct arshaler for the anonymous struct.

// Go: jsonrpc.go:80 RequestMessage


// Go v2 default struct arshaler (no MarshalJSON method).

// Go v2 calls UnmarshalJSON with the raw value.

// Go: jsonrpc.go:103 the anonymous `raw` struct of RequestMessage.UnmarshalJSON.

// Go v2 default struct arshaler for the anonymous struct.

// Go: jsonrpc.go:118 ResponseMessage
// PORT: `id` has no omitzero in Go, so a nil id writes `null`.


// Go v2 default struct arshaler.

// Go v2 default struct arshaler.
