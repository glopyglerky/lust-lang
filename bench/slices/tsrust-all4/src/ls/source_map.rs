//! Port of Go `ls/source_map.go`.

use crate::ls::prelude::*;

use crate::spanmap::{Feature, Fidelity};


// Go: ls/source_map.go:65 script
// PORT: `text` is shared with the host's file (`Host::read_file`), as the Go
// string is.


// PORT: Go passes a nil `*script` as an `lsconv.Script`; its methods then
// dereference nil and panic. `get_script` returns `Option<Script>`, and
// `&self.get_script(name)` is the Go interface value.


// PORT: the generated position is read only by `getNonLocalDefinition`,
// which runs on the dispatch thread, so these two stay on `NewProgram`.
