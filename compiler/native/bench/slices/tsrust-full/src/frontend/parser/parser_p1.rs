//! Port of typescript-go `internal/parser/parser.go` lines 1 to 1408: the
//! `Parser` struct, `ParseSourceFile`, JSON text, parser state, errors,
//! mark and rewind, token helpers, the source file worker, the list
//! functions, the `parseExpected*` helpers and `parseStatement` to
//! `parseCaseClause`.
//!
//! Parts U5 to U10 (parser_p2 to parser_p5, jsdoc.rs) add more
//! `impl Parser` blocks. They read and write the `Parser` fields directly.

use crate::frontend::prelude::*;
// PORT: explicit imports. `scanner_util` (in the prelude) still has the old
// `Scanner` and `new_scanner`. An explicit import wins over the glob.
use crate::frontend::scanner::scanner_p1::{
    ScannerState, TEXT_TO_KEYWORD, 
};
use std::cell::OnceCell;
use std::sync::LazyLock;

// Go: parser.go:19 ParsingContext
/// Go `ParsingContext`. The value is the bit index in `ParsingContexts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ParsingContext {
    SourceElements,           // Elements in source file
    BlockStatements,          // Statements in block
    SwitchClauses,            // Clauses in switch statement
    SwitchClauseStatements,   // Statements in switch clause
    TypeMembers,              // Members in interface or type literal
    ClassMembers,             // Members in class declaration
    EnumMembers,              // Members in enum declaration
    HeritageClauseElement,    // Elements in a heritage clause
    VariableDeclarations,     // Variable declarations in variable statement
    ObjectBindingElements,    // Binding elements in object binding list
    ArrayBindingElements,     // Binding elements in array binding list
    ArgumentExpressions,      // Expressions in argument list
    ObjectLiteralMembers,     // Members in object literal
    JsxAttributes,            // Attributes in jsx element
    JsxChildren,              // Things between opening and closing JSX tags
    ArrayLiteralMembers,      // Members in array literal
    Parameters,               // Parameters in parameter list
    JsDocParameters,          // JSDoc parameters in parameter list of JSDoc function type
    RestProperties,           // Property names in a rest type list
    TypeParameters,           // Type parameters in type parameter list
    TypeArguments,            // Type arguments in type argument list
    TupleElementTypes,        // Element types in tuple element type list
    HeritageClauses,          // Heritage clauses for a class or interface declaration.
    ImportOrExportSpecifiers, // Named import clause's import specifier list
    ImportAttributes,         // Import attributes
    JsDocComment,             // Parsing via JSDocParser
    Count,                    // Number of parsing contexts
}

impl ParsingContext {
}

// Go: parser.go:51 ParsingContexts
pub type ParsingContexts = i32;

// Go: parser.go:53 JSDocInfo
#[derive(Clone, Debug)]
pub struct JsDocInfo {
    pub parent: Node,
    pub js_docs: Vec<Node>,
}

// Go: parser.go:58 jsdocScannerInfo
pub type JsdocScannerInfo = u8;

pub const JSDOC_SCANNER_INFO_HAS_JS_DOC: JsdocScannerInfo = 1 << 0;
pub const JSDOC_SCANNER_INFO_HAS_DEPRECATED: JsdocScannerInfo = 1 << 1;
pub const JSDOC_SCANNER_INFO_HAS_SEE_OR_LINK: JsdocScannerInfo = 1 << 2;



// Go: parser.go:66 Parser
// PORT: Go `nodeSliceArena` and `stringSliceArena` are allocation arenas.
// Rust vectors own their data, so they are not ported. `hasParseError` is in
// `ParseDiagnostics` (see there). Go `setParentFromContext` is a closure
// field; p5 `override_parent_in_immediate_children` uses `current_parent`
// directly. `store` is the node store of the file (Go has no store).

// Go: parser.go:104 newParser
// PORT: Go keeps parsers in a `sync.Pool` (`getParser`/`putParser`). A new
// parser is made for each parse here.

// Go: parser.go:110 viableKeywordSuggestions
// PORT: Go calls `scanner.GetViableKeywordSuggestions()` (scanner.go:2288),
// which ranges over the `textToKeyword` map. Go map order is random. Here the
// order is the keyword table order.


// Go: parser.go:116 isMissingNodeList
// PORT: Go marks a missing list by its shared `missingListNodes` backing
// array. Rust lists do not share backing arrays, so `create_missing_list`
// marks the list with the astdata `has_trailing_comma` bit on an empty list.
// The store never sets that bit (Go computes the trailing comma from the
// list ends) and an empty list cannot have a trailing comma.

// Go: parser.go:135 ParseSourceFile
// PORT: Go `NewSourceFile` makes a heap node. Here the parse makes a node
// store for the file first and freezes it at the end. The store keeps the
// file name as `&'static str`, so the name is leaked once per file. The
// store and the result keep the text (`FileText`), and the parser borrows
// it while it runs.





// Go: parser.go:318 scanError
// PORT: Go `p.scanError` is a method. The scanner callback cannot borrow the
// parser, so it writes the shared `ParseDiagnostics` (see there).

// Go: parser.go:227 getErrorSpanForNode

// Go: parser.go:264 isDoubleQuotedString

// Go: parser.go:282 ParseIsolatedEntityName
// PORT: the nodes use the synthetic factory of `new_parser`, as there is
// no source file.

// Go: parser.go:341 ParserState
#[derive(Clone, Copy, Debug)]
pub struct ParserState<'a> {
    pub scanner_state: ScannerState<'a>,
    pub context_flags: NodeFlags,
    pub diagnostics_len: usize,
    pub js_diagnostics_len: usize,
    pub jsdoc_infos_len: usize,
    pub reparsed_clones_len: usize,
    pub statement_has_await_identifier: bool,
    pub has_parse_error: bool,
}

// Go: parser.go:1193 isDeclareModifier

#[cfg(test)]
mod debug_site_tests {
    use super::new_parser;
    use crate::core::go_panic_text;

    // Go `isInSomeParsingContext` (parser.go:744) runs
    // `debug.Assert(p.parsingContexts != 0, "Missing parsing context")`.
    #[test]
    fn missing_parsing_context_is_a_go_debug_failure() {
        let text = go_panic_text(|| {
            new_parser().is_in_some_parsing_context();
        });
        assert_eq!(
            text,
            "Debug failure. False expression: Missing parsing context"
        );
    }
}
