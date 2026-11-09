//! Go `printer/printer.go` lines 1 to 1344: the `Printer` struct, options,
//! print handlers, low-level writing, line terminator counts, custom emit
//! behavior checks, tokens, literals and names.
//!
//! Other parts (printer_p2 to printer_p5) add more `impl Printer` blocks.

use crate::flags_macros::go_enum;
use crate::gostd::debug::kind_string;
use crate::prelude::*;
use std::borrow::Cow;
use std::cell::{Cell, RefMut};

// Go: printer/printer.go:35 PrinterOptions
#[derive(Clone, Debug, Default)]
pub struct PrinterOptions {
    pub remove_comments: bool,
    pub new_line: NewLineKind,
    pub omit_trailing_semicolon: bool,
    pub no_emit_helpers: bool,
    // Module                        core.ModuleKind
    // ModuleResolution              core.ModuleResolutionKind
    pub target: ScriptTarget,
    pub source_map: bool,
    pub inline_source_map: bool,
    pub inline_sources: bool,
    pub omit_brace_source_map_positions: bool,
    // ExtendedDiagnostics           bool
    pub only_print_js_doc_style: bool,
    pub never_ascii_escape: bool,
    // StripInternal                 bool
    pub preserve_source_newlines: bool,
    pub terminate_unterminated_literals: bool, // !!!
}

// Go: printer/printer.go:55 PrintHandlers
// PORT: Go func fields become `Option<Rc<dyn Fn>>`. The commented-out `!!!`
// hooks in Go are not ported.

// Go: sourcemap/source.go:5 Source
/// Go `sourcemap.Source` as the printer holds it. Go `*ast.SourceFile`
/// implements the interface, and the port keeps that source file as its
/// `Node` (nil is `SourceMapSource::NIL`). ts#63936:
/// `PrintHandlers.map_source_position` can return a source that is not a
/// node (Go `compiler.declarationMapSource` in `emitter.rs`).
#[derive(Clone)]
pub enum SourceMapSource {
    Node(Node),
    Other(Rc<dyn crate::sourcemap::source::Source>),
}

impl SourceMapSource {
    pub const NIL: SourceMapSource = SourceMapSource::Node(Node::NIL);

    /// Go `source == nil`.
    pub fn is_nil(&self) -> bool {
        matches!(self, SourceMapSource::Node(node) if node.is_nil())
    }

    /// Go `source != nil`.
    pub fn is_some(&self) -> bool {
        !self.is_nil()
    }


}


/// Go `*sourcemap.Generator`.
pub use crate::sourcemap::generator::Generator as SourceMapGenerator;
/// Go `sourcemap.SourceIndex`.
pub use crate::sourcemap::generator::SourceIndex;



// Go: printer/printer.go:117 Printer



// Go: printer/printer.go:149 detachedCommentsInfo
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DetachedCommentsInfo {
    pub(crate) node_pos: i32,
    pub(crate) detached_comment_end_pos: i32,
}

// Go: printer/printer.go:154 commentState
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CommentState {
    pub(crate) emit_flags: EmitFlags, // holds the emit flags for the current node
    pub(crate) comment_range: TextRange, // holds the comment range calculated for the current node
    pub(crate) container_pos: i32, // captures the value of containerPos prior to entering an node
    pub(crate) container_end: i32, // captures the value of containerEnd prior to entering an node
    pub(crate) declaration_list_container_end: i32, // captures the value of declarationListContainerEnd prior to entering an node
}

// Go: printer/printer.go:162 sourceMapState
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SourceMapState {
    pub(crate) emit_flags: EmitFlags, // holds the emit flags for the current node
    pub(crate) source_map_range: TextRange, // holds the source map range calculated for the current node
    pub(crate) has_token_source_map_range: bool, // captures whether the source map range was set for the current node
}

// Go: printer/printer.go:168 printerState
// PORT: Go holds arena pointers that may be nil. Rust holds the states by
// value; a nil pointer is `None`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PrinterState {
    pub(crate) comment_state: Option<CommentState>,
    pub(crate) source_map_state: Option<SourceMapState>,
}

// PORT: Go `ast.CommentRange` (ast/ast.go). No port of it exists yet, so it
// is defined here. Move it to the ast module when one lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommentRange {
    pub text_range: TextRange,
    pub kind: SyntaxKind,
    pub has_trailing_new_line: bool,
}

impl CommentRange {
    pub fn pos(&self) -> i32 {
        self.text_range.pos()
    }

    pub fn end(&self) -> i32 {
        self.text_range.end()
    }

    pub fn len(&self) -> i32 {
        self.text_range.len()
    }
}

// Go: printer/printer.go:173 NewPrinter




// Go: printer/printer.go:196 getLiteralTextOfNode
// PORT: body of the Printer method as a free function over the Printer state
// it reads (`NodeTextState` and `p.nameGenerator`). The name generator
// callback runs it without the Printer. See `new_printer`.
// PERF: returns a Cow, borrowed where the text is a source slice or
// `node.text()`.

// Go: printer/printer.go:230 getTextOfNode
// PORT: free function form of the Printer method. See
// `get_literal_text_of_node_worker`.



// Go: printer/utilities.go:199 canUseOriginalText
// PORT: a copy of the private `can_use_original_text` in utilities.rs, for
// `get_literal_text_cow`. Keep the two the same. The debug_assert in
// `get_literal_text_cow` checks the result against `get_literal_text`.

//
// Low-level writing
//

// Go: printer/printer.go:268 WriteKind
go_enum!(WriteKind, i32 {
    NONE = 0; // WriteKindNone
    KEYWORD = 1; // WriteKindKeyword
    OPERATOR = 2; // WriteKindOperator
    PUNCTUATION = 3; // WriteKindPunctuation
    STRING_LITERAL = 4; // WriteKindStringLiteral
    PARAMETER = 5; // WriteKindParameter
    PROPERTY = 6; // WriteKindProperty
    COMMENT = 7; // WriteKindComment
    LITERAL = 8; // WriteKindLiteral
});


// PORT: Go compares `*ast.NodeList` pointers. Two handles name the same list
// when both are nil, or both point at the same list in the same file.

// Go: stringutil/util.go:87 SplitLines
// PORT: the stringutil package has no port yet; this private copy serves writeLines.

// Go: stringutil/util.go:115 GuessIndentation
// PORT: the stringutil package has no port yet; this private copy serves writeLines.
