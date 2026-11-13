use crate::ls::prelude::*;

// Port of Go `ls/autoinsert.go`.

use crate::astnav;
use crate::gostd::{Context, GoError};
use crate::lsp::lsproto;
use crate::spanmap::Feature;


// Go: ls/autoinsert.go:77 isUnclosedTag
// PORT: Go takes `*ast.JsxElement`; the node handle is the JsxElement node.

// Go: ls/autoinsert.go:93 isUnclosedFragment
// PORT: Go takes `*ast.JsxFragment`; the node handle is the JsxFragment node.
