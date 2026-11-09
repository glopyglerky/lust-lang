//! Port of Go `ls/findallreferences.go`, lines 1-1245: the reference types,
//! entry points and definitions. The search half (Go lines 1246-2654) is
//! `findallreferences_p2.rs`.
//!
//! PORT notes for the whole file:
//! - Go `*ReferenceEntry` and `*SymbolAndEntries` are shared and changed in
//!   place (`resolveEntry` fills the ranges, the search appends references),
//!   so they are `Rc<RefCell<..>>`.
//! - Go reads `*ast.Symbol` fields without a checker. Symbol data lives in a
//!   checker arena here. Free functions that read a symbol take
//!   `symbols: &SymbolArena` first, as the `ast` helpers do. Language service
//!   methods where Go reads a symbol without a checker get the request
//!   checker (`ls_program::get_type_checker`) for the read. The pool returns
//!   the checker of the request, which made the symbols.
//! - `ls_program::get_type_checker` returns the same `Rc<RefCell<Checker>>`
//!   for nested calls in one request (Go gets the same checker again). A
//!   `borrow_mut` is held only where no nested call can borrow it again.
//! - The search methods are generic over the program (`ProgramView`, see
//!   `program_view.rs`), so that a cross-project request can run them on a
//!   search thread. They get the checker with `ProgramView::get_type_checker`,
//!   which is `ls_program::get_type_checker` on the dispatch thread.

use crate::ls::prelude::*;

use crate::astnav;
use crate::flags_macros::go_enum;
use crate::frontend::tspath;
use crate::gostd::{Context, GoError};
use crate::ls::lsconv;
use crate::lsp::lsproto;
use crate::lsp::lsproto::{HasTextDocumentPosition, HasTextDocumentURI};
use crate::program::ls_program;
use crate::spanmap::Feature;
use std::cell::OnceCell;
use std::collections::VecDeque;

// === types for settings ===
// Go: ls/findallreferences.go:29 referenceUse
go_enum!(ReferenceUse, i32 {
    NONE = 0;
    OTHER = 1;
    REFERENCES = 2;
    RENAME = 3;
});

// Go: ls/findallreferences.go:38 refOptions
// PORT: Go field `use` is a Rust keyword: `use_`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RefOptions {
    pub find_in_strings: bool,
    pub find_in_comments: bool,
    pub use_: ReferenceUse, // other, references, rename
    pub implementations: bool,
    pub use_aliases_for_rename: bool, // renamed from providePrefixAndSuffixTextForRename. default: true
}

// === types for results ===

// Go: ls/findallreferences.go:48 refInfo
// PORT: Go `*ast.SourceFile` is the file root `Node`; Go
// `*ast.FileReference` (nil in the module specifier case) is an `Option`.
#[derive(Clone, Debug, Default)]
pub struct RefInfo {
    pub file: Node,
    pub file_name: String,
    pub reference: Option<FileReference>,
    pub unverified: bool,
}

// Go: ls/findallreferences.go:55 SymbolAndEntries
// PORT: Go `*Definition` is never changed after it is made, so the
// definition is held by value (`Option` for nil).
#[derive(Clone, Debug, Default)]
pub struct SymbolAndEntries {
    pub definition: Option<Definition>,
    pub references: Vec<Rc<RefCell<ReferenceEntry>>>,
}

// Go: ls/findallreferences.go:60 NewSymbolAndEntries
pub fn new_symbol_and_entries(
    kind: DefinitionKind,
    node: Node,
    symbol: SymbolId,
    references: Vec<Rc<RefCell<ReferenceEntry>>>,
) -> Rc<RefCell<SymbolAndEntries>> {
    Rc::new(RefCell::new(SymbolAndEntries {
        definition: Some(Definition {
            kind,
            node,
            symbol,
            triple_slash_file_ref: None,
        }),
        references,
    }))
}

// Go: ls/findallreferences.go:71 DefinitionKind
go_enum!(DefinitionKind, i32 {
    SYMBOL = 0;
    LABEL = 1;
    KEYWORD = 2;
    THIS = 3;
    STRING = 4;
    TRIPLE_SLASH_REFERENCE = 5;
});

// Go: ls/findallreferences.go:82 Definition
#[derive(Clone, Debug, Default)]
pub struct Definition {
    pub kind: DefinitionKind,
    pub symbol: SymbolId,
    pub node: Node,
    pub triple_slash_file_ref: Option<TripleSlashDefinition>,
}

// Go: ls/findallreferences.go:88 tripleSlashDefinition
// PORT: Go `reference *ast.FileReference` can be nil (a module specifier
// reference from `getReferenceAtPosition`), so it is an `Option`.
#[derive(Clone, Debug, Default)]
pub struct TripleSlashDefinition {
    pub reference: Option<FileReference>,
    pub file: Node,
}

// Go: ls/findallreferences.go:93 entryKind
go_enum!(EntryKind, i32 {
    NONE = 0;
    RANGE = 1;
    NODE = 2;
    STRING_LITERAL = 3;
    SEARCHED_LOCAL_FOUND_PROPERTY = 4;
    SEARCHED_PROPERTY_FOUND_LOCAL = 5;
});

// Go: ls/findallreferences.go:104 ReferenceEntry
// PORT: Go `*core.TextRange` and `*lsproto.Location` are `Option`s. Go
// `*ast.SourceFile` is the file root `Node` (nil is `Node::NIL`).
#[derive(Clone, Debug, Default)]
pub struct ReferenceEntry {
    pub kind: EntryKind,
    pub node: Node,
    pub context: Node, // !!! ContextWithStartAndEndNode, optional
    pub source_file: Node,
    pub text_range: Option<TextRange>,
    pub lsp_range: Option<lsproto::Location>,
    pub unmappable: bool,
}

impl ReferenceEntry {
    // Go: ls/findallreferences.go:115 Node
    // Node returns the AST node for this reference entry.
    pub fn node(&self) -> Node {
        self.node
    }

    // Go: ls/findallreferences.go:120 IsNodeEntry
    // IsNodeEntry returns true if this is a node-backed reference entry.
    pub fn is_node_entry(&self) -> bool {
        self.node.is_some()
    }
}

impl SymbolAndEntries {
    // Go: ls/findallreferences.go:125 References
    // References returns the reference entries for this symbol.
    pub fn references(&self) -> &[Rc<RefCell<ReferenceEntry>>] {
        &self.references
    }

    // Go: ls/findallreferences.go:130 DefinitionNode
    // DefinitionNode returns the defining AST node for this symbol, if any.
    // PORT: reading `symbol.Declarations` takes the symbol arena.

    // Go: ls/findallreferences.go:143 DefinitionSymbol
    pub fn definition_symbol(&self) -> SymbolId {
        let Some(definition) = &self.definition else {
            return SymbolId::NIL;
        };
        definition.symbol
    }

    // Go: ls/findallreferences.go:150 canUseDefinitionSymbol
}


// Go: ls/findallreferences.go:208 newNodeEntryWithKind

// Go: ls/findallreferences.go:214 newNodeEntry

// Go: ls/findallreferences.go:223 getContextNodeForNodeEntry

// Go: ls/findallreferences.go:286 getContextNode

// Go: ls/findallreferences.go:335 getRangeOfNode


// Go: ls/findallreferences.go:354 isValidReferencePosition

// Go: ls/findallreferences.go:378 isForRenameWithPrefixAndSuffixText
pub fn is_for_rename_with_prefix_and_suffix_text(options: RefOptions) -> bool {
    options.use_ == ReferenceUse::RENAME && options.use_aliases_for_rename
}

// Go: ls/findallreferences.go:382 skipPastExportOrImportSpecifierOrUnion

// Go: ls/findallreferences.go:407 getSymbolScope
// PORT: Go reads the symbol without a checker and calls the package
// function `checker.IsExternalModuleSymbol`, which is a `Checker` method
// here. The checker is the first parameter.

// === functions on (*ls) ===

// Go: ls/findallreferences.go:480 position
// PORT: Go `position`; the Rust type name is `Position`. Write
// `lsproto::Position` qualified in ls code.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub uri: lsproto::DocumentUri,
    pub pos: lsproto::Position,
}

// Go: ls/findallreferences.go:479 `var _ lsproto.HasTextDocumentPosition = (*position)(nil)`

impl HasTextDocumentURI for Position {
    // Go: ls/findallreferences.go:487 TextDocumentURI
    fn text_document_uri(&self) -> lsproto::DocumentUri {
        self.uri.clone()
    }
}

impl HasTextDocumentPosition for Position {
    // Go: ls/findallreferences.go:488 TextDocumentPosition
    fn text_document_position(&self) -> lsproto::Position {
        self.pos
    }
}

// Go: ls/findallreferences.go:490 nonLocalDefinition
// PORT: Go embeds `position`; here the field `position`, and the two
// `HasTextDocument*` impls below forward to it. The Go `sync.OnceValue`
// functions close over the language service, so the struct borrows it
// (`'l`); each function caches its result in a `OnceCell`. Go returns a
// `lsproto.HasTextDocumentPosition` that is a `*position` or nil:
// `Option<Position>`.
pub struct NonLocalDefinition<'l> {
    pub position: Position,
    pub get_source_position: Box<dyn Fn() -> Option<Position> + 'l>,
    pub get_generated_position: Box<dyn Fn() -> Option<Position> + 'l>,
}

impl HasTextDocumentURI for NonLocalDefinition<'_> {
    // Go: promoted from the embedded `position`.
    fn text_document_uri(&self) -> lsproto::DocumentUri {
        self.position.text_document_uri()
    }
}

impl HasTextDocumentPosition for NonLocalDefinition<'_> {
    // Go: promoted from the embedded `position`.
    fn text_document_position(&self) -> lsproto::Position {
        self.position.text_document_position()
    }
}

// Go: ls/findallreferences.go:496 getFileAndStartPosFromDeclaration
// PORT: Go `core.TextPos` is `i32`.


// Go: ls/findallreferences.go:561 isDefinitionVisible
// This is special handling to determine if we should load up more projects and find location in other projects
// By default arrows (and such other ast kinds) are not visible as declaration emitter doesnt need them
// But we want to handle them specially so that they are visible if their parent is visible
// PORT: Go `emitResolver.IsDeclarationVisible` locks the resolver's checker
// and calls `isDeclarationVisible`. The caller already holds that checker
// (`c`), so this calls the inner method with it; the Rust exported form
// borrows the compile-pool checker of a worker thread.


// Go: ls/findallreferences.go:631 symbolEntryTransformOptions
#[derive(Clone, Copy, Debug, Default)]
pub struct SymbolEntryTransformOptions {
    // Force the result to be Location objects.
    pub require_locations_result: bool,
    // Omit node(s) containing the original position.
    pub drop_origin_nodes: bool,
}

// Go: ls/findallreferences.go:638 SymbolAndEntriesData
#[derive(Clone, Debug, Default)]
pub struct SymbolAndEntriesData {
    pub original_node: Node,
    pub symbols_and_entries: Vec<Rc<RefCell<SymbolAndEntries>>>,
    pub position: i32,
}









// Go: ls/findallreferences.go:866 referencedSymbolDefinitionInfo
// referencedSymbolDefinitionInfo holds the computed info for a definition
#[derive(Clone, Debug, Default)]
pub struct ReferencedSymbolDefinitionInfo {
    pub node: Node,
    pub location: lsproto::Location,
    pub display_text: Option<lsproto::VSClassifiedTextElement>,
}



// Go: ls/findallreferences.go:1081 isDeclarationOfSymbol
// PORT: reading `target.Declarations` takes the symbol arena.


// Go: ls/findallreferences.go:1210 SignatureUsage
// SignatureUsage represents a single usage of a signature declaration,
// pairing the reference name node with its containing call expression (if any).
#[derive(Clone, Copy, Debug, Default)]
pub struct SignatureUsage {
    pub name: Node, // The identifier reference node
    pub call: Node, // The containing call expression, or nil if not a call usage
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
    use crate::frontend::tspath::Path;
    use crate::scanner_util::go_string_from_bytes;
    use std::fmt::Write;

    // PORT: no Go test. `getRangeOfNode` counts the length of a literal and
    // steps its end back in Go bytes. An unterminated literal can end in a
    // marker unit (an invalid byte, a byte of a WTF-8 surrogate or a real
    // U+FDD0), which has more port bytes than Go bytes. The expected lines
    // are from a Go test at pin N with Go 1.27.1 (`followups17/tools/gomodel`
    // in the lane dir, an overlay test file in package `ls`) that prints, for
    // each string literal like node in `ForEachChild` order, its pos and end
    // and the range of `getRangeOfNode`, in Go bytes.
    #[test]
    fn range_of_a_string_literal_counts_go_bytes() {
        let go = "\
R0 17-20 18-20
R1 17-21 19-20
R2 2-6 3-5
R3 2-7 3-6
R4 2-6 3-5
R5 2-7 3-6
R6 2-5 3-4
R7 2-5 3-4
R8 2-7 3-6
R9 2-7 3-6
R9 12-14 12-14
R9 19-22 20-21
R10 2-4 2-4
R11 2-7 3-6
R12 14-17 15-17
";
        let texts: [&[u8]; 13] = [
            b"import { v } from \"\xff\n",
            b"import { v } from \"a\xff\n",
            b"o[\"\xed\xa0\x80\n];\n",
            b"o[\"p\xed\xa0\x80\n];\n",
            b"o[\"\xef\xb7\x90\n];\n",
            b"o[\"p\xef\xb7\x90\n];\n",
            b"o[\"\\\xff\n];\n",
            b"o['\xc3\xa9\n];\n",
            b"o['\xf0\x9f\x98\x80\n];\n",
            b"o[\"a\xffb\"];\no[\"\"];\no[\"x\"];\n",
            b"o[`\xff",
            b"o[`p\xef\xb7\x90",
            b"declare module \"\xff\n{ }\n",
        ];
        let mut port = String::new();
        for (i, bytes) in texts.into_iter().enumerate() {
            let text = go_string_from_bytes(bytes.to_vec());
            let file = parse_source_file(
                &SourceFileParseOptions {
                    file_name: "/a.ts".to_string(),
                    path: Path("/a.ts".to_string()),
                    ..Default::default()
                },
                crate::ast::FileText::new(text.clone(), false),
                ScriptKind::TS,
            );
            let go = |pos: i32| go_byte_offset(&text, pos);
            fn visit(node: Node, f: &mut dyn FnMut(Node)) {
                f(node);
                node.for_each_child(|child| {
                    visit(child, f);
                    false
                });
            }
            file.root.for_each_child(|child| {
                visit(child, &mut |node| {
                    if is_string_literal_like(node) {
                        let range = get_range_of_node(node, file.root, Node::NIL);
                        writeln!(
                            port,
                            "R{i} {}-{} {}-{}",
                            go(node.pos()),
                            go(node.end()),
                            go(range.pos()),
                            go(range.end())
                        )
                        .unwrap();
                    }
                });
                false
            });
        }
        assert_eq!(port, go);
    }
}
