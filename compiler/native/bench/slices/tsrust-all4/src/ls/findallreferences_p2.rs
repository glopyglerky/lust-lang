//! Port of Go `ls/findallreferences.go` lines 1246-2654: the search half of
//! find-all-references (special searches, module references and the
//! `refState` search engine).

use crate::ls::prelude::*;

use crate::astnav;
use crate::gostd::Context;
use crate::scanner_util::{
    GoOffsets, 
    
};
use std::borrow::Cow;

// PORT (whole file):
// - The types of Go lines 29-473 (`ReferenceEntry`, `SymbolAndEntries`,
//   `Definition`, `RefOptions`, ...) are in findallreferences_p1.rs. Go
//   `*ReferenceEntry` and `*SymbolAndEntries` are `Rc<RefCell<..>>`
//   (map-ls-navigation 2.2).
// - Go `*checker.Checker` parameters are `&mut Checker`. `refState` holds the
//   checker for the whole search (`RefState<'c, P>`).
// - Go `*compiler.Program` parameters are `program: P` with
//   `P: ProgramView` (program_view.rs), so that the search can run on a
//   search thread.
// - Go `[]*ast.SourceFile` is `&[Node]` (file roots). The source file name set
//   (`*collections.Set[string]`) is `&FxHashSet<String>`.
// - Go positions (`int`) are `i32` byte offsets.
// - Go `func(...)` parameters are `&mut dyn FnMut(...)`. A callback that
//   calls the checker gets it as its first argument.
// - A Go function that returns `[]*SymbolAndEntries` returns a `Vec`; Go nil
//   and empty are the same `Vec` unless a caller tells them apart (then the
//   function returns `Option<Vec<..>>` and says so).


// Go: ls/findallreferences.go:1394 isStringLiteralPropertyReference


// Go: ls/findallreferences.go:1421 getReferencedSymbolsSpecial
// PORT: every non-nil Go result has one element, so an empty `Vec` is Go nil.

// Go: ls/findallreferences.go:1477 getLabelReferencesInNode

// Go: ls/findallreferences.go:1490 getReferencesForThisKeyword

// Go: ls/findallreferences.go:1568 getReferencesForSuperKeyword

// Go: ls/findallreferences.go:1604 getAllReferencesForImportMeta

// Go: ls/findallreferences.go:1620 getAllReferencesForKeyword

// Go: ls/findallreferences.go:1637 getPossibleSymbolReferenceNodes
// PORT: the crate prelude also exports the checker/services.go function of
// this name; this file uses its own, as Go package `ls` does.

// Go: ls/findallreferences.go:1646 getPossibleSymbolReferencePositions
// PORT: Go indexes the text by byte. The search runs on the bytes, because
// `position + symbolNameLength + 1` can fall inside a multi-byte character,
// where a `&str` slice would panic.
// PORT: Go searches the Go bytes. The text is in the source form and a
// string literal's name in the value form (see
// `scanner_util::GO_STRING_MARKER`): a lone surrogate is 3 invalid byte
// units in the text and one surrogate unit in the name, so a port search
// does not find it. A name with a marker unit, or with a char that can be
// the second char of one (lead byte 0xF4), is searched on the Go bytes
// (`go_string_bytes`) and the Go positions are mapped back
// (`port_byte_offset`, with one scan for all of them: `GoOffsets`). So is
// any name in a text with a marker when the container does not start the
// text: as in Go, the first index is relative
// to the container start and is then used as an absolute position, which
// is only the same position in both forms when the bytes before agree.
// Otherwise the port search finds the same positions. When the container
// starts the text, each is a match of whole units, and the bytes next to it
// are read as Go bytes (`go_unit_before`, `go_unit_at`), since a marker
// unit has other bytes than Go's. With another container the text has no
// marker, so its bytes are Go's.
// PERF: Go `strings.Index` is one `memmem::Finder`, built once per call. It
// returns the same first index. A byte-by-byte compare cost 29% to 56% of
// find-references time on large files such as lib.dom.d.ts. The usual name
// costs no scan for markers: a byte before a match is a unit's only when it
// is not ASCII, and a byte after one only when it is the marker's lead byte.

// Go: ls/findallreferences.go:1692 findFirstJsxNode
// findFirstJsxNode recursively searches for the first JSX element, self-closing element, or fragment

// Go: ls/findallreferences.go:1718 getReferencesForNonModule

// Go: ls/findallreferences.go:1723 getMergedAliasedSymbolOfNamespaceExportDeclaration


// -- Core algorithm for find all references --

// Go: ls/findallreferences.go:1838 getSpecialSearchKind

// Go: ls/findallreferences.go:1856 getReferencedSymbolsForSymbol

// Go: ls/findallreferences.go:1888 refSearch
// Symbol that is currently being searched for.
// This will be replaced if we find an alias for the symbol.
// PORT: Go `includes` is a closure over `allSearchSymbols`; here it owns a
// copy of the list (map-ls-navigation 3.1).
pub struct RefSearch {
    // If coming from an export, we will not recursively search for the imported symbol (since that's where we came from).
    pub coming_from: ImpExpKind, // import, export

    pub symbol: SymbolId,
    pub text: String,
    pub escaped_text: String,

    // Only set if `options.implementations` is true. These are the symbols checked to get the implementations of a property access.
    pub parents: Vec<SymbolId>,

    pub all_search_symbols: Vec<SymbolId>,

    // Whether a symbol is in the search set.
    // Do not compare directly to `symbol` because there may be related symbols to search for. See `populateSearchSymbolSet`.
    pub includes: Rc<dyn Fn(SymbolId) -> bool>,
}

// Go: ls/findallreferences.go:1906 inheritKey
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InheritKey {
    pub symbol: SymbolId,
    pub parent: SymbolId,
}

// Go: ls/findallreferences.go:1911 refState
// PORT: the state borrows the checker, the context and the source file list
// and set for the whole search (`'c`). Go `collections.Set[*ast.Node]` node
// seen trackers are `FxHashSet<Node>`. `symbolToReferences` and
// `sourceFileToSeenSymbols` are lookups only; `result` keeps the Go order.

// Go: ls/findallreferences.go:1929 newState
// PORT: Go returns `*refState`; the state is returned by value.


// Go: ls/findallreferences.go:2020 getReferenceEntriesForShorthandPropertyAssignment

// Go: ls/findallreferences.go:2035 isMethodOrAccessor

// Go: ls/findallreferences.go:2039 tryGetClassByExtendingIdentifier

// Go: ls/findallreferences.go:2043 getClassConstructorSymbol
// PORT: Go reads the symbol fields directly; they are in the checker's arena.

// Go: ls/findallreferences.go:2050 hasOwnConstructor

// Go: ls/findallreferences.go:2054 findOwnConstructorReferences

// Go: ls/findallreferences.go:2083 findSuperConstructorAccesses

// Go: ls/findallreferences.go:2103 forEachDescendantOfKind


#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
    use crate::frontend::tspath::Path;
    use crate::scanner_util::port_byte_offset;
    use crate::scanner_util::{go_string_from_bytes, push_js_string_rune};

    // PORT: no Go test. Go searches the Go bytes of the text. The text is in
    // the source form, where a lone surrogate is 3 invalid byte units, and a
    // string literal's name in the value form, where it is one surrogate unit
    // (see `scanner_util::GO_STRING_MARKER`). The bytes next to a match are
    // Go bytes: 0xC5 (`Å`) and 0xFF (`ÿ`) are identifier parts, 0x80 is not.
    // The checker's copy of the search gives the same positions.
    #[test]
    fn possible_reference_positions_search_go_bytes() {
        let mut lone = String::new();
        push_js_string_rune(&mut lone, 0xD800);
        let cases: [(&[u8], &str, &[i32]); 3] = [
            (
                b"a = \"\xed\xa0\x80\"; b = \"\xed\xa0\x80\";",
                &lone,
                &[5, 16],
            ),
            (b"\xc5foo foo\x80 foo\xff", "foo", &[5]),
            // A real U+FDD0 is M + M in both forms.
            ("x\u{FDD0}y \u{FDD0}y".as_bytes(), "\u{FDD0}\u{FDD0}y", &[6]),
        ];
        for (bytes, name, want) in cases {
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
            let got = get_possible_symbol_reference_positions(file.root, name, Node::NIL);
            let want: Vec<i32> = want.iter().map(|&g| port_byte_offset(&text, g)).collect();
            assert_eq!(got, want, "{bytes:?} {name:?}");
            // Go checker/services.go:655, the identifier search of
            // `IsSymbolReferencedInFile`, has the same body.
            let checker_got = crate::checker::services::get_possible_symbol_reference_positions(
                file.root, name, file.root,
            );
            assert_eq!(checker_got, want, "checker: {bytes:?} {name:?}");
        }
        // A container that does not start the text: as in Go, the first
        // index is relative to it and is used as an absolute position. Here
        // that is Go offset 11, inside an `é` (0xC3 before it) or after an
        // invalid byte 0xFF, both identifier parts, so it is no match.
        let cases: [(&[u8], i32, &[i32]); 2] = [
            (
                "let ééééé = 1;function f(x = 1) { return x; }".as_bytes(),
                19,
                &[30, 46],
            ),
            (
                b"let a = \"\xff\xff\xff\xff\xff\";function f(x = 1) { return x; }",
                16,
                &[27, 43],
            ),
        ];
        for (bytes, go_container_pos, want) in cases {
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
            let x = port_byte_offset(&text, go_container_pos + 11);
            let function = astnav::get_touching_property_name(file.root, x)
                .parent()
                .parent();
            assert_eq!(function.kind(), SyntaxKind::FunctionDeclaration);
            assert_eq!(function.pos(), port_byte_offset(&text, go_container_pos));
            let got = get_possible_symbol_reference_positions(file.root, "x", function);
            let want: Vec<i32> = want.iter().map(|&g| port_byte_offset(&text, g)).collect();
            assert_eq!(got, want, "{bytes:?}");
        }
    }
}
