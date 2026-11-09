//! Go `printer/printer.go` lines 2455 to 3689: expressions, misc and
//! statements. The `Printer` struct is in `printer_p1.rs`.

use crate::gostd::debug::kind_string;
use crate::prelude::*;

// PORT: Go uses local `printerState`, `tokenEmitFlags`, `ListFormat`,
// `WriteKind`, `EmitFlags`, `getLiteralTextFlags`, `SynthesizedComment` and
// `ast.CommentRange` from the other printer units. Their Rust names are
// `PrinterState`, `TokenEmitFlags::X`, `ListFormat::X`, `WriteKind::X`,
// `EmitFlags::X`, `GetLiteralTextFlags::X`, `SynthesizedComment` and
// `CommentRange` (Go constant prefixes `tef`, `LF`, `EF` and `WriteKind` are
// dropped, as for `TypeFlagsAny` -> `TypeFlags::ANY`).

//
// Expressions
//


//
// Misc
//


//
// Statements
//

