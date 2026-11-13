//! Port of Go `ls/codelens.go`.

use crate::ls::prelude::*;

use crate::frontend::json::{MarshalerTo, json_marshal, };
use crate::scanner_util::{GoOffsets, };
use crate::spanmap::Feature;


// Go: ls/codelens.go:30 (the state of the `visit` closure in ProvideCodeLenses)


// Go: ls/codelens.go:68 codeLensKey

// Go: ls/codelens.go:73 keyForCodeLens


// PORT: Go puts the typed values in `[]any` and the JSON writer marshals
// each one with its own marshaler. `lsproto::Command.arguments` holds
// `LspAny`, so each value is marshaled and read back as `LspAny`. Objects
// keep key order and integers write the same text, so the output JSON is
// the same.


// Go: ls/codelens.go:188 isValidImplementationsCodeLensNode

// Go: ls/codelens.go:215 isValidReferenceLensNode
