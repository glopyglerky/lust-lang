//! Port of internal/jsonrpc/jsonrpc.go.
//!
//! Package jsonrpc provides generic JSON-RPC 2.0 types and utilities
//! that can be shared between LSP and other JSON-RPC based protocols.
//!
//! PORT: Go marshals the tagged structs by reflection (JSON v2 default
//! rules); each one has a hand-written `MarshalerTo` / `UnmarshalerFrom`
//! here. Go `MarshalJSON` / `UnmarshalJSON` methods are kept as
//! `marshal_json` / `unmarshal_json` and the trait impls call them.

use crate::jsonrpc::prelude::*;

use crate::frontend::json::{
    JsonDecoder, JsonError, MarshalerTo, UnmarshalerFrom, json_marshal, 
    
};
use crate::frontend::json_ext::{
    AnyValue, JsonValue, 
    write_object_end, write_object_start,
    
};
use crate::gostd::{GoError, errors};
use std::sync::LazyLock;

// Go: jsonrpc.go:14 JSONRPCVersion
/// JSONRPCVersion represents the JSON-RPC version field, always "2.0".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JSONRPCVersion;

// Go: jsonrpc.go:16 jsonRPCVersion
const JSON_RPC_VERSION: &str = "\"2.0\"";

impl JSONRPCVersion {
    // Go: jsonrpc.go:18 MarshalJSON
    pub fn marshal_json(&self) -> Result<Vec<u8>, GoError> {
        Ok(JSON_RPC_VERSION.as_bytes().to_vec())
    }

    // Go: jsonrpc.go:24 UnmarshalJSON
}

// Go: jsonrpc.go:22 ErrInvalidJSONRPCVersion

// Go v2 calls MarshalJSON and writes its output with WriteValue.

// Go v2 calls UnmarshalJSON with the raw value (`null` included).
// A method error gets the v2 wrapping (`unmarshal_json_method_error`).

// Go: jsonrpc.go:32 ID
/// ID represents a JSON-RPC message ID, which can be either a string or integer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ID {
    str: String,
    int: i32,
}

// Go: jsonrpc.go:38 NewID
// NewID creates an ID from an IntegerOrString value.
// PORT: Go returns `*ID`; it is never nil, so the Rust result is the value.
pub fn new_id(raw_value: &IntegerOrString) -> ID {
    if let Some(s) = &raw_value.string {
        return ID {
            str: s.clone(),
            ..ID::default()
        };
    }
    ID {
        int: raw_value
            .integer
            .expect("invalid memory address or nil pointer dereference"),
        ..ID::default()
    }
}

// Go: jsonrpc.go:46 NewIDString
// NewIDString creates a string ID.
pub fn new_id_string(str: &str) -> ID {
    ID {
        str: str.to_string(),
        ..ID::default()
    }
}

// Go: jsonrpc.go:51 NewIDInt
// NewIDInt creates an integer ID.
pub fn new_id_int(i: i32) -> ID {
    ID {
        int: i,
        ..ID::default()
    }
}

impl ID {
    // Go: jsonrpc.go:55 String
    pub fn string(&self) -> String {
        if !self.str.is_empty() {
            return self.str.clone();
        }
        self.int.to_string()
    }

    // Go: jsonrpc.go:62 MarshalJSON

    // Go: jsonrpc.go:69 UnmarshalJSON

    // Go: jsonrpc.go:77 TryInt
    // PORT: the Go method checks a nil receiver, so the Rust receiver is an
    // `Option` (`None` is nil).
    pub fn try_int(id: Option<&ID>) -> (i32, bool) {
        match id {
            Some(id) if id.str.is_empty() => (id.int, true),
            _ => (0, false),
        }
    }

    // Go: jsonrpc.go:84 MustInt
    pub fn must_int(&self) -> i32 {
        if !self.str.is_empty() {
            panic!("ID is not an integer");
        }
        self.int
    }
}

// Go `%s` / `%v` of an `*ID` calls String().
impl std::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string())
    }
}

// Go v2 calls MarshalJSON and writes its output with WriteValue.

// Go v2 calls UnmarshalJSON with the raw value. A `*ID` field handles
// `null` in the pointer arshaler (`Option<ID>`).
// A method error gets the v2 wrapping (`unmarshal_json_method_error`).

// Go: jsonrpc.go:92 IntegerOrString
/// IntegerOrString is a helper type for creating IDs.
/// PORT: Go never marshals it (no JSON tags), so it has no JSON impls.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct IntegerOrString {
    pub integer: Option<i32>,
    pub string: Option<String>,
}

// Go: jsonrpc.go:98 ResponseError


// Go: `*ResponseError` is an `error`; `errors::from_value` wraps it.



// Go: jsonrpc.go:120
// Standard JSON-RPC error codes.
pub const CODE_PARSE_ERROR: i32 = -32700;
pub const CODE_INVALID_REQUEST: i32 = -32600;
pub const CODE_METHOD_NOT_FOUND: i32 = -32601;
pub const CODE_INVALID_PARAMS: i32 = -32602;
pub const CODE_INTERNAL_ERROR: i32 = -32603;

// Go: jsonrpc.go:129 MessageKind
/// MessageKind indicates what type of message this is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MessageKind(pub i32);

// Go: jsonrpc.go:131
impl MessageKind {
    pub const NOTIFICATION: MessageKind = MessageKind(0);
    pub const REQUEST: MessageKind = MessageKind(1);
    pub const RESPONSE: MessageKind = MessageKind(2);
}

// Go: jsonrpc.go:139 Message




// Go: jsonrpc.go:175 RequestMessage



// Go: jsonrpc.go:183 ResponseMessage


