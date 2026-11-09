//! Printer part 2: printer.go 1345 to 2454. Signature elements, type members,
//! types and binding patterns.
//!
//! PORT: Go methods take typed node pointers (`*ast.TypeParameterDeclaration`,
//! ...). Here every method takes a `Node` handle. Go `node.AsX().Field` becomes
//! the matching `Node` accessor. Go `ListFormat`, `WriteKind`,
//! `tokenEmitFlags` and `EmitFlags` constants use the crate `go_enum!` and
//! `go_flags!` style (`LFModifiers` -> `ListFormat::MODIFIERS`,
//! `WriteKindKeyword` -> `WriteKind::KEYWORD`, `tefNoComments` ->
//! `TokenEmitFlags::NO_COMMENTS`, `EFNoSourceMap` -> `EmitFlags::NO_SOURCE_MAP`).
//! Go `PrintHandlers` is embedded in `Printer`; here it is the field
//! `print_handlers`.

use crate::gostd::debug::kind_string;
use crate::prelude::*;
use crate::printer::*;

//
// Signature elements
//


// Go: printer/printer.go:1550 canEmitSimpleArrowHead

